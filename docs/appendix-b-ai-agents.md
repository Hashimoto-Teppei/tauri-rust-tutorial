# 付録B AI エージェントで Tauri アプリを開発する

> 2026年10月時点の調査です。Claude Code を例にしていますが、多くは Cursor・Codex CLI などでも使えます。

Tauri × Rust は、AI エージェントが **間違えやすい組み合わせ** です。

- Tauri は v1 と v2 で API が大きく違い、古い v1 の情報が学習データに多く残っている
- Rust の所有権エラーは、その場しのぎの `.clone()` や `unwrap()` で「直った」ことにされがち
- Rust ⇔ TS の境界のずれ（第8章の `rename_all_fields` など）は、両方の型チェックをすり抜ける

そのため、エージェントに **①正しい資料を与え、②結果を自分で確かめさせる** 仕組みを用意するのが効果的です。おすすめ度の高い順に紹介します。

## B.1 正しいドキュメントを参照させる

### Context7（MCP サーバー）

ライブラリの最新ドキュメントを、エージェントが必要なときに検索して読める MCP サーバーです。Tauri v2・serde・tokio などを「記憶」ではなく「現行のドキュメント」に基づいて書かせられます。

```sh
claude mcp add --transport http context7 https://mcp.context7.com/mcp
```

API キーなしでも使えます。よく使うなら https://context7.com/dashboard で無料のキーを取得するとレート制限が緩くなります（キーの渡し方は https://context7.com/docs/resources/all-clients を参照）。

プロンプトに「Tauri v2 のドキュメントを Context7 で確認してから実装して」と添えるだけでも、v1 の API が混ざるのをかなり防げます。

### Tauri 公式の llms.txt

Tauri のドキュメントは、LLM 向けにまとめたテキスト版を公開しています。

- 目次：https://v2.tauri.app/llms.txt
- 全文：https://v2.tauri.app/llms-full.txt
- 要約版：https://v2.tauri.app/llms-small.txt

`CLAUDE.md` に「Tauri の API は https://v2.tauri.app/llms.txt を参照すること」と書いておくと、エージェントが自分で取りに行きます。

### プロジェクトの CLAUDE.md

**最も効果が高いのは、自分のプロジェクトで実際に踏んだ落とし穴を書いておくこと** です。このリポジトリの `CLAUDE.md` の「注意点」がその例です（`rename_all_fields`、`Menu::default`、CSP の `ipc:` など）。汎用のスキルには載っていない、あなたのアプリ固有の知識になります。

## B.2 コードを正確に読ませる：rust-analyzer LSP プラグイン

Anthropic 公式マーケットプレイスのプラグインです。エージェントが rust-analyzer を使い、定義へのジャンプ、参照の検索、コンパイラや clippy の診断を利用できるようになります。毎回 `cargo check` を実行して出力を読むより速く、正確です。

```sh
# Claude Code 内で
/plugin install rust-analyzer-lsp@claude-plugins-official
```

rust-analyzer 本体も必要です。このリポジトリでは `mise.toml` の components に追加済みなので、`mise install` で全員の環境にそろって入ります。

```toml
rust = { version = "1.99.1", components = "rustfmt,clippy,rust-analyzer" }
```

## B.3 動いているアプリを見せる：Tauri 用 MCP サーバー

エージェントは通常、**画面を見られません**。コンパイルが通っても UI が崩れている、ボタンを押すと invoke が reject される、といった問題に気づけません。Tauri 用の MCP サーバーを入れると、起動中のアプリのスクリーンショット、DOM、Console のログ、IPC の呼び出しをエージェントが直接確認でき、クリックなどの操作もできるようになります。

主な選択肢：

