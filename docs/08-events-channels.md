# 第8章 Events と Channels

command は「フロント → Rust → 戻り値」の一往復です。Rust 側から能動的にフロントへ送る手段は2つあります。

| | Events | Channels |
|---|---|---|
| 用途 | 通知（「データが変わった」「メニューが押された」） | 1つの処理に紐づく連続データ（進捗、ログ、ストリーム） |
| 宛先 | 全ウィンドウにブロードキャスト（`emit_to` で特定ウィンドウ） | invoke した呼び出し元だけ |
| 順序・性能 | 小さいデータ向け | 順序保証・高スループット |
| Electron | `webContents.send` | `MessagePort` に近い |

**この章で触るファイル**

| ファイル | 作業 |
|---|---|
| `src-tauri/src/error.rs` | `Tauri` バリアントを追加 |
| `src-tauri/src/commands.rs` | create / update / delete に `emit` を追加、`analyze_notes` を追加 |
| `src-tauri/src/lib.rs` | `generate_handler!` に `analyze_notes` を追加 |
| `src/api.ts` | `AnalyzeEvent` 型と `analyzeNotes` を追加 |
| `src/App.tsx` | 手動の `reload()` をイベント購読に置き換え、解析ボタンを追加 |

## 8.1 Events：Rust → フロント

`emit` は失敗すると `tauri::Error` を返すので、まず `error.rs` に `?` で変換できるバリアントを足します。

```rust
// src-tauri/src/error.rs の AppError に追加
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
```

`commands.rs` の変更系3つで、`AppHandle` を受け取り、変更後に `emit` するようにします。

```rust
// src-tauri/src/commands.rs
use tauri::{AppHandle, Emitter, State}; // emit は Emitter trait のメソッド

#[tauri::command]
pub fn create_note(app: AppHandle, store: State<'_, NoteStore>, title: String) -> AppResult<Note> {
    let note = store.create(title)?;
    app.emit("notes-changed", ())?; // ペイロードは Serialize できれば何でもよい
    Ok(note)
}
```

`update_note` と `delete_note` も同じ形に直します（引数の先頭に `app: AppHandle` を足し、`Ok(...)` の前に `app.emit("notes-changed", ())?;` を入れる）。完成形は `reference/notes-app/src-tauri/src/commands.rs` にあります。`AppHandle` は Tauri が注入する引数なので、フロントの呼び出し側は変更不要です。

フロント側では、第7章で各操作の後に書いた `await reload();` を3か所とも消し、代わりにイベントを購読します。

```tsx
// src/App.tsx
import { listen } from "@tauri-apps/api/event";

async function reload() {
  const list = await api.listNotes();
  setNotes(list);
  // 別ウィンドウ等で削除されたメモを開いたままにしない
  setSelected((s) => (s && list.some((n) => n.id === s.id) ? s : null));
}

useEffect(() => {
  run(reload);
  const unlisten = listen("notes-changed", () => run(reload)); // Promise<UnlistenFn>
  return () => {
    unlisten.then((f) => f()); // 解除しないと StrictMode の二重マウントで二重登録になる
  };
}, []);
```

これで、どの操作で変更されても一覧が自動で更新されます。ウィンドウを複数開いた場合も、全ウィンドウが同期します。

- 特定のウィンドウだけに送るなら `app.emit_to("main", "event", payload)` を使います
- フロント → Rust 方向のイベント（JS の `emit`、Rust の `app.listen`）もありますが、基本は command を使いましょう

## 8.2 Channels：進捗のストリーミング

「全メモの文字数を解析し、進捗をリアルタイム表示する」機能を作ります。

```sh
cd src-tauri && cargo add tokio --features time && cd ..
```

`commands.rs` に追記します。

```rust
// src-tauri/src/commands.rs に追記
use serde::Serialize;
use std::time::Duration;
use tauri::ipc::Channel;

#[derive(Clone, Serialize)]
// TS の判別共用体として届く: {"kind":"progress","done":1,"total":3}
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AnalyzeEvent {
    Progress { done: usize, total: usize },
    Finished { total_chars: usize },
}

/// 全メモの文字数を数える。重い処理のふりをして進捗を Channel で送る
#[tauri::command]
pub async fn analyze_notes(
    store: State<'_, NoteStore>,
    on_event: Channel<AnalyzeEvent>,
) -> AppResult<()> {
    let notes = store.list();
    let total = notes.len();
    let mut total_chars = 0;
    for (i, note) in notes.iter().enumerate() {
        total_chars += note.body.chars().count();
        tokio::time::sleep(Duration::from_millis(300)).await; // 重い処理のふり
        on_event.send(AnalyzeEvent::Progress { done: i + 1, total })?;
    }
    on_event.send(AnalyzeEvent::Finished { total_chars })?;
    Ok(())
}
```

- `async fn` なのでメインスレッド（UI）をブロックしません
- `std::thread::sleep` ではなく `tokio::time::sleep` を使います（第5章）
- `lib.rs` の `generate_handler!` に `commands::analyze_notes` を追加するのを忘れずに

> **ハマりどころ**：enum に付けた `rename_all` は **バリアント名**（`kind` の値）にしか効きません。`rename_all_fields` がないと `total_chars` は snake_case のまま送られ、TS 側の `e.totalChars` は `undefined` になります（しかも型チェックは通ってしまいます）。
> こうした Rust ⇔ TS の形のずれは、`serde_json::to_string` の結果を assert する小さなテストで防げます（`reference/notes-app/src-tauri/src/commands.rs` の末尾を参照）。

フロント側では、まず `api.ts` に追記します（`import` 行は `Channel` を足した形に置き換えます）。

```ts
// src/api.ts
import { Channel, invoke } from "@tauri-apps/api/core";

export type AnalyzeEvent =
  | { kind: "progress"; done: number; total: number }
  | { kind: "finished"; totalChars: number };

export function analyzeNotes(onEvent: (e: AnalyzeEvent) => void) {
  const channel = new Channel<AnalyzeEvent>();
  channel.onmessage = onEvent;
  return invoke<void>("analyze_notes", { onEvent: channel }); // on_event → onEvent
}
```

続いて `App.tsx` に追記します。

```tsx
// src/App.tsx の App 内に追加
async function analyze() {
  await api.analyzeNotes((e) => {
    if (e.kind === "progress") setStatus(`解析中… ${e.done}/${e.total}`);
    else setStatus(`合計 ${e.totalChars} 文字`);
  });
}

// サイドバーの「＋ 新規」ボタンの下に追加
<button onClick={() => run(analyze)}>文字数を解析</button>
```

## ✅ 確認

- 新規作成・保存・削除の後に、手動の `reload()` なしで一覧が更新される
- 「文字数を解析」で `解析中… 1/3` → `合計 N 文字` と表示が変わる

## やってみよう

1. `rename_all_fields` を一時的に消して「合計 undefined 文字」になることを確かめ、`console.log(e)` で実際に届いた JSON を見てみる
2. 発展：解析中に「キャンセル」できるようにする
   - ヒント1：`AtomicBool`（第7章の `AtomicU64` の bool 版）を持つ `struct AnalyzeCancel(AtomicBool)` を作り、`setup` で `manage` する
   - ヒント2：`analyze_notes` の開始時に `store(false, Ordering::Relaxed)` でリセットし、ループ内で `load(Ordering::Relaxed)` が true なら `break` する
   - ヒント3：`cancel_analyze` command で `store(true, ...)` する。この command は解析中に呼ばれるので、解析側が `async` でないとメインスレッドが塞がって呼べない点にも注目
