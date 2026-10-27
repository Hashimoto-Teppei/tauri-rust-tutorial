# 第9章 プラグインと Capabilities

## 9.1 Tauri 2.0 のセキュリティモデル

Electron では「Renderer に Node の力を渡さない」ために、`contextIsolation` や preload を自分で正しく設定する必要がありました。
Tauri 2.0 は **デフォルト拒否** です。フロントから呼べるのは次の2つだけです。

1. 自分で `generate_handler!` に登録した **command**（既定では全ウィンドウから呼べる）
2. **capability ファイルで許可した**、Tauri コアの API とプラグインの API

```jsonc
// src-tauri/capabilities/default.json（実際のファイルにはコメントを書かないこと）
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],              // どのウィンドウに適用するか
  "permissions": [
    "core:default",                 // イベント・パス等の基本 API
    "opener:default",
    "dialog:default"
  ]
}
```

- `$schema` があるので、VS Code で **権限名の補完** が効きます（`pnpm tauri dev` を一度実行すると `gen/schemas` が生成されます）
- 許可していない API を呼ぶと、invoke が `"... not allowed ..."` というエラーで reject されます
- `windows` を分ければ「設定ウィンドウだけ fs を許可」のような最小権限の設計ができます
- 自作の command も、`build.rs` で `tauri_build::Attributes::new().app_manifest(...)` を使って権限化すれば capability で制御できます（中〜大規模アプリ向け。本チュートリアルでは扱いません）

| 権限の書き方 | 意味 |
|---|---|
| `"dialog:default"` | プラグイン推奨の権限セット |
| `"dialog:allow-save"` | 個別の API だけ許可 |
| `"fs:deny-remove"` | 明示的に拒否 |
| `{ "identifier": "fs:allow-read-text-file", "allow": [{ "path": "$APPDATA/**" }] }` | **スコープ付き**の許可 |

## 9.2 プラグインの追加

```sh
pnpm tauri add dialog
```

この1コマンドで、次の4つがすべて行われます。

1. `Cargo.toml` に `tauri-plugin-dialog` を追加
2. `package.json` に `@tauri-apps/plugin-dialog` を追加
3. `lib.rs` に `.plugin(tauri_plugin_dialog::init())` を追加
4. `capabilities/default.json` に `"dialog:default"` を追加

主な公式プラグインは次のとおりです。

| プラグイン | 用途 | Electron での相当 |
|---|---|---|
| `dialog` | ファイル選択・保存・メッセージダイアログ | `dialog` |
| `fs` | ファイル読み書き（スコープ制限付き） | `fs`（Node） |
| `http` | CORS に縛られない HTTP リクエスト | Node の `fetch` / `net` |
| `store` | 永続 key-value ストア | electron-store |
| `sql` | SQLite など | better-sqlite3 |
| `opener` | URL やファイルを既定のアプリで開く | `shell.openExternal` |
| `notification` | OS 通知 | `Notification` |
| `clipboard-manager` | クリップボード | `clipboard` |
| `global-shortcut` | グローバルショートカット | `globalShortcut` |
| `updater` | 自動更新 | electron-updater |
| `window-state` | ウィンドウの位置・サイズを記憶 | electron-window-state |
| `single-instance` | 多重起動の防止 | `requestSingleInstanceLock` |

一覧：https://v2.tauri.app/plugin/

## 9.3 メモの書き出し機能

「保存先を選ぶ」のは dialog プラグイン（フロント）、「書き込む」のは自前の command（Rust）が担当します。

**この節で触るファイル**

| ファイル | 作業 |
|---|---|
| `src-tauri/src/error.rs` | `InvalidPath` バリアントを追加 |
| `src-tauri/src/commands.rs` | `export_note` を追加 |
| `src-tauri/src/lib.rs` | `generate_handler!` に `commands::export_note` を追加 |
| `src/api.ts` | `exportNote` を追加 |
| `src/App.tsx` | `exportNote` 関数と「書き出し」ボタンを追加 |

