# 第7章 command と State — メモの CRUD

この章で Notes アプリの土台を作ります：メモの作成・一覧・更新・削除と、JSON ファイルへの永続化です。

**この章で触るファイル**（すべて `notes-app/` 配下）

| ファイル | 作業 |
|---|---|
| `src-tauri/src/error.rs` | 新規作成 |
| `src-tauri/src/notes.rs` | 新規作成（`reference/` からコピー） |
| `src-tauri/src/commands.rs` | 新規作成 |
| `src-tauri/src/lib.rs` | 全体を置き換え（`greet` は削除） |
| `src/api.ts` | 新規作成 |
| `src/App.tsx`, `src/App.css` | 全体を置き換え |

## 7.1 command の基本ルール

```rust
#[tauri::command]
fn add(a: i32, b: i32) -> i32 { a + b }
```

```ts
await invoke<number>("add", { a: 1, b: 2 });
```

- 引数は **JSON から Deserialize できる型**、戻り値は **Serialize できる型** にする
- 引数名は Rust の snake_case が **フロントでは camelCase** になる（`note_id` → `{ noteId }`）
- 戻り値が `Result<T, E>` なら `Ok` → resolve、`Err` → **reject**（`E` も Serialize 必須）
- 同期関数（`fn`）は **メインスレッド** で実行される。重い処理は `async fn` にする
- `async fn` で参照型の引数（`&str`, `State<'_, T>`）を使う場合、戻り値は `Result` にする必要がある

### 特別な引数（フロントからは渡さない。Tauri が注入する）

| 型 | 内容 |
|---|---|
| `tauri::AppHandle` | アプリ全体へのハンドル（emit、パス取得、ウィンドウ取得…） |
| `tauri::State<'_, T>` | `manage` した状態 |
| `tauri::WebviewWindow` | 呼び出し元ウィンドウ |
| `tauri::ipc::Channel<T>` | ストリーム送信用（第8章） |

## 7.2 エラー型：`error.rs`

```sh
cd src-tauri && cargo add thiserror && cd ..
```

第4章の thiserror パターンに、**Serialize の実装** を足すだけです。

```rust
// src-tauri/src/error.rs
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("メモが見つかりません (id: {0})")]
    NotFound(u64),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

// フロントエンドへは Display の文字列として渡す（invoke の catch で受け取れる）
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

`std::io::Error` は Serialize を実装していないので、そのままでは command の戻り値に使えません。それを包むのがこの型の役目です。バリアントは第8章・第9章で1つずつ増やします。

## 7.3 State：アプリ全体の状態

Electron で main プロセスのモジュールスコープ変数に持っていたものは、Tauri では `manage` で登録します。

```rust
app.manage(MyState { ... });                     // 登録（型ごとに1つ）