| プロジェクト | 特徴 |
|---|---|
| [hypothesi/mcp-server-tauri](https://github.com/hypothesi/mcp-server-tauri) | 機能が最も多い（スクリーンショット、DOM、ログ、UI 操作、IPC 監視、モバイル対応）。MIT。MCP サーバー（npm：`@hypothesi/tauri-mcp-server`）と、アプリに組み込む Rust プラグイン（`tauri-plugin-mcp-bridge`）の2つで構成される |
| [P3GLEG/tauri-plugin-mcp](https://github.com/P3GLEG/tauri-plugin-mcp) | スクリーンショットをファイルに保存し、縮小版だけ返すのでトークンを節約できる |

```sh
# MCP サーバーの登録（hypothesi の場合）
claude mcp add tauri -- npx -y @hypothesi/tauri-mcp-server
```

アプリ側に組み込むプラグインのセットアップは、各リポジトリの README に従ってください。

> **セキュリティ上の注意**：これらのプラグインは、外部から WebView を操作できる口をアプリに開けます。必ず `#[cfg(debug_assertions)]` で **dev ビルドだけに** 組み込み、リリースビルドに入らないことを確認してください（付録A の CrabNebula DevTools と同じ書き方です）。

## B.4 エージェントスキル

スキルは、特定の作業のための手順や知識を `SKILL.md` にまとめたもので、エージェントは関係する作業のときだけ読み込みます。

### Rust 全般：leonardomso/rust-skills

[leonardomso/rust-skills](https://github.com/leonardomso/rust-skills)（MIT）は、所有権、エラー処理、async（tokio）、API 設計、テストなど 26 カテゴリのルール集です。各ルールに「悪い例・良い例・理由」が付いており、エージェントが `clone()` や `unwrap()` に逃げるのを抑えられます。Rust 2024 edition 向けです。

```sh
# このプロジェクトだけで使う場合
git clone https://github.com/leonardomso/rust-skills.git .claude/skills/rust-skills
```

学習者にとっても、各ルールの「良い例・悪い例」は Part 1 の復習教材として読む価値があります。

### Tauri 専用のスキル

コミュニティ製のものが多数あります。

| リポジトリ | 内容 |
|---|---|
| [nodnarbnitram/claude-code-extensions](https://github.com/nodnarbnitram/claude-code-extensions)（`tauri-v2`） | command・IPC・capability の設定。`generate_handler!` への登録漏れのような定番ミスも扱う |
| [dchuk/claude-code-tauri-skills](https://github.com/dchuk/claude-code-tauri-skills) | セットアップ、セキュリティ、配布などを 39 のスキルに分割 |

ただし、**どれも個人が作ったもので、内容の正確さは確認できていません**。調査時点でスター数は数十程度、コミット履歴も浅いものが中心でした。導入する前に `SKILL.md` を読み、本チュートリアルの内容（v2 の API、capability の書き方など）と食い違いがないかを確かめてください。古い Tauri v1 の書き方が混ざっているスキルは、かえって害になります。

**おすすめは、汎用の Tauri スキルを探すより、B.1 の「Context7 + llms.txt + 自分の CLAUDE.md」で済ませることです**。それでも足りない作業（例：「新しい command を追加する手順」）が出てきたら、自分用のスキルとして書き起こすのが確実です。

```markdown
<!-- .claude/skills/add-tauri-command/SKILL.md の例 -->
---
name: add-tauri-command
description: Tauri の command を新しく追加するときの手順
---

1. ロジックは notes.rs などの Tauri 非依存の層に書き、#[cfg(test)] でテストする
2. commands.rs に #[tauri::command] を追加する。戻り値は AppResult<T>
3. lib.rs の generate_handler! に登録する
4. src/api.ts に型付きのラッパーを追加する（引数名は camelCase）
5. 戻り値の型に serde 属性があれば、serde_json::to_string の assert テストを追加する
6. cargo clippy -- -D warnings と pnpm build が通ることを確認する
```

## B.5 エージェントに任せるときのコツ

- **確認コマンドを決めておく**：`cargo test`、`cargo clippy -- -D warnings`、`pnpm build` の3つが通るまで「完了」と言わせない（`CLAUDE.md` に書いておく）
- **capability の変更は人間がレビューする**：エージェントは、エラーを消すために `fs:default` やスコープなしの広い権限を足しがちです。`capabilities/*.json` の差分は必ず自分で読みましょう（Claude Code なら `/security-review` も使えます）
- **Rust ⇔ TS の境界はテストで固める**：JSON の形をテストで固定しておけば、エージェントが serde 属性を変えたときにすぐ気づけます（第8章）
- **学習中は、まず自分で書いてからレビューさせる**：エージェントに書かせたコードを読むだけでは、所有権の感覚は身につきません。「なぜこの `clone()` が必要なのか説明して」と聞くのは、良い学習方法です

## 出典

- [Context7](https://github.com/upstash/context7)
- [Tauri llms.txt](https://v2.tauri.app/llms.txt)
- [Rust Analyzer LSP（Claude プラグイン）](https://claude.com/marketplace/plugins/rust-analyzer-lsp)
- [hypothesi/mcp-server-tauri](https://github.com/hypothesi/mcp-server-tauri)
- [P3GLEG/tauri-plugin-mcp](https://github.com/P3GLEG/tauri-plugin-mcp)
- [leonardomso/rust-skills](https://github.com/leonardomso/rust-skills)
- [nodnarbnitram/claude-code-extensions](https://github.com/nodnarbnitram/claude-code-extensions)
- [dchuk/claude-code-tauri-skills](https://github.com/dchuk/claude-code-tauri-skills)
