# Rust × Tauri 2.0 チュートリアル

TypeScript / React / Python の経験があり、Electron Forge でデスクトップアプリを作ったことがある人向けの学習コースです。
「知っている概念との対応づけ」を軸に、Rust の文法で詰まりやすい箇所と Tauri の IPC・セキュリティモデルに時間を集中させています。

## ゴール

最終的に **Markdown メモアプリ（Tauri Notes）** を React + TypeScript + Rust で作り、macOS / Windows 向けにビルドできるようになること。

## コース構成

| # | 章 | 内容 | 目安 |
|---|----|------|------|
| 0 | [環境構築](docs/00-setup.md) | mise によるバージョン固定, OS の前提ツール, エディタ | 30分〜1h |
| **Part 1: Rust** | | | |
| 1 | [Rust の基本](docs/01-rust-basics.md) | cargo, 変数, 型, 関数, 制御フロー（TS/Python 対応表） | 1.5h |
| 2 | [所有権と借用](docs/02-ownership.md) | **最重要**。GC がない世界のメモリ管理 | 2h |
| 3 | [struct / enum / trait](docs/03-types-traits.md) | interface・union 型・クラスとの対応 | 2h |
| 4 | [エラー処理とコレクション](docs/04-errors-collections.md) | Result, `?`, thiserror, Vec/HashMap, クロージャ, イテレータ | 2h |
| 5 | [モジュール・serde・並行処理](docs/05-modules-serde-async.md) | Tauri で必ず使う 3 点セット | 2h |
| **Part 2: Tauri 2.0** | | | |
| 6 | [Tauri 概観と Electron との違い](docs/06-tauri-overview.md) | プロセスモデル対応表、プロジェクト作成 | 1h |
| 7 | [command と State](docs/07-commands-state.md) | `invoke` / エラー / 状態管理 → メモ CRUD | 3h |
| 8 | [Events と Channels](docs/08-events-channels.md) | Rust → フロントへの通知・ストリーム | 1h |
| 9 | [プラグインと Capabilities](docs/09-plugins-capabilities.md) | dialog / fs / 権限モデル、ファイル保存 | 1.5h |
| 10 | [メニュー・トレイ・ウィンドウ](docs/10-menu-tray-windows.md) | ネイティブ UI 統合 | 1h |
| 11 | [ビルドと配布](docs/11-build-release.md) | バンドル、署名、自動更新、テスト | 1h |
| **付録** | | | |
| A | [便利なライブラリ](docs/appendix-a-libraries.md) | 公式プラグイン、Rust クレート、型共有、DevTools、CI | 必要に応じて |
| B | [AI エージェントでの開発](docs/appendix-b-ai-agents.md) | Context7、llms.txt、rust-analyzer LSP、Tauri MCP、エージェントスキル | 必要に応じて |
| C | [逆引きレシピ](docs/appendix-c-recipes.md) | 「TS / Python / Electron ならこう書く」から Rust・Tauri の書き方を引く。エラーメッセージ別の対処 | 必要に応じて |

## 進め方

1. 各章を読み、コード例を手元で動かす
2. Part 1 の各章末にある演習を `exercises/practice/` で解く（テストが通ればクリア）
3. Part 2 は `notes-app/`（第6章で作成）を章ごとに育てていく。完成形は `reference/notes-app/`（詰まったら参照）

```sh
mise trust && mise install   # 初回のみ（第0章）
cd exercises
cargo test -p practice ch02          # 第2章の演習だけ実行
cargo test -p practice ch02::tests::test_longest   # 単一テスト
cargo test -p solutions              # 模範解答（全テスト通過済み）
```

## 対応環境

- **OS**：macOS / Windows（Linux もビルドとテストは CI で確認済み）。OS ごとの準備は [第0章](docs/00-setup.md) を参照
- **ランタイム**：`mise.toml` で Node / pnpm / Rust のバージョンを固定
- **エディタ**：VS Code / Cursor / Windsurf では、推奨拡張（`.vscode/extensions.json`）のインストールを促されます
- **AI エージェント**：作業ガイドは `AGENTS.md` にあり、Claude Code（`CLAUDE.md` 経由）、Codex、Cursor、GitHub Copilot、Gemini CLI（`.gemini/settings.json` 経由）が読み込みます。詳しくは [付録B](docs/appendix-b-ai-agents.md)

[![CI](https://github.com/Hashimoto-Teppei/tauri-rust-tutorial/actions/workflows/ci.yml/badge.svg)](https://github.com/Hashimoto-Teppei/tauri-rust-tutorial/actions/workflows/ci.yml)

## 学習のコツ

- **コンパイラのエラーメッセージを全文読む。** Rust のエラーは「どう直せばいいか」まで書いてあることが多い
- 所有権で詰まったら、まず `.clone()` で動かしてから後で最適化してよい
- 公式リソース: [The Rust Book（日本語版）](https://doc.rust-jp.rs/book-ja/) / [Rust by Example](https://doc.rust-jp.rs/rust-by-example-ja/) / [Tauri v2 Docs](https://v2.tauri.app/)

## ライセンス

- `docs/` と各 README の文章：[CC BY 4.0](LICENSE-docs)（出典を表示すれば、改変・再配布・商用利用が可能）
- `exercises/`・`reference/` などのコード：[MIT](LICENSE)