#[tauri::command]
fn handler(state: State<'_, MyState>) { ... }    // 取り出し
```

- Tauri が内部で `Arc` 管理するので、自分で `Arc` を書く必要はありません
- command は複数スレッドから同時に呼ばれ得るので、**書き換えるフィールドは `Mutex` で包みます**

## 7.4 データ層：`notes.rs`

`reference/notes-app/src-tauri/src/notes.rs` を **丸ごと** `notes-app/src-tauri/src/notes.rs` にコピーしてください。Tauri に依存しない普通の Rust コードなので、Part 1 の知識で読めます。読みどころは次の4点です。

```rust
pub struct NoteStore {
    path: PathBuf,
    notes: Mutex<Vec<Note>>,
    last_id: AtomicU64,
}
```

**① `&self` なのに書き換えられる。** メソッドはすべて `&self`（不変参照）ですが、`Mutex` のおかげで中身を書き換えられます。これを **内部可変性** と呼びます。`lock()` が返すガードはスコープを抜けると自動でアンロックされるので、TS の `try/finally` は要りません。

**② 変更は `modify` に集約する。**

```rust
fn modify<T>(&self, f: impl FnOnce(&mut Vec<Note>) -> AppResult<T>) -> AppResult<T> {
    let mut notes = self.notes.lock().unwrap();
    let mut next = notes.clone();
    let result = f(&mut next)?;
    self.save(&next)?;
    *notes = next;
    Ok(result)
}
```

流れは「コピーを変更 → 保存 → 成功したらメモリに反映」です。ディスクへの保存に失敗してもメモリと内容がずれません。引数は第4章の `FnOnce` クロージャです。

**③ 保存は一時ファイル経由。** `save` は `notes.json.tmp` に書いてから `rename` します。書き込み中にクラッシュしても `notes.json` は壊れません。

**④ テストが同居している。** ファイル末尾に `#[cfg(test)]` のテストがあり、Tauri を起動せずに `cargo test` で検証できます。ビジネスロジックを Tauri から切り離しておくのはこのためです。

> `get` メソッドは第9章で使うので、それまでは「`get` is never used」という警告が出ます（エラーではありません）。

## 7.5 command：`commands.rs`

```rust
// src-tauri/src/commands.rs
use crate::error::AppResult;
use crate::notes::{Note, NoteStore};
use tauri::State;

#[tauri::command]
pub fn list_notes(store: State<'_, NoteStore>) -> Vec<Note> {
    store.list()
}

#[tauri::command]
pub fn create_note(store: State<'_, NoteStore>, title: String) -> AppResult<Note> {
    store.create(title)
}

#[tauri::command]
pub fn update_note(
    store: State<'_, NoteStore>,
    id: u64,
    title: String,
    body: String,
) -> AppResult<Note> {
    store.update(id, title, body)
}

#[tauri::command]
pub fn delete_note(store: State<'_, NoteStore>, id: u64) -> AppResult<()> {
    store.delete(id)
}
```

## 7.6 組み立て：`lib.rs`

保存先は OS ごとのアプリデータディレクトリです（macOS なら `~/Library/Application Support/com.example.notes/`、Windows なら `C:\Users\<ユーザー名>\AppData\Roaming\com.example.notes\`）。パスの取得に `AppHandle` が要るので、起動時に1回呼ばれる `setup` フックで State を登録します。

`lib.rs` を丸ごと次の内容に置き換えます。

```rust
// src-tauri/src/lib.rs
mod commands;
mod error;
mod notes;

use notes::NoteStore;
use tauri::Manager; // app.path() / app.manage() を使うのに必要な trait

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 起動時にデータを読み込み State に登録
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            app.manage(NoteStore::load(dir.join("notes.json"))?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_notes,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- `use tauri::Manager;` を忘れると「no method named `path`」というエラーになります。Rust では **trait のメソッドは、その trait を use しないと呼べません**。Tauri で最もよく遭遇するエラーの一つです
- `invoke_handler` が有効なのは1回だけです。2回呼ぶと後の呼び出しが勝ち、先に登録した command は消えます。すべての command を1つの `generate_handler!` にまとめてください

ここで一度 `pnpm tauri dev` を実行し、コンパイルが通ることを確認しましょう（画面はまだテンプレートのままで、Greet ボタンは動かなくなっています）。

## 7.7 フロント側：`api.ts`

`invoke` をコンポーネントに散らばらせず、Electron の preload で API を定義していたように1ファイルにまとめます。

```ts
// src/api.ts
// Rust の command を型付きで呼ぶ薄いラッパー（Electron の preload + contextBridge の役割）
import { invoke } from "@tauri-apps/api/core";

export type Note = {
  id: number;
  title: string;
  body: string;
  updatedAt: number;
};

export const listNotes = () => invoke<Note[]>("list_notes");
export const createNote = (title: string) => invoke<Note>("create_note", { title });
export const updateNote = (id: number, title: string, body: string) =>
  invoke<Note>("update_note", { id, title, body });
export const deleteNote = (id: number) => invoke<void>("delete_note", { id });
```

> Rust と TS で型を二重に定義するのが嫌になったら、`tauri-specta` や `ts-rs` で Rust の型から TS の型を自動生成できます。

## 7.8 UI：`App.tsx`

`src/App.css` は `reference/notes-app/src/App.css` をそのままコピーし、`src/App.tsx` は次の内容に置き換えます（`src/assets/react.svg` は不要になるので消して構いません）。

```tsx
// src/App.tsx
import { useEffect, useState } from "react";
import * as api from "./api";
import type { Note } from "./api";
import "./App.css";

function App() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [selected, setSelected] = useState<Note | null>(null);
  const [status, setStatus] = useState("");

  // invoke の reject（Rust の AppError の Display 文字列）を画面に出す
  const run = (action: () => Promise<unknown>) =>
    action().catch((e) => setStatus(`エラー: ${e}`));

  async function reload() {
    setNotes(await api.listNotes());
  }

  useEffect(() => {
    run(reload);
  }, []);

  async function addNote() {
    setSelected(await api.createNote("無題のメモ"));
    await reload(); // 第8章でイベント購読に置き換える
  }

  async function saveNote() {
    if (!selected) return;
    // 戻り値で selected を置き換えない（保存中に入力した内容を消さないため）
    await api.updateNote(selected.id, selected.title, selected.body);
    setStatus("保存しました");
    await reload();
  }

  async function removeNote() {
    if (!selected) return;
    await api.deleteNote(selected.id);
    setSelected(null);
    await reload();
  }

  return (
    <main className="layout">
      <aside className="sidebar">
        <button onClick={() => run(addNote)}>＋ 新規</button>
        <ul>
          {notes.map((n) => (
            <li
              key={n.id}
              className={n.id === selected?.id ? "active" : ""}
              onClick={() => setSelected(n)}
            >
              {n.title}
            </li>
          ))}
        </ul>
      </aside>

      <section className="editor">
        {selected ? (
          <>
            <input
              value={selected.title}
              onChange={(e) => setSelected({ ...selected, title: e.target.value })}
            />
            <textarea
              value={selected.body}
              onChange={(e) => setSelected({ ...selected, body: e.target.value })}
            />
            <div className="actions">
              <button onClick={() => run(saveNote)}>保存</button>
              <button onClick={() => run(removeNote)}>削除</button>
            </div>
          </>
        ) : (
          <p>メモを選択してください</p>
        )}
        <p className="status">{status}</p>
      </section>
    </main>
  );
}

export default App;
```

## ✅ 確認

- 「＋ 新規」でメモが増え、編集して「保存」するとタイトルが一覧に反映される
- アプリを終了して `pnpm tauri dev` で起動し直しても、メモが残っている
- `cargo test --manifest-path src-tauri/Cargo.toml` で `notes::tests::crud_roundtrip` が通る

### よくあるエラー

| 症状 | 原因と対処 |
|---|---|
| `no method named 'path' found` | `use tauri::Manager;` がない |
| 起動直後に `state not managed for field ...` で panic | `setup` 内の `app.manage(...)` が実行されていない（`.setup` を2回呼ぶと後の呼び出しが勝つ） |
| invoke が `command xxx not found` で reject | `generate_handler!` への登録漏れ、またはフロントの command 名の綴りミス |
| invoke が `missing required key ...` で reject | 引数名の不一致。Rust の `note_id` はフロントでは `noteId` |

## やってみよう

1. `notes.json` の場所を `println!("{:?}", dir);` で出力し、中身を開いて見てみる（出力は `pnpm tauri dev` を実行したターミナルに出ます）
2. `api.updateNote(999, "x", "y")` を DevTools の Console から呼び、エラー文字列が返ることを確認する（`import` できないので、一時的に `window.api = api` を `App.tsx` に書くと楽です）
3. 発展：タイトルか本文に文字列を含むメモだけを返す `search_notes` を追加する
   - ヒント1：`notes.rs` に `pub fn search(&self, query: &str) -> Vec<Note>` を追加。`self.list().into_iter().filter(|n| n.title.contains(query) || n.body.contains(query)).collect()` の形で書ける
   - ヒント2：`commands.rs` に `search_notes(store, query: String)` を追加し、**`generate_handler!` への登録を忘れずに**
   - ヒント3：大文字小文字を無視するなら、両方を `to_lowercase()` してから比較する