```rust
// src-tauri/src/error.rs の AppError に追加
    #[error("書き出し先は .md ファイルのみ許可しています: {0}")]
    InvalidPath(String),
```

```rust
// src-tauri/src/commands.rs に追記（use 行の AppResult を {AppError, AppResult} に変更）
use crate::error::{AppError, AppResult};

/// フロントのダイアログで選ばれたパスへ Markdown として書き出す
#[tauri::command]
pub fn export_note(store: State<'_, NoteStore>, id: u64, path: String) -> AppResult<()> {
    // command は capability の scope 外で動くので、受け取ったパスは自分で検証する
    if !path.ends_with(".md") {
        return Err(AppError::InvalidPath(path));
    }
    let note = store.get(id)?;
    std::fs::write(path, format!("# {}\n\n{}", note.title, note.body))?;
    Ok(())
}
```

```ts
// src/api.ts に追記
export const exportNote = (id: number, path: string) => invoke<void>("export_note", { id, path });
```

```tsx
// src/App.tsx
import { save } from "@tauri-apps/plugin-dialog";

async function exportNote() {
  if (!selected) return;
  const path = await save({
    defaultPath: `${selected.title}.md`,
    filters: [{ name: "Markdown", extensions: ["md"] }],
  });
  if (!path) return; // キャンセル時は null
  await api.exportNote(selected.id, path);
  setStatus(`書き出しました: ${path}`);
}

// 「保存」と「削除」の間に追加
<button onClick={() => run(exportNote)}>書き出し</button>
```

### fs プラグインを使わなかった理由と、その代償

書き込みは `@tauri-apps/plugin-fs` の `writeTextFile` でもできます。ただしその場合は、**書き込んでよい場所をスコープで設定する** 必要があります。ユーザーが選ぶ任意の場所への書き込みはスコープで表しにくいため、ここでは Rust 側で書いています。

一方で、自作の command は既定では capability の制限を受けません。フロントが XSS などで乗っ取られると、`export_note` に任意のパスを渡せてしまいます。そのため **command の引数は信頼できない入力として検証する** のが原則です（ここでは拡張子だけをチェックしています）。

## 9.4 フロントエンドの CSP

`tauri.conf.json` の `app.security.csp` は初期状態で `null`（無効）です。外部リソースを読み込まないアプリなら、設定しておくと XSS の被害を抑えられます。

```json
"security": {
  "csp": "default-src 'self'; connect-src ipc: http://ipc.localhost; style-src 'self' 'unsafe-inline'"
}
```

- `connect-src ipc: http://ipc.localhost` は **Tauri の IPC（invoke）の通り道** です。これがないと IPC が遅い代替経路にフォールバックし、Console に警告が出ます
- Tauri はビルド時に、この CSP へスクリプトの nonce や hash を自動で追加します
- `reference/` では `null` のままにしてあります（下の「やってみよう」の課題です）

## ✅ 確認

- 「書き出し」で保存ダイアログが開き、選んだ場所に `.md` ファイルができる
- ダイアログで拡張子を `.txt` に変えると、ステータス欄に「書き出し先は .md ファイルのみ…」と表示される

## やってみよう

1. `capabilities/default.json` から `"dialog:default"` を消して書き出しを試し、エラーメッセージを確認する（確認したら戻す）
2. 9.4 の CSP を設定し、DevTools の Console に CSP 違反や IPC の警告が出ないことを確認する
3. `pnpm tauri add store` で store プラグインを追加し、「最後に開いたメモの id」を保存・復元する
   - ヒント：`import { load } from "@tauri-apps/plugin-store"; const store = await load("settings.json");` → `await store.set("lastId", id)` / `await store.get<number>("lastId")`。API の詳細は https://v2.tauri.app/plugin/store/ を参照
4. 発展：`pnpm tauri add fs` を実行し、`$DOCUMENT/notes-export/**` にだけ書き込める capability を作って、フロントから `writeTextFile` で書き出す版を作る
