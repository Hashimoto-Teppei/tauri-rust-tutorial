# 第4章 エラー処理とコレクション

## 4.1 例外はない。Result を返す

```rust
enum Result<T, E> { Ok(T), Err(E) }   // 概念上の定義
```

```rust
let n: Result<i32, _> = "42".parse::<i32>();
match n {
    Ok(v) => println!("{v}"),
    Err(e) => println!("error: {e}"),
}
```

TS の `throw` と違い、**失敗し得ることが型に現れる** ので、呼び出し側は無視できません（無視すると警告）。

### `?` 演算子：エラーなら即 return

```rust
fn read_number(path: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)?;   // Err なら関数から return Err(e)
    let n = text.trim().parse::<i32>()?;
    Ok(n)
}
```

`?` は、async 関数内で `await` した Promise が reject されると例外が呼び出し元へ伝播する挙動を、明示的かつ型安全にしたもの、と考えると近いです。
Option に対しても使えます（None なら return None）。

### panic

`panic!()`, `unwrap()`, `expect("msg")`, 配列の範囲外アクセスはプログラムを停止させます。
**回復可能なエラーは Result、バグ（あり得ない状態）は panic** が使い分けの原則です。

## 4.2 独自エラー型：thiserror

アプリ内のエラーは enum で表すのが定石です。`thiserror` クレートで `Display` 実装を自動生成します。

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("ファイルが見つかりません: {0}")]
    NotFound(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),     // #[from]: io::Error から ? で自動変換
}

fn load(path: &str) -> Result<String, AppError> {
    let s = std::fs::read_to_string(path)?;   // io::Error → AppError::Io に自動変換
    Ok(s)
}
```

> **Tauri の command はこのパターンをそのまま使います**（第7章）。

`anyhow` クレートは「エラー型を細かく定義しない」アプリ向けの選択肢です。`anyhow::Result<T>` と書けば何でも `?` で投げられます。

## 4.3 コレクション

| TS / Python | Rust |
|---|---|
| `Array` / `list` | `Vec<T>` |
| `Map` / `dict` | `HashMap<K, V>`（順序なし）, `BTreeMap<K, V>`（キー順） |
| `Set` / `set` | `HashSet<T>` |

```rust
use std::collections::HashMap;

let mut v = vec![1, 2, 3];
v.push(4);
let first = v.get(0);         // Option<&i32>（範囲外でも panic しない）
let first = v[0];             // 範囲外なら panic

let mut m = HashMap::new();
m.insert("a".to_string(), 1);
*m.entry("a".to_string()).or_insert(0) += 1;   // カウントの定番イディオム
if let Some(v) = m.get("a") { ... }
```

## 4.4 クロージャ

```rust
let factor = 2;
let double = |x: i32| x * factor;          // 周りの変数を「借用」してキャプチャ
let mut count = 0;
let mut inc = || count += 1;               // 書き換えるなら可変借用（クロージャ自体も mut）
let name = String::from("a");
let owned = move || println!("{name}");    // move: 所有権ごとクロージャに移す
```

TS のアロー関数と違い、**キャプチャの仕方（借用か所有か）が型に影響** します。

| trait | 意味 | 典型的な受け手 |
|---|---|---|
| `Fn` | キャプチャを読むだけ。何度でも呼べる | `map`, `filter` |
| `FnMut` | キャプチャを書き換える。何度でも呼べる | `for_each`, ソートの比較関数 |
| `FnOnce` | キャプチャを消費する。1回だけ | `thread::spawn`, `Option::unwrap_or_else` |

別スレッドや非同期タスクに渡すクロージャは、元の関数より長生きし得るので `move` が必須です（`std::thread::spawn(move || ...)`, `tokio::spawn(async move { ... })`）。Tauri の `.setup(|app| ...)` や `.on_menu_event(|app, event| ...)` もクロージャです。

## 4.5 イテレータ

TS の配列メソッドとほぼ1対1で対応しますが、**遅延評価** で `collect()` 等を呼ぶまで実行されません。

| TS | Rust |
|---|---|
| `arr.map(f)` | `iter().map(f)` |
| `arr.filter(f)` | `iter().filter(f)` |
| `arr.find(f)` | `iter().find(f)` → `Option` |
| `arr.some(f)` / `every(f)` | `any(f)` / `all(f)` |
| `arr.reduce(f, init)` | `fold(init, f)` |
| `arr.forEach(f)` | `for x in arr` が推奨 |
| `arr.entries()` | `iter().enumerate()` |
| `arr.slice(0, n)` | `iter().take(n)` |
| `Object.fromEntries` | `collect::<HashMap<_, _>>()` |

3種類の取り出し方：

```rust
v.iter()        // &T を順に借用
v.iter_mut()    // &mut T
v.into_iter()   // T（v の所有権を消費）
```

### collect の便利技

`Iterator<Item = Result<T, E>>` は `Result<Vec<T>, E>` に collect できます（最初の Err で止まる）。

```rust
let nums: Result<Vec<i32>, _> = ["1", "2", "x"].iter().map(|s| s.parse::<i32>()).collect();
// → Err(...)
```

## 演習

```sh
cargo test -p practice ch04
```

- `parse_kv` … `split_once('=')` と `ok_or_else` と `?`
- `parse_config` … 上の「Result への collect」を使うと1式で書けます
- `long_words` … `HashSet::insert` が「新規なら true」を返すことを利用（`filter` に渡すクロージャが `seen` を書き換えるので FnMut）。文字数は `chars().count()` で
