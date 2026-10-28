# 付録C 逆引きレシピ

「TS / Python ならこう書くけど、Rust では？」「Electron のあれは Tauri では？」を引くための付録です。各レシピは最小限のコード片と、詳しく説明している章へのリンクだけを載せています。

- **C.1〜C.6**：Rust（TS / Python からの読み替え）
- **C.7〜C.11**：Tauri（Electron からの読み替え）
- **C.12**：エラーメッセージから引く

コード片の `s`, `v`, `m` などは、直前の行で宣言したものとして読んでください。

## C.1 文字列

| やりたいこと | TS / Python | Rust |
|---|---|---|
| 埋め込み | `` `${a}-${b}` `` / `f"{a}-{b}"` | `format!("{a}-{b}")` |
| デバッグ表示 | `console.log(obj)` / `print(repr(obj))` | `println!("{obj:?}")`（`#[derive(Debug)]` が必要） |
| 連結 | `a + b` | `format!("{a}{b}")`、または `a.push_str(&b)`（`a` は `let mut a: String`） |
| 長さ（文字数） | `s.length` / `len(s)` | `s.chars().count()`（`s.len()` は **バイト数**） |
| 前後の空白を削る | `s.trim()` / `s.strip()` | `s.trim()` |
| 含む・前方一致 | `s.includes(x)` / `x in s` | `s.contains(x)` / `s.starts_with(x)` |
| 分割 | `s.split(",")` | `s.split(',').collect::<Vec<_>>()` |
| 大文字・小文字 | `s.toLowerCase()` / `s.lower()` | `s.to_lowercase()` |
| 置換 | `s.replaceAll(a, b)` / `s.replace(a, b)` | `s.replace(a, b)`（すべて置換） |
| 数値に変換 | `Number(s)` / `int(s)` | `s.parse::<i32>()` → `Result`（失敗が `NaN` や例外ではなく `Err`） |
| 数値を文字列に | `String(n)` / `str(n)` | `n.to_string()` |
| 先頭 n 文字 | `s.slice(0, n)` / `s[:n]` | `s.chars().take(n).collect::<String>()`（`&s[..n]` はバイト位置なので日本語で panic し得る） |

