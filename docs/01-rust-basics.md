# 第1章 Rust の基本

## 1.1 最初のプロジェクト

```sh
cargo new hello && cd hello && cargo run
```

```rust
// src/main.rs
fn main() {
    println!("Hello, world!"); // `!` 付きはマクロ（関数ではない）
}
```

> この章のコードには `&`・`unwrap()`・`&'static str` が出てきますが、今は次の理解で十分です。
> - `&x` … x を「借りて読む」（第2章）
> - `.unwrap()` … 失敗したらクラッシュさせて中身を取り出す（第3章）
> - `&'static str` … 文字列リテラルの型（第2章）
> - 演習の `todo!()` … 「未実装」を表すマクロ。消して実装を書く

`cargo new` はバイナリ（実行ファイル）、`cargo new --lib` はライブラリを作ります。Tauri アプリの Rust 側は「ライブラリ + 薄い main.rs」構成です。

## 1.2 変数：デフォルトで不変

```rust
let x = 5;          // TS の const 相当。再代入不可
let mut y = 5;      // TS の let 相当
y += 1;

// 不変は「中身まで」: let v = vec![1]; v.push(2); はエラー（TS の const 配列は push できる）
let x = x * 2;      // シャドーイング: 同名で新しい変数を作る（型を変えてもよい）
let s = "42";
let s: i32 = s.parse().unwrap();
```

| TS / Python | Rust |
|---|---|
| `const x = 5` | `let x = 5;` |
| `let x = 5` | `let mut x = 5;` |
| `number` | `i32`, `i64`, `u32`, `usize`, `f64` …（サイズと符号を明示） |
| `string` | `String`（所有）と `&str`（借用）→ 第2章 |
| `boolean` | `bool` |
| `[number, string]` | `(i32, String)` タプル |
| `number[]` | `Vec<i32>`（可変長）, `[i32; 3]`（固定長） |
| `null` / `undefined` / `None` | **存在しない**。`Option<T>` を使う → 第3章 |

> `usize` は配列インデックスや長さに使う型。`v.len()` は `usize` を返すので、`i32` と混ぜると型エラーになります（暗黙の数値変換はありません。`as` で明示変換）。

## 1.3 関数

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b   // セミコロンなしの最後の式が戻り値
}
```

**「式」と「文」の区別** が重要です。`if` も `match` も `{}` ブロックも値を返す式です。

```rust
let label = if n > 0 { "positive" } else { "non-positive" }; // 三項演算子の代わり
let v = {
    let a = 1;
    a + 1          // ブロックの値は 2
};
```

## 1.4 制御フロー

```rust
for i in 0..5 { }          // 0〜4  (Python の range(5))
for i in 0..=5 { }         // 0〜5
for x in &vec { }          // 要素を借用して回す
while cond { }
loop { break 42; }         // 無限ループ。break で値を返せる
```

### match：強化版 switch

```rust
match score {
    90..=100 => "A",
    70..=89  => "B",
    50..=69  => "C",
    0..=49   => "D",
    _        => "invalid",   // 網羅性チェック: すべてのケースを扱わないとコンパイルエラー
}
```

TS の `switch` と違い、**フォールスルーなし・網羅必須・値を返す**。第3章の enum と組み合わせると真価を発揮します。

## 1.5 文字列フォーマット

```rust
let name = "Rust";
println!("Hello, {name}!");        // テンプレートリテラル相当
println!("{:?}", vec![1, 2]);      // Debug 表示（console.log 的）
println!("{:#?}", some_struct);    // 整形された Debug 表示
let s = format!("{}-{}", a, b);    // 文字列を作るだけ
```

`dbg!(expr)` は値と行番号を stderr に出して値をそのまま返すので、デバッグに便利です。

## 1.6 クロージャとイテレータ（さわりだけ）

```rust
let doubled: Vec<i32> = nums.iter().map(|x| x * 2).collect();
// TS: nums.map(x => x * 2)
let evens: Vec<&i32> = nums.iter().filter(|x| *x % 2 == 0).collect();
```

`collect()` で結果の型を決める必要があるため、型注釈（`Vec<i32>` など）を付けるのが定番です。

## 演習

`exercises/practice/src/ch01.rs` を実装して以下を通してください。

```sh
cargo test -p practice ch01
```

- `fizzbuzz` … `match` にタプル `(i % 3, i % 5)` を渡すときれいに書けます
- `sum_of_evens` … `for` 版とイテレータ版（`.iter().filter().sum()`）の両方を試してみましょう
- `grade` … 範囲パターン `90..=100`
