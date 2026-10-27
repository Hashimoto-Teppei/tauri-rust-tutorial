# 第3章 struct / enum / trait

## 3.1 struct：データとメソッド

Rust にクラスはありません。**データ（struct）** と **振る舞い（impl）** を分けて書きます。

```rust
#[derive(Debug, Clone, PartialEq)]   // よく使う機能を自動実装
pub struct User {
    pub name: String,
    pub age: u32,
}

impl User {
    // 関連関数（self なし）= static メソッド。コンストラクタは慣習的に new
    pub fn new(name: &str, age: u32) -> Self {
        Self { name: name.to_string(), age }
    }

    pub fn greet(&self) -> String {          // &self: 読み取り
        format!("I'm {}", self.name)
    }

    pub fn birthday(&mut self) {             // &mut self: 書き換え
        self.age += 1;
    }
}

let mut u = User::new("Alice", 30);
u.birthday();
```

| TS | Rust |
|---|---|
| `class User { constructor(...) }` | `struct User` + `impl User { fn new(...) }` |
| `this` | `self`（明示的に引数に書く） |
| `static create()` | `fn create() -> Self`（self なし） |
| `private` | デフォルトで非公開。公開するものに `pub` |
| 継承 `extends` | **なし**。trait とコンポジションで代用 |

### よく使う derive

| derive | できるようになること |
|---|---|
| `Debug` | `{:?}` で表示 |
| `Clone` | `.clone()` |
| `PartialEq` | `==` で比較 |
| `Default` | `User::default()` で全フィールドのデフォルト値 |
| `Serialize, Deserialize` | JSON 変換（serde、第5章）。**Tauri で必須** |

## 3.2 enum：TS の判別共用体（discriminated union）

Rust の enum は各バリアントがデータを持てます。TS の判別共用体と思えばほぼ正解です。

```ts
// TS
type Shape =
  | { kind: "circle"; r: number }
  | { kind: "rect"; w: number; h: number };
```

```rust
// Rust
enum Shape {
    Circle { r: f64 },           // 構造体風バリアント
    Rect { w: f64, h: f64 },
    Square(f64),                 // タプル風バリアント（名前なしフィールド）
    Point,                       // データなしも可
}

fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle { r } => std::f64::consts::PI * r * r,
        Shape::Rect { w, h } => w * h,
        Shape::Square(size) => size * size,
        Shape::Point => 0.0,
    }   // バリアントを追加したら、ここがコンパイルエラーになって教えてくれる
}
```

## 3.3 Option：null のない世界

```rust
enum Option<T> { Some(T), None }    // 標準ライブラリの定義（概念上）
```

```rust
fn find(id: u32) -> Option<User> { ... }

match find(1) {
    Some(user) => println!("{}", user.name),
    None => println!("not found"),
}

if let Some(user) = find(1) { ... }          // 1ケースだけ扱う
let name = find(1).map(|u| u.name).unwrap_or("guest".into()); // u?.name ?? "guest"
let Some(user) = find(1) else { return; };   // let-else: 早期 return
```

| TS | Rust |
|---|---|
| `T \| undefined` | `Option<T>` |
| `x?.foo` | `x.map(\|v\| v.foo)` / `x.as_ref().map(...)` |
| `x ?? d` | `x.unwrap_or(d)` / `unwrap_or_else(\|\| ...)` |
| `x!` | `x.unwrap()`（TS の `!` は実行時チェックなしの型注釈だが、unwrap は実行時に検査し None なら panic = クラッシュ） |

> `unwrap()` は「絶対 None ではない」と言い切れる時と、テスト・試作時だけにしましょう。

## 3.4 trait：interface に近いもの

```rust
pub trait Describe {
    fn name(&self) -> String;                 // 実装必須
    fn describe(&self) -> String {            // デフォルト実装
        format!("これは{}です", self.name())
    }
}

impl Describe for Shape {
    fn name(&self) -> String { ... }
}
```

TS の interface との違い：

- **後付けできる**：自分で定義した trait を、他人の型（`String` など）に実装できる。ただし「外部の trait を外部の型に」（例: `impl Display for Vec<T>`）は実装できない（orphan rule）
- **構造的型付けではない**：同じメソッドを持っていても `impl` しない限りその trait の型ではない

標準ライブラリの trait（`Display`, `From`, `Iterator`, `Default` …）を実装すると、言語機能と統合されます。例：`impl Display for User` すると `println!("{user}")` が使えるようになる。

## 3.5 ジェネリクスと trait 境界

```rust
fn describe_all<T: Describe>(items: &[T]) -> Vec<String> {
    items.iter().map(|i| i.describe()).collect()
}
// 同じ意味: fn describe_all(items: &[impl Describe]) -> ...
```

TS の `<T extends Describe>` と同じ発想です。コンパイル時に型ごとに展開される（単相化）ので実行時コストはゼロです。

異なる型を1つの Vec に混ぜたい場合は `Vec<Box<dyn Describe>>`（動的ディスパッチ）を使います。

## 演習

```sh
cargo test -p practice ch03
```

- `largest` は `iter().max_by(|a, b| a.area().total_cmp(&b.area()))` の形で。`f64` は `NaN` があるため `Ord` を実装しておらず、`<` 以外での並べ替えには `total_cmp` を使います
- `Shape::Rect(Rect)` はタプル風バリアントなので `Shape::Rect(rect) => rect.area()` とマッチします
- `describe_all` はジェネリクスの練習
