# 第6章 Tauri 概観と Electron との違い

## 6.1 アーキテクチャ比較

| | Electron | Tauri 2.0 |
|---|---|---|
| バックエンド | Main プロセス（Node.js） | **Core プロセス（Rust）** |
| フロントエンド | Renderer（同梱の Chromium） | **OS 標準の WebView**（macOS: WKWebView, Windows: WebView2, Linux: WebKitGTK） |
| 橋渡し | preload + `contextBridge` + `ipcRenderer.invoke` / `ipcMain.handle` | **Commands**（`invoke`）と **Events** / **Channels** |
| セキュリティ | `contextIsolation`, `nodeIntegration: false` を自分で設定 | **Capabilities**（許可したAPIしか呼べない。デフォルト拒否） |
| ネイティブ機能 | Node の fs 等を main で直接 / npm モジュール | Rust 標準ライブラリ + **公式プラグイン**（fs, dialog, store, updater…） |
| バイナリサイズ | 〜100MB 以上 | 数MB〜十数MB |
| モバイル | なし | iOS / Android 対応（2.0〜） |

### 概念の対応表（Electron 経験者向け）

| Electron | Tauri |
|---|---|
| `ipcMain.handle("ch", fn)` | `#[tauri::command] fn ch(...)` + `generate_handler![ch]` |
| `ipcRenderer.invoke("ch", args)` | `invoke("ch", { args })` |
| `webContents.send("ev", data)` | `app.emit_to("main", "ev", data)`（`app.emit` は全ウィンドウへのブロードキャスト） |
| `ipcRenderer.on("ev", fn)` | `listen("ev", fn)` |
| main プロセスのグローバル変数 | `app.manage(state)` + `State<T>` |
| `app.getPath("userData")` | `app.path().app_data_dir()` |
| `BrowserWindow` | `WebviewWindow`（`tauri.conf.json` または Rust で生成） |
| `Menu.setApplicationMenu` | `app.set_menu(menu)` |
| `Tray` | `TrayIconBuilder` |
| `dialog.showSaveDialog` | `@tauri-apps/plugin-dialog` の `save()` |
| electron-forge make | `pnpm tauri build` |
| electron-updater | `tauri-plugin-updater` |

### Electron の常識が通じない5点

1. **フロントに Node.js はない**：WebView の中で `fs`, `path`, `child_process` や Node 向け npm パッケージは動きません。ファイルや OS の機能は「プラグイン」か「自作の command」経由で使います
2. **`fetch` は CORS の制約を受ける**：普通のブラウザと同じです。外部 API を CORS 無視で叩きたいなら `http` プラグインか、Rust 側（`reqwest` など）で通信します
3. **Rust 側にブレークポイントは張れない（DevTools では）**：Rust のデバッグは `println!` / `dbg!` が基本です。本格的にやるなら VS Code の CodeLLDB 拡張を使います
4. **Rust のコンパイルエラーはターミナルにしか出ない**：`pnpm tauri dev` 中に Rust がコンパイルできないと、ウィンドウは古いバイナリのまま動き続けます。「変更が反映されない」と思ったらターミナルを見ましょう
5. **プロセス間の境界は JSON**：command の引数・戻り値はすべて JSON にシリアライズされます。`Date`、`Map`、クラスのインスタンス、関数はそのままでは渡せません

### 注意点：WebView の差

Chromium を同梱しないため、**OS によってレンダリングエンジンが違います**。macOS（Safari 系）で動いても Windows（Chromium 系 Edge）で見た目が違う、ということが起こり得ます。最新の CSS / JS 機能を使うときは Safari の対応状況を確認しましょう。

## 6.2 プロジェクト作成

```sh
pnpm create tauri-app notes-app --template react-ts --manager pnpm --identifier com.example.notes
cd notes-app
pnpm install
pnpm tauri dev      # 初回は Rust の依存ビルドで数分かかる
```

> 本リポジトリでは `notes-app/` をリポジトリ直下に作り、章ごとに育てていきます。完成形は `reference/notes-app/` にあります。
> - テンプレートは時期によって依存のバージョンが変わります。`package.json` や `Cargo.toml` のバージョンが `reference/notes-app` と大きく違う場合は、reference に合わせてください
> - `notes-app/` は `exercises/` の Cargo workspace の外にあるので、リポジトリ直下から cargo を使うときは `cargo test --manifest-path notes-app/src-tauri/Cargo.toml` のように指定します

## 6.3 生成されたファイル

```
notes-app/
├── src/                     # フロントエンド（普通の Vite + React）
│   ├── App.tsx
│   └── main.tsx
├── index.html
├── vite.config.ts           # 開発サーバ: localhost:1420 固定
├── package.json
└── src-tauri/               # Rust 側（Electron の main プロセスに相当）
    ├── Cargo.toml
    ├── tauri.conf.json      # アプリ名, ウィンドウ, ビルド, バンドル設定
    ├── capabilities/
    │   └── default.json     # フロントに許可する権限
    ├── icons/
    ├── build.rs             # tauri-build（触らない）
    └── src/
        ├── main.rs          # エントリポイント。lib の run() を呼ぶだけ
        └── lib.rs           # 本体。ここに Builder を書く
```

`main.rs` と `lib.rs` が分かれているのはモバイル対応のため（モバイルでは lib がエントリになる）。デスクトップのみでも `lib.rs` に書くのが慣習です。

### lib.rs（初期状態）

```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()                        // ビルダーパターンで組み立て
        .plugin(tauri_plugin_opener::init())         // プラグイン登録
        .invoke_handler(tauri::generate_handler![greet]) // command 登録
        .run(tauri::generate_context!())             // tauri.conf.json を埋め込んで起動
        .expect("error while running tauri application");
}
```

### フロント側

```ts
import { invoke } from "@tauri-apps/api/core";
setGreetMsg(await invoke("greet", { name }));
```

### tauri.conf.json の要所

```jsonc
{
  "productName": "notes-app",
  "identifier": "com.example.notes",   // 逆ドメイン形式。データ保存ディレクトリ名にもなる
  "build": {
    "beforeDevCommand": "pnpm dev",     // tauri dev 時に vite を起動
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../dist"           // 本番ではビルド済み静的ファイルを同梱
  },
  "app": {
    "windows": [{ "title": "notes-app", "width": 800, "height": 600 }], // label 省略時は "main"
    "security": { "csp": null }
  }
}
```

## 6.4 開発のコツ

- **フロントの変更は HMR で即反映、Rust の変更は自動で再ビルド＆再起動** されます
- WebView の DevTools は右クリック →「検証」で開けます（dev ビルドのみ。ショートカットは macOS が ⌘⌥I、Windows が Ctrl+Shift+I）
- Rust の `println!` / `dbg!` は `pnpm tauri dev` を実行したターミナルに出ます
- Rust 側のテストは `cargo test --manifest-path src-tauri/Cargo.toml`

## やってみよう

`greet` を改造して、引数に `age: u32` を追加し、フロントから `invoke("greet", { name, age })` で呼んでみましょう。
`age` に文字列を渡すとどうなるか（→ invoke が reject される）も確認してください。serde による自動バリデーションです。
