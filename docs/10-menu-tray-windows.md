# 第10章 メニュー・トレイ・ウィンドウ

ネイティブ UI は、Rust 側の `setup` で組み立てるのが基本です。JS API（`@tauri-apps/api/menu`）もありますが、起動時に確定するものは Rust で書く方が素直です。

**この章で触るファイル**

| ファイル | 作業 |
|---|---|
| `src-tauri/Cargo.toml` | `tauri` に `tray-icon` 機能を追加 |
| `src-tauri/src/lib.rs` | `setup` にメニューとトレイを追記、`on_menu_event` を追加 |
| `src/App.tsx` | `menu-new-note` イベントを購読 |

この章の完成形が `reference/notes-app` と一致します。10.3 以降は `reference/` に含めていない「やってみよう」向けの内容です。

## 10.1 アプリケーションメニュー

`lib.rs` の先頭の `use` を次のように増やします。

```rust
use tauri::menu::{Menu, MenuItem, Submenu};
use tauri::{Emitter, Manager};
```

第7章で書いた `setup` クロージャの **`Ok(())` の直前** に追記します（`.setup` を2回呼ぶと後の呼び出しが勝ち、State の登録が消えてしまうので、必ず同じクロージャの中に書きます）。

```rust
.setup(|app| {
    // ...第7章の State 登録はそのまま...

    // メニュー（macOS の標準メニューを残したまま「メモ」を追加）
    let new_note =
        MenuItem::with_id(app, "new_note", "新規メモ", true, Some("CmdOrCtrl+N"))?;
    let notes_menu = Submenu::with_items(app, "メモ", true, &[&new_note])?;
    let menu = Menu::default(app.handle())?;
    menu.append(&notes_menu)?;
    app.set_menu(menu)?;
    Ok(())
})
```

`.setup(...)` の直後に、メニューのクリックを受け取るハンドラを足します。

```rust
// アプリメニューとトレイメニュー両方のクリックがここに来る
.on_menu_event(|app, event| match event.id().as_ref() {
    "new_note" => {
        let _ = app.emit_to("main", "menu-new-note", ()); // main ウィンドウだけに送る
    }
    _ => {}
})
```

- `emit` ではなく `emit_to` を使うのは、ウィンドウが複数あると全ウィンドウがメモを作ってしまうからです
- `let _ =` は「Result を意図的に捨てる」書き方です（ここでは失敗しても打つ手がないため）

> **macOS の落とし穴**：`Menu::new` で一から作ると、標準の「編集」メニューが消えて **⌘C / ⌘V が効かなくなります**（macOS ではコピー＆ペーストのショートカットがメニュー経由で動くため）。`Menu::default` をベースに追加するか、自作する場合は `PredefinedMenuItem::copy` などを含めてください。

`MenuItem::with_id` の引数は `(manager, id, テキスト, 有効か, アクセラレータ)` です。アクセラレータが不要なら `None::<&str>` と型を明示します（`None` だけだと型推論できないため）。

フロント側では、第8章の `useEffect` で購読するイベントを増やします。

```tsx
// src/App.tsx の useEffect を置き換え
useEffect(() => {
  run(reload);
  const unlisteners = [
    listen("notes-changed", () => run(reload)),
    listen("menu-new-note", () => run(addNote)),
  ];
  return () => {
    unlisteners.forEach((p) => p.then((unlisten) => unlisten()));
  };
}, []);
```

## 10.2 システムトレイ

`Cargo.toml` で機能フラグを有効にします。

```toml
tauri = { version = "2", features = ["tray-icon"] }
```

`use` に `use tauri::tray::TrayIconBuilder;` を足し、`setup` の `Ok(())` の直前（10.1 のメニューの後）に追記します。

```rust
// トレイアイコン
let show = MenuItem::with_id(app, "show", "ウィンドウを表示", true, None::<&str>)?;
let quit = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
let tray_menu = Menu::with_items(app, &[&show, &quit])?;
TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&tray_menu)
    .build(app)?;
```

トレイメニューのクリックも `on_menu_event` に届くので、`match` に分岐を足します。

```rust
"show" => {
    if let Some(w) = app.get_webview_window("main") { // Manager trait のメソッド
        let _ = w.show();
        let _ = w.set_focus();
    }
}
"quit" => app.exit(0),
```

ここまでで `reference/notes-app/src-tauri/src/lib.rs` と同じ内容になります。

## ✅ 確認

- メニューバー（Windows はウィンドウ上部）に「メモ」が増え、⌘N（Windows は Ctrl+N）で新規メモが作られる
- ⌘C / ⌘V（Windows は Ctrl+C / Ctrl+V）が引き続き使える
- トレイ（macOS はメニューバー右側、Windows はタスクバー右下の通知領域。`^` で隠れている場合あり）のアイコンから「終了」できる

## 10.3 閉じるボタンでトレイに格納する（やってみよう）

```rust
// .on_menu_event(...) の後に追加
.on_window_event(|window, event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == "main" {
            // main だけ隠す（他のウィンドウは普通に閉じられるように）
            let _ = window.hide();
            api.prevent_close();
        }
    }
})
```

トレイの「ウィンドウを表示」で復帰できることを確認しましょう。macOS で Dock アイコンのクリックでも復帰させたい場合は、`RunEvent::Reopen` を処理する必要があります（`.run(...)` を `.build(...)?.run(|app, event| ...)` の形に変えて受け取ります）。

## 10.4 ウィンドウ

### 設定ファイルで定義する

`tauri.conf.json` の `app.windows` を書き換えると、初期ウィンドウを変えられます。

```json
"windows": [
  {
    "label": "main",
    "title": "Tauri Notes",
    "width": 960,
    "height": 640,
    "minWidth": 600,
    "minHeight": 400
  }
]
```

### 動的に開く（Rust）

```rust
use tauri::{WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
async fn open_settings(app: tauri::AppHandle) -> AppResult<()> {
    WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("index.html".into()))
        .title("設定")
        .inner_size(400.0, 300.0)
        .build()?;
    Ok(())
}
```

同じ `index.html` を読み込むので、フロント側ではウィンドウの label を見て表示を切り替えます。

```tsx
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const isSettings = getCurrentWebviewWindow().label === "settings";
```

- Windows では、同期 command の中でウィンドウを作るとデッドロックすることがあります。**ウィンドウの生成は `async` command で** 行います
- 新しいウィンドウにも権限が必要なら、capability の `"windows"` に `"settings"` を追加します

## やってみよう

1. 10.3 の「閉じるボタンでトレイに格納」を実装する
2. 10.4 の `open_settings` を実装し、メニューに「設定…」（`CmdOrCtrl+,`）を追加して開けるようにする
3. 発展：`pnpm tauri add window-state` で、ウィンドウの位置とサイズを記憶させる
