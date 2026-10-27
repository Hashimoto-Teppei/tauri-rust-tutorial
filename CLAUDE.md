# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## このリポジトリについて

Rust と Tauri 2.0 を学ぶための日本語チュートリアル。学習者は TypeScript / React / Python 経験者で Electron Forge 経験あり。説明は常に「TS / Python / Electron の何に相当するか」を軸にする。回答・コメント・ドキュメントは日本語（ですます調、文中のコロンは全角「：」）。

## 環境

`mise.toml` で node / pnpm / rust（+rustfmt, clippy, rust-analyzer）を固定している。バージョンを変えるときは `reference/notes-app/package.json` の `packageManager` も合わせる。リポジトリ外（スクラッチ等）では mise の Rust が効かないので `mise exec rust@1.99.1 -- <cmd>` を使う。

## 構成

- `docs/00〜11-*.md` — 本文。Part 1（00〜05: Rust）→ Part 2（06〜11: Tauri）。章末は Part 1 が「演習」、Part 2 が「✅ 確認」＋「やってみよう」
- `exercises/` — Part 1 の演習用 Cargo workspace
  - `practice/` — 学習者が `todo!()` を埋める版。`solutions/` から関数本体を `todo!()` に置換したもの（ch03 の `Describe::name` と ch05 の serde 属性も意図的に削ってある）
  - `solutions/` — 模範解答
  - **両クレートの `chXX.rs` の docstring と `#[cfg(test)]` は同一に保つ**。docstring だけで仕様（境界値・空白の扱いなど）が分かるように書く
- `reference/notes-app/` — Part 2 で作るメモアプリの完成形（= 第10章 10.2 終了時点）
  - ロジックは `NoteStore`（notes.rs）に集約し Tauri 非依存でテスト可能にしている
  - `src/api.ts` が invoke の型付きラッパー（Electron の preload 相当）
- `notes-app/` — 学習者が第6章で作る作業用アプリ（存在しない場合もある）

### Part 2 の章ごとの段階

第7〜10章は章を順に進めるとコンパイルが通るよう、各章の「この章で触るファイル」と差分を載せている。第7章時点の `commands.rs` / `lib.rs` / `api.ts` / `App.tsx` は docs 内にしか存在しない。reference や章のコードを変えたら、その後の章の差分と矛盾しないか確認し、途中段階もスクラッチで `pnpm build` と `cargo clippy` を通して検証する。

## コマンド

```sh
mise trust && mise install

# 演習
cd exercises
cargo test -p practice ch03                       # 章ごと
cargo test -p practice ch03::tests::test_largest  # 単一テスト
cargo test -p solutions                           # 模範解答は全テスト通過が前提

# 完成版アプリ（reference/notes-app で）
pnpm install --frozen-lockfile
pnpm tauri dev
pnpm build                                        # tsc 型チェック + vite build
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings   # 第11章で学習者にも実行させるので必ず通す
cargo fmt --manifest-path src-tauri/Cargo.toml    # docs のコード片も fmt 後の形に合わせる
```

検証で生成した `src-tauri/target`, `src-tauri/gen`, `dist` は reference から消しておく。

## 注意点（このリポジトリで実際に踏んだもの）

- Rust ⇔ TS の JSON 形状がずれても TS の型チェックは通る。serde 属性を変えたら `commands.rs` 末尾のような `serde_json::to_string` の assert テストで確認する。enum の `rename_all` はバリアント名にしか効かず、フィールドには `rename_all_fields` が必要
- command は既定で capability の制限外で動くので、フロントから受け取るパス等は Rust 側で検証する（`export_note` の `.md` チェック）
- macOS でアプリメニューを差し替えるときは `Menu::default` をベースにする（`Menu::new` だと ⌘C/⌘V が消える）
- CSP を設定するなら `connect-src ipc: http://ipc.localhost` が必要（ないと IPC が遅い経路にフォールバックする）
- メモ id は時刻ベースだと同一ミリ秒で再利用され得たので、`AtomicU64` のカウンタにしている
