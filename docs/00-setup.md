# 第0章 環境構築

このリポジトリは [mise](https://mise.jdx.dev/) で Node / pnpm / Rust のバージョンを `mise.toml` に固定しています（Node の `.nvmrc` + corepack を Rust まで広げたもの、と考えてください）。Rust 本体は mise が内部で rustup を使って入れるので、rustup を手動で入れる必要はありません。

## 1. OS の前提ツール（mise では入らない）

- **macOS**：`xcode-select --install`（Xcode Command Line Tools）
- **Windows**：[Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) で「C++ によるデスクトップ開発」を選択（リンカとして Part 1 の `cargo test` でも必須）。WebView2 は Windows 10/11 に標準搭載
- **Linux**：https://v2.tauri.app/start/prerequisites/#linux の webkit2gtk などを入れる（本チュートリアルの動作確認は macOS のみ）

## 2. mise のインストールと有効化

```sh
# macOS
brew install mise
echo 'eval "$(mise activate zsh)"' >> ~/.zshrc && exec zsh
```

```powershell
# Windows (PowerShell)
winget install jdx.mise
Add-Content $PROFILE '(&mise activate pwsh) | Out-String | Invoke-Expression'
# PowerShell を開き直す
```

## 3. ツールチェーンのインストール

```sh
cd tauri-rust-tutorial
mise trust          # このリポジトリの mise.toml を信頼する（初回のみ）
mise install        # node / pnpm / rust（+ rustfmt, clippy, rust-analyzer）を入れる
mise ls --current   # mise.toml のバージョンになっていれば OK
```

> 既に rustup や Node を入れている場合も共存できます。このディレクトリ配下では `mise.toml` のバージョンが優先されます。
> `pnpm tauri ...` などが「cargo が見つからない」と言う場合は、そのシェルで mise が有効になっていません。`mise exec -- <コマンド>` で実行するか、手順2を確認してください。

## Rust のツール対応表

| Rust | Node / Python での相当 |
|---|---|
| `mise`（内部で rustup） | nvm / pyenv |
| `cargo` | npm + tsc + jest をまとめたもの |
| `Cargo.toml` | package.json / pyproject.toml |
| `Cargo.lock` | package-lock.json / pnpm-lock.yaml |
| crates.io | npm registry / PyPI |
| `cargo add serde` | `pnpm add serde` |
| `cargo run` / `cargo build` / `cargo test` | `pnpm start` / `pnpm build` / `pnpm test` |
| `cargo fmt` / `cargo clippy` | prettier / eslint |

## エディタ

VS Code なら **rust-analyzer** 拡張は必須です（型推論結果がインレイ表示され、学習効率が段違い）。
Tauri 用に **Tauri** 拡張も入れておくと `tauri.conf.json` の補完が効きます。

## 動作確認

```sh
cd exercises
cargo test -p solutions   # 全テストが ok になれば準備完了（初回は依存のコンパイルで数分かかる）
```
