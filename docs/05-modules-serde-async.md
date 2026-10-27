# 第5章 モジュール・serde・並行処理

Tauri の Rust コードを読み書きするのに必要な最後のピースです。

## 5.1 モジュールとクレート

```
src/
├── lib.rs        // クレートのルート
├── db.rs         // mod db;
└── commands/
    ├── mod.rs    // mod commands;（または src/commands.rs）
    └── notes.rs  // commands/mod.rs 内で pub mod notes;
```

```rust
// lib.rs
mod db;                    // db.rs を読み込む（TS の import とは逆に「宣言」が必要）
pub mod commands;

use crate::db::Database;   // crate:: = 自クレートのルートから
use std::collections::HashMap;

// commands/notes.rs 内では:
use super::*;              // super:: = 親モジュール（ここでは commands）
```

| TS | Rust |
|---|---|
| ファイル = モジュール（自動） | `mod xxx;` で明示的に宣言 |
| `export` | `pub` |
| `import { A } from "./a"` | `use crate::a::A;` |
| npm パッケージ | クレート（`cargo add xxx`） |

## 5.2 serde：JSON 変換

Tauri のフロント ⇔ Rust 間のデータは JSON でシリアライズされます。その変換を担うのが serde です。

```sh
cargo add serde --features derive
cargo add serde_json
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]   // Rust の snake_case ⇔ TS の camelCase
pub struct Note {
    pub id: u32,
    pub is_pinned: bool,             // JSON では "isPinned"
    #[serde(default)]                // 欠けていたら Default 値
    pub tags: Vec<String>,
    pub memo: Option<String>,        // 読み込み: null / 欠落 → None、書き出し: None → null
}

let json = serde_json::to_string(&note)?;
let note: Note = serde_json::from_str(&json)?;
```

TS 側の型は `memo: string | null` です（`memo?: string` ではない）。キーごと省略したいなら `#[serde(skip_serializing_if = "Option::is_none")]` を付けます。

> デシリアライズする struct や command の引数・戻り値には `&str` ではなく **`String` などの所有型** を使います。借用だとライフタイムが絡んでコンパイルが通りにくくなります。

**Zod のようなバリデーションが型定義だけで自動で効く** のが強力です。JSON の形が違えばデシリアライズ時にエラーになります。

enum も変換できます：

```rust
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum Event { Saved { id: u32 }, Deleted { id: u32 } }
// → {"kind":"saved","id":1}  TS の判別共用体そのもの
```

## 5.3 スレッドと共有状態

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));     // Arc: スレッド間で共有できる参照カウント
let handles: Vec<_> = (0..4)
    .map(|_| {
        let c = Arc::clone(&counter);      // スレッドごとに参照を複製
        std::thread::spawn(move || {       // move: クロージャに所有権を移す
            *c.lock().unwrap() += 1;       // lock() で排他的にアクセス
        })
    })
    .collect();
for h in handles {
    h.join().unwrap();                     // 全スレッドの終了を待つ（await 相当）
}
println!("{}", *counter.lock().unwrap());  // 4
```

| 型 | 役割 |
|---|---|
| `Rc<T>` | 単一スレッド用の共有所有権 |
| `Arc<T>` | スレッド安全な共有所有権 |
| `Mutex<T>` / `RwLock<T>` | 中身を排他的に書き換える |

Rust は「`Send` / `Sync` でない型を他スレッドに送る」コードをコンパイルさせません。JS のシングルスレッドの気楽さはないかわりに、データ競合はコンパイル時に排除されます。

> **Tauri の State は内部で Arc 管理されるので、自分で書くのは `Mutex<T>` だけ** です（第7章）。

## 5.4 async / await

JS と違い、Rust 自体は非同期ランタイムを持ちません。`tokio` が事実上の標準で、**Tauri も内部で tokio を使っています**。

```rust
async fn fetch(id: u32) -> u32 { ... }

#[tokio::main]
async fn main() {
    let a = fetch(1).await;                                 // 逐次
    let (a, b) = tokio::join!(fetch(1), fetch(2));          // Promise.all 相当
    let handle = tokio::spawn(async { fetch(3).await });    // バックグラウンドタスク
    let c = handle.await.unwrap();
}
```

JS との違い：

- **Future は await されるまで何もしない**（Promise は作った瞬間に走り出す）
- `std::thread::sleep` など同期的にブロックする処理を async 内で呼ぶとランタイムが詰まる。`tokio::time::sleep` や `tokio::task::spawn_blocking` を使う
- `std::sync::Mutex` のロックを保持したまま `.await` しない（必要なら `tokio::sync::Mutex`）

## 演習

```sh
cargo test -p practice ch05
```

- `Note` には serde 属性が付いていません。テストが期待する JSON（`isPinned`、`tags` 省略可）から必要な `#[serde(...)]` を考えて付けよう
- `fetch_three` は 250ms 以内に終わらないとテストが落ちます（逐次実行では 300ms）

Part 1 はここまで。お疲れさまでした。次章から Tauri です。