**関数の引数は `&str`、struct のフィールドや戻り値は `String`** が基本です（[2.4](02-ownership.md#24-string-と-str)）。

```rust
fn greet(name: &str) -> String {
    format!("こんにちは、{name}さん")
}

let owned = String::from("太郎");
greet(&owned); // String は & を付けると &str として渡せる
greet("花子"); // リテラルはもともと &str
```

## C.2 null / undefined を扱う（Option）

詳しくは [3.3](03-types-traits.md#33-optionnull-のない世界)。

| やりたいこと | TS | Rust（`x: Option<T>`） |
|---|---|---|
| 既定値 | `x ?? 0` | `x.unwrap_or(0)` |
| 既定値（計算が重い） | `x ?? compute()` | `x.unwrap_or_else(compute)` |
| 中身を変換 | `x?.name` | `x.map(\|v\| v.name)` |
| 変換の結果も Option | `x?.parent?.name` | `x.and_then(\|v\| v.parent).map(\|p\| p.name)` |
| あるときだけ処理 | `if (x) { ... }` | `if let Some(v) = x { ... }` |
| なければ早期 return | `if (!x) return;` | `let Some(v) = x else { return; };` |
| null なら Error に | `if (!x) throw ...` | `x.ok_or_else(\|\| AppError::NotFound(id))?` |
| 参照だけ借りる | — | `x.as_ref()`（`Option<String>` → `Option<&String>`。中身を move しない） |

```rust
fn first_word_len(s: &str) -> usize {
    let Some(word) = s.split_whitespace().next() else {
        return 0;
    };
    word.chars().count()
}
```

## C.3 配列・辞書

詳しくは [4.3](04-errors-collections.md#43-コレクション) と [4.5](04-errors-collections.md#45-イテレータ)。`v: Vec<T>`、`m: HashMap<K, V>` とします。

| やりたいこと | TS / Python | Rust |
|---|---|---|
| map / filter | `v.map(f).filter(g)` | `v.iter().map(f).filter(g).collect::<Vec<_>>()` |
| 合計 | `v.reduce((a, b) => a + b, 0)` / `sum(v)` | `v.iter().sum::<i32>()` |
| 畳み込み | `v.reduce(f, init)` | `v.iter().fold(init, f)` |
| 最初の一致 | `v.find(f)` | `v.iter().find(\|x\| ...)` → `Option<&T>` |
| 位置 | `v.findIndex(f)` | `v.iter().position(\|x\| ...)` → `Option<usize>` |
| some / every | `v.some(f)` / `v.every(f)` | `v.iter().any(f)` / `v.iter().all(f)` |
| 含む | `v.includes(x)` / `x in v` | `v.contains(&x)` |
| 添字付きループ | `v.forEach((x, i) => ...)` / `enumerate(v)` | `for (i, x) in v.iter().enumerate()` |
| 2つを組にする | `zip(a, b)` | `a.iter().zip(b.iter())` |
| 並べ替え | `v.sort((a, b) => a - b)` | `v.sort()`（`v` は `mut`。戻り値ではなくその場で並ぶ） |
| キーで並べ替え | `sorted(v, key=f)` | `v.sort_by_key(\|x\| x.age)` |
| 降順 | `sorted(v, reverse=True)` | `v.sort_by_key(\|x\| std::cmp::Reverse(x.age))` |
| 小数で並べ替え | `v.sort((a, b) => a - b)` | `v.sort_by(\|a, b\| a.total_cmp(b))`（`f64` は `sort()` できない） |
| 重複除去 | `[...new Set(v)]` | `v.sort(); v.dedup();`（順序を保つなら `HashSet` で既出チェック） |
| 文字列で連結 | `v.join(", ")` | `v.join(", ")`（`Vec<String>` / `Vec<&str>` のとき） |
| 先頭 n 件 | `v.slice(0, n)` / `v[:n]` | `v.iter().take(n)`、または `&v[..n.min(v.len())]` |
| 要素を消す | `v = v.filter(f)` | `v.retain(\|x\| ...)` |
| 辞書の取得 | `m[k] ?? 0` / `m.get(k, 0)` | `m.get(&k).copied().unwrap_or(0)` |
| 辞書で数える | `m[k] = (m[k] ?? 0) + 1` | `*m.entry(k).or_insert(0) += 1` |
| グループ化 | `Object.groupBy(v, f)` | `m.entry(key).or_insert_with(Vec::new).push(x)` |
| キー順に回す | `sorted(m.items())` | `BTreeMap` を使う（`HashMap` の順序は毎回変わる） |

```rust
use std::collections::HashMap;

let words = ["rust", "tauri", "rust"];
let mut count: HashMap<&str, usize> = HashMap::new();
for w in words {
    *count.entry(w).or_insert(0) += 1;
}
assert_eq!(count["rust"], 2);
```

`iter()` は要素を借りる（`&T`）、`into_iter()` は所有権ごと受け取る（元の `Vec` は使えなくなる）、`iter_mut()` は書き換え用です。迷ったら `iter()` から始めます。

## C.4 エラー処理

詳しくは [第4章](04-errors-collections.md) と [7.2](07-commands-state.md#72-エラー型errorrs)。

| やりたいこと | TS / Python | Rust |
|---|---|---|
| 失敗し得る関数 | `throw` する関数 | 戻り値を `Result<T, E>` にする |
| エラーを投げる | `throw new Error("...")` | `return Err(AppError::InvalidPath(path));` |
| そのまま呼び出し元へ | 何もしない（自動で伝播） | 呼び出しの後ろに `?` |
| 捕まえて分岐 | `try { } catch (e) { }` | `match result { Ok(v) => ..., Err(e) => ... }` |
| 特定のエラーだけ救う | `except FileNotFoundError:` | `Err(e) if e.kind() == ErrorKind::NotFound => ...`（`NoteStore::load` 参照） |
| 失敗したら既定値 | `try { } catch { return 0 }` | `s.parse().unwrap_or(0)` |
| 失敗は無視 | 空の `catch` | `let _ = window.hide();` |
| エラーの種類を変換 | `catch (e) { throw new MyError(e) }` | `#[from]` を付けて `?`、または `.map_err(\|e\| ...)?` |
| 絶対に起きないはず | `x!`（非 null アサーション） | `.expect("理由")`（起きたら panic） |

```rust
#[derive(Debug, thiserror::Error)]
enum ConfigError {
    #[error("ポート番号が不正です: {0}")]
    BadPort(String),
    #[error(transparent)]
    Io(#[from] std::io::Error), // これがあるので io::Error に ? が使える
}

fn read_port(path: &str) -> Result<u16, ConfigError> {
    let text = std::fs::read_to_string(path)?;
    let text = text.trim();
    text.parse()
        .map_err(|_| ConfigError::BadPort(text.to_string()))
}
```

`main` やテストで手早く `?` を使いたいときは、戻り値を `Result<(), Box<dyn std::error::Error>>` にすると、どのエラー型でも `?` で返せます。

## C.5 struct / JSON

詳しくは [第3章](03-types-traits.md) と [5.2](05-modules-serde-async.md#52-serdejson-変換)。

| やりたいこと | TS / Python | Rust |
|---|---|---|
| 一部だけ変えたコピー | `{ ...a, title: "x" }` | `Note { title: "x".into(), ..a.clone() }` |
| 既定値付きで作る | `{ ...defaults, ...opts }` | `#[derive(Default)]` → `Settings { width: 800, ..Default::default() }` |
| 比較できるように | `===`（参照比較） | `#[derive(PartialEq)]` で `==` が値比較になる |
| JSON 文字列に | `JSON.stringify(x)` | `serde_json::to_string(&x)?`（整形は `to_string_pretty`） |
| JSON から | `JSON.parse(s) as T` | `serde_json::from_str::<T>(&s)?`（形が違えば `Err`） |
| 型を決めずに読む | `JSON.parse(s)` の `any` | `serde_json::Value`。`v["user"]["name"].as_str()` |
| camelCase で出す | — | `#[serde(rename_all = "camelCase")]` |
| フィールド名を個別に | — | `#[serde(rename = "type")] kind: String` |
| 欠けていたら既定値 | `x.tags ?? []` | `#[serde(default)]` |
| `None` のとき省略 | `memo?: string` | `#[serde(skip_serializing_if = "Option::is_none")]` |
| TS の型を自動生成 | — | `ts-rs` / `tauri-specta`（[付録A](appendix-a-libraries.md#a3-rust--ts-の型共有)） |
| 判別共用体 | `{ kind: "a"; ... } \| { kind: "b"; ... }` | `#[serde(tag = "kind")]` の enum（フィールドの camelCase には `rename_all_fields` も） |

```rust
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    window_width: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_note_id: Option<u64>,
}

let json = serde_json::to_string(&Settings {
    window_width: 800,
    last_note_id: None,
})
.unwrap();
assert_eq!(json, r#"{"windowWidth":800}"#);
```

最後の `assert_eq!` のように **JSON の形をテストで固定する** と、TS 側の型とのずれに気づけます（[8.2](08-events-channels.md#82-channels進捗のストリーミング)）。

## C.6 ファイル・時間・非同期

| やりたいこと | TS（Node）/ Python | Rust |
|---|---|---|
| テキストを読む | `fs.readFileSync(p, "utf8")` / `open(p).read()` | `std::fs::read_to_string(p)?` |
| テキストを書く | `fs.writeFileSync(p, s)` | `std::fs::write(p, s)?` |
| フォルダを作る | `fs.mkdirSync(p, { recursive: true })` | `std::fs::create_dir_all(p)?` |
| 存在確認 | `fs.existsSync(p)` / `os.path.exists(p)` | `p.exists()`（`p: &Path`） |
| パスの結合 | `path.join(a, "b.json")` | `a.join("b.json")`（`a: PathBuf` / `&Path`） |
| 拡張子 | `path.extname(p)` | `p.extension()` → `Option<&OsStr>` |
| 現在時刻（ミリ秒） | `Date.now()` | `SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64`（`notes.rs` の `now_ms`） |
| 経過時間 | `performance.now()` の差 | `let t = Instant::now(); ...; t.elapsed()` |
| 待つ | `await sleep(300)` | `tokio::time::sleep(Duration::from_millis(300)).await` |
| 並列に待つ | `Promise.all([a(), b()])` | `tokio::join!(a(), b())` |
| 重い同期処理を逃がす | Worker / `run_in_executor` | `tauri::async_runtime::spawn_blocking(move \|\| ...).await` |
| スレッド間で共有 | — | `Arc<Mutex<T>>`（[5.3](05-modules-serde-async.md#53-スレッドと共有状態)） |
| 環境変数 | `process.env.X` / `os.environ["X"]` | `std::env::var("X")` → `Result` |

非同期まわりの注意点は [5.4](05-modules-serde-async.md#54-async--await) にまとめています。tokio のマクロ（`tokio::join!` など）を Tauri アプリで使うときは、`Cargo.toml` の `tokio` の `features` に `"macros"` を足します。

## C.7 IPC（command）

| やりたいこと | Electron | Tauri |
|---|---|---|
| フロントから Rust の関数を呼ぶ | `ipcMain.handle` + `ipcRenderer.invoke` | `#[tauri::command]` + `invoke`（[7.1](07-commands-state.md#71-command-の基本ルール)） |
| 型付きで呼ぶ | preload + `contextBridge` | `src/api.ts` のような薄いラッパー（[7.7](07-commands-state.md#77-フロント側apits)） |
| 失敗をフロントに伝える | `throw` → reject | `Result<T, AppError>` の `Err` → reject。`AppError` は `Serialize` を実装する（[7.2](07-commands-state.md#72-エラー型errorrs)） |
| 呼び出し元のウィンドウを知る | `event.sender` | 引数に `window: tauri::WebviewWindow` を足す（フロントからは渡さない） |
| 重い処理で UI を固めない | — | `async fn` の command にする。同期 `fn` はメインスレッドで動く |
| 新しい command を足す | `ipcMain.handle` を追加 | 関数を書き、**`generate_handler![]` に登録** |

```rust
#[tauri::command]
pub fn window_label(window: tauri::WebviewWindow) -> String {
    window.label().to_string()
}
```

```ts
const label = await invoke<string>("window_label"); // 引数は不要
```

## C.8 状態・データ保存

| やりたいこと | Electron | Tauri |
|---|---|---|
| アプリ全体の状態 | main プロセスのグローバル変数 | `app.manage(x)` + 引数 `State<'_, T>`。書き換えるなら中身を `Mutex` に（[7.3](07-commands-state.md#73-stateアプリ全体の状態)） |
| ユーザーデータの場所 | `app.getPath("userData")` | `app.path().app_data_dir()?`（`use tauri::Manager;` が必要） |
| 他の場所 | `app.getPath("documents")` など | `app.path().document_dir()?`、`download_dir()`、`app_config_dir()`、`app_log_dir()` |
| 設定の保存 | `electron-store` | `store` プラグイン（[第9章 やってみよう](09-plugins-capabilities.md#やってみよう)） |
| 自前の JSON に保存 | `fs.writeFileSync` | `NoteStore::save` の「一時ファイル → rename」（[7.4](07-commands-state.md#74-データ層notesrs)） |
| SQLite | `better-sqlite3` | `sql` プラグイン、または Rust 側で `rusqlite` / `sqlx`（[付録A](appendix-a-libraries.md)） |
| 秘密情報 | `safeStorage` | `keyring` クレート（[付録A](appendix-a-libraries.md)）。`stronghold` プラグインは非推奨（v3 で削除予定） |

## C.9 Rust → フロントへの通知

| やりたいこと | Electron | Tauri |
|---|---|---|
| 全ウィンドウに知らせる | `win.webContents.send` を全ウィンドウに | `app.emit("name", payload)?`（`use tauri::Emitter;`）（[8.1](08-events-channels.md#81-eventsrust--フロント)） |
| 特定のウィンドウだけ | `win.webContents.send` | `app.emit_to("main", "name", payload)?` |
| フロントで受け取る | `ipcRenderer.on` | `listen("name", cb)`。`useEffect` の cleanup で `unlisten` する |
| 進捗をストリームで返す | `send` を何度も呼ぶ | 引数に `Channel<T>` を取り `send`（[8.2](08-events-channels.md#82-channels進捗のストリーミング)） |
| メニューから React を動かす | `webContents.send` | `on_menu_event` で `emit_to` → フロントで `listen`（`lib.rs`） |

## C.10 ネイティブ機能

| やりたいこと | Electron | Tauri |
|---|---|---|
| 保存 / 開くダイアログ | `dialog.showSaveDialog` | `dialog` プラグインの `save()` / `open()`（[9.3](09-plugins-capabilities.md#93-メモの書き出し機能)） |
| 確認ダイアログ | `dialog.showMessageBox` | `dialog` プラグインの `ask()` / `confirm()` / `message()` |
| URL をブラウザで開く | `shell.openExternal` | `opener` プラグインの `openUrl(url)` |
| ファイルを書く | main で `fs` | command で `std::fs`（パスは Rust 側で検証）、または `fs` プラグイン + scope（[9.3](09-plugins-capabilities.md#fs-プラグインを使わなかった理由とその代償)） |
| フロントに API を使わせる | — | capability の `permissions` に追加（[9.1](09-plugins-capabilities.md#91-tauri-20-のセキュリティモデル)） |
| アプリメニュー | `Menu.setApplicationMenu` | `Menu::default` に `append` → `app.set_menu`（[10.1](10-menu-tray-windows.md#101-アプリケーションメニュー)） |
| ショートカット | `accelerator: "CmdOrCtrl+N"` | `MenuItem::with_id(..., Some("CmdOrCtrl+N"))` |
| 非アクティブでも効くショートカット | `globalShortcut` | `global-shortcut` プラグイン |
| トレイ | `new Tray` | `TrayIconBuilder`（`tray-icon` feature）（[10.2](10-menu-tray-windows.md#102-システムトレイ)） |
| OS の通知 | `new Notification` | `notification` プラグイン |
| 多重起動の防止 | `app.requestSingleInstanceLock` | `single-instance` プラグイン |
| クリップボード | `clipboard` | `clipboard-manager` プラグイン（テキストの手入力なら WebView 標準の ⌘C / Ctrl+C で足りる） |

プラグインはどれも `pnpm tauri add <名前>` で入ります（[9.2](09-plugins-capabilities.md#92-プラグインの追加)）。

## C.11 ウィンドウ・アプリのライフサイクル

| やりたいこと | Electron | Tauri |
|---|---|---|
| 初期ウィンドウの大きさ | `new BrowserWindow({ width })` | `tauri.conf.json` の `app.windows`（[10.4](10-menu-tray-windows.md#104-ウィンドウ)） |
| ウィンドウを追加で開く | `new BrowserWindow` | `async` command で `WebviewWindowBuilder`（[10.4](10-menu-tray-windows.md#動的に開くrust)） |
| ウィンドウを取得 | `BrowserWindow.fromId` | `app.get_webview_window("main")`（`use tauri::Manager;`） |
| フロントで自分の label | — | `getCurrentWebviewWindow().label` |
| 閉じるボタンで隠す | `win.on("close", e => e.preventDefault())` | `on_window_event` で `CloseRequested` → `api.prevent_close()`（[10.3](10-menu-tray-windows.md#103-閉じるボタンでトレイに格納するやってみよう)） |
| 表示して前面に | `win.show(); win.focus()` | `w.show()` + `w.set_focus()` |
| 位置・サイズを記憶 | `electron-window-state` | `window-state` プラグイン |
| アプリを終了 | `app.quit()` | `app.exit(0)` |
| 起動時の初期化 | `app.whenReady()` | `Builder::setup(\|app\| { ... })` |
| dev ビルドかどうか | `!app.isPackaged` | Rust：`cfg!(debug_assertions)`、フロント：`import.meta.env.DEV` |
| DevTools を開く | `webContents.openDevTools()` | dev ビルドでは右クリック →「要素の詳細を表示」（Windows は「検査」）、または ⌘⌥I（Windows は Ctrl+Shift+I） |
| ログ | `electron-log` | `log` プラグイン（[付録A](appendix-a-libraries.md)） |
| invoke をモックしてテスト | — | `@tauri-apps/api/mocks` の `mockIPC`（[11.5](11-build-release.md#115-テスト)） |

## C.12 エラーメッセージから引く

### Rust のコンパイルエラー

| メッセージ | よくある原因 | 対処 |
|---|---|---|
| `borrow of moved value` (E0382) | 値を関数に渡した（move した）後で使った | 渡す側を `&x` にする。後でも使うなら `x.clone()`（[2.2](02-ownership.md#22-所有権の3ルール)） |
| `cannot borrow ... as mutable because it is also borrowed as immutable` (E0502) | `v.iter()` で回しながら `v.push()` した、など | 先に必要な値を集めてから、ループの外で変更する |
| `mismatched types: expected String, found &str` (E0308) | `"abc"` を `String` の場所に渡した | `"abc".to_string()` / `.into()` |
| `expected &str, found String` (E0308) | `String` を `&str` の引数に渡した | `&s` を渡す |
| `the ? operator can only be used in a function that returns Result` (E0277) | `main` や戻り値 `()` の関数で `?` を使った | 戻り値を `Result<..., ...>` にする（C.4） |
| `` `?` couldn't convert the error to `AppError` `` (E0277) | `AppError` にそのエラー型の `#[from]` がない | `AppError` にバリアントを足す、または `.map_err(...)` |
| `cannot assign twice to immutable variable` (E0384) | `let` で宣言した変数を書き換えた | `let mut` にする |
| `cannot borrow ... as mutable, as it is not declared as mutable` (E0596) | `let` で宣言した `Vec` に `push` した、など | `let mut` にする |
| `closure may outlive the current function, but it borrows ...` (E0373) | `thread::spawn` などに渡すクロージャが外の変数を借りている | クロージャの前に `move` を付ける（[5.3](05-modules-serde-async.md#53-スレッドと共有状態)） |
| `` no method named `xxx` found `` (E0599) | trait を `use` していない | `use tauri::Manager;`、`use tauri::Emitter;` など。エラー文の help に `use` 候補が出る |
| `` `Foo` doesn't implement `Debug` `` (E0277) | `{:?}` で表示しようとした | `#[derive(Debug)]` を付ける |
| `missing lifetime specifier` (E0106) | struct のフィールドや戻り値に `&str` を使った | `String` にする（[2.5](02-ownership.md#25-ライフタイム)） |
| `future cannot be sent between threads safely` | `std::sync::Mutex` のロックを持ったまま `.await` した | ロックをブロック `{ }` に閉じ込めて、`.await` の前に解放する（[5.4](05-modules-serde-async.md#54-async--await)） |
| ``async commands that contain references as inputs must return a `Result` `` | `async fn` の command で `State<'_, T>` や `&str` を受け取り、戻り値が `Result` でない | 戻り値を `AppResult<T>` にする（`analyze_notes` 参照）。`&str` は `String` にしてもよい（[7.1](07-commands-state.md#71-command-の基本ルール)） |

### Tauri の実行時エラー

| 症状 | 原因と対処 |
|---|---|
| `command xxx not found` | `generate_handler!` への登録漏れ、または綴りミス（[7章 よくあるエラー](07-commands-state.md#よくあるエラー)） |
| `missing required key xxx` | 引数名の不一致。Rust の `note_id` はフロントでは `noteId`（[7章 よくあるエラー](07-commands-state.md#よくあるエラー)） |
| `xxx not allowed. Permissions associated with this command: ...` | capability に権限がない。エラー文に出た権限名を `capabilities/*.json` に足す。詳しい文面は dev ビルドだけで、本番ビルドでは `Command xxx not allowed by ACL` になる（[9.1](09-plugins-capabilities.md#91-tauri-20-のセキュリティモデル)） |
| invoke が `state not managed for field ...` で reject | `app.manage(...)` が実行されていない。Rust 側の `app.state::<T>()` なら `state() called before manage()` で panic する（[7章 よくあるエラー](07-commands-state.md#よくあるエラー)） |
| TS では型が合っているのに値が `undefined` | Rust ⇔ TS の JSON の形のずれ。`serde_json::to_string` の結果をテストで確認（C.5） |
| ⌘C / ⌘V が効かなくなった（macOS） | メニューを `Menu::new` で作った。`Menu::default` をベースにする（[10.1](10-menu-tray-windows.md#101-アプリケーションメニュー)） |
| Console に IPC のフォールバックの警告 | CSP に `connect-src ipc: http://ipc.localhost` がない（[9.4](09-plugins-capabilities.md#94-フロントエンドの-csp)） |
| Windows でウィンドウを開く command が固まる | 同期 command でウィンドウを作った。`async fn` にする（[10.4](10-menu-tray-windows.md#動的に開くrust)） |
