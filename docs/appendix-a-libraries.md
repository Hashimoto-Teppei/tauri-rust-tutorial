# 付録A Tauri 開発で便利なライブラリ

> 2026年10月時点の調査です。バージョンや開発状況は変わるので、導入前に各リポジトリを確認してください。
> ★ は「まず入れてよい」もの、無印は「必要になったら」検討するものです。

## A.1 公式プラグイン（tauri-apps/plugins-workspace）

`pnpm tauri add <名前>` の1コマンドで、Rust・JS・capability の3か所に追加できます（第9章）。本編で使った `dialog` と `opener` 以外の主なものを挙げます。

| プラグイン | 用途 | 使いどころ |
|---|---|---|
| ★ `log` | Rust と JS のログをまとめて、ターミナル・ファイル・WebView の Console に出す | `println!` から卒業するときに最初に入れる。ログファイルはユーザーの不具合報告にも使える |
| ★ `window-state` | ウィンドウの位置・サイズを記憶する | ほぼすべてのデスクトップアプリで欲しくなる |
| ★ `single-instance` | 多重起動を防ぎ、2つ目の起動を1つ目に通知する | `deep-link` と組み合わせるときは必須 |
| `store` | JSON ファイルベースの key-value ストア | 設定値の保存。本編の `NoteStore` のような自前実装の代わり |
| `sql` | SQLite / MySQL / PostgreSQL（sqlx ベース） | フロントから直接 SQL を発行したいとき。Rust 側で完結させるなら後述の `sqlx` / `rusqlite` |
| `http` | Rust 側で HTTP 通信する（CORS の制約を受けない） | 外部 API を WebView から叩きたいとき |
| `updater` + `process` | 自動更新と再起動 | 配布するなら必須（第11章） |
| `deep-link` | `myapp://...` の URL でアプリを開く | OAuth ログインのコールバックなど |
| `global-shortcut` | アプリが非アクティブでも効くショートカット | ランチャー系のアプリ |
| `autostart` | OS 起動時に自動起動する | 常駐アプリ |
| `notification` | OS の通知 | |
| `clipboard-manager` | クリップボード | |
| `shell` | 子プロセスの起動（サイドカー） | 同梱した CLI ツールを動かすとき。権限は最小限に |
| `positioner` | ウィンドウをトレイの下などの定位置へ動かす | メニューバー常駐アプリ |
| `stronghold` | 暗号化されたシークレットの保存 | パスワードや API キーの保存 |
| `os` | OS の種類・バージョン・ロケールの取得 | |

対応プラットフォームの一覧：https://github.com/tauri-apps/plugins-workspace

## A.2 Rust のクレート

| クレート | 用途 | メモ |
|---|---|---|
| ★ `thiserror` | エラー型の定義 | 本編で使用（第4章・第7章） |
| `anyhow` | 細かいエラー型を作らないアプリ向けのエラー処理 | `setup` 内や CLI ツールで。command の戻り値にはそのまま使えない（Serialize できない）点に注意 |
| ★ `serde` / `serde_json` | JSON 変換 | 本編で使用 |
| ★ `tokio` | 非同期ランタイム | Tauri 自体が使っている。`time`・`fs`・`sync` など必要な機能だけ有効にする |
| `tracing` + `tracing-subscriber` | 構造化ログ・スパン | 非同期処理の追跡に強い。`log` プラグインとはどちらかに寄せる |
| `reqwest` | HTTP クライアント | Rust 側で API を叩くとき。`http` プラグインの中身もこれ |
| `sqlx` / `rusqlite` | SQLite など | `sqlx` は非同期とコンパイル時の SQL 検証、`rusqlite` は同期でシンプル |
| `uuid` | ID 生成 | 本編の連番 id の代わりに。`features = ["v4"]` や時刻順の `v7` |
| `chrono` / `jiff` | 日時 | `jiff` は新しめでタイムゾーンの扱いが堅い |
| `notify` | ファイル変更の監視 | 「外部エディタで編集されたら再読み込み」など |
| `keyring` | OS のキーチェーン / 資格情報マネージャー | トークンの安全な保存 |

## A.3 Rust ⇔ TS の型共有

第7章で触れた「型の二重定義」を解消するライブラリです。

| ライブラリ | 特徴 | 状況 |
|---|---|---|
| `tauri-specta` | command・イベントから TS の型と **呼び出し関数ごと** 生成する。`api.ts` 相当を手で書かなくてよくなる | Tauri v2 対応版は **2.0.0-rc** 系（RC 段階） |
| `ts-rs` | `#[derive(TS)]` した struct / enum から TS の型定義だけを生成する | 安定。Tauri に依存しないので導入が軽い |

どちらも serde の `rename_all` 等を読んで型を出すので、第8章のような「JSON の形のずれ」も防げます。RC を避けたいなら `ts-rs` から始めるのが無難です。

## A.4 フロントエンド

| ライブラリ | 用途 | メモ |
|---|---|---|
| ★ `@tanstack/react-query` | `invoke` の結果のキャッシュ・再取得・ローディング状態 | `queryFn: api.listNotes` とし、`notes-changed` イベントで `invalidateQueries` すると第8章の `reload` がきれいに書ける |
| `zustand` | 軽量な状態管理 | 複数ウィンドウや多画面になったとき |
| ★ `vitest` + `@tauri-apps/api/mocks` | フロントの単体テスト | `mockIPC` で invoke をモック（第11章） |

## A.5 開発・デバッグ・CI のツール

| ツール | 用途 | メモ |
|---|---|---|
| ★ CrabNebula DevTools（`tauri-plugin-devtools`） | command の呼び出し・所要時間、イベント、ログをブラウザの UI で見る | 公式ドキュメントで紹介されている。**`#[cfg(debug_assertions)]` で dev ビルドだけ** 有効にする。他のロガーと同時に使うと起動時に落ちることがある |
| ★ `tauri-apps/tauri-action` | GitHub Actions で各 OS 向けにビルドし、Release にアップロードする | クロスコンパイルできない Tauri の配布ではほぼ必須（第11章） |
| `cargo-nextest` | 高速なテストランナー | テストが増えてきたら |
| `bacon` | 保存のたびに `cargo check` / `clippy` を走らせ続ける | Rust 学習中の「書く→エラーを見る」ループが速くなる |
| `cargo-machete` | 使っていない依存クレートを検出 | |
| `cargo-deny` | ライセンス・脆弱性・重複依存のチェック | 配布前に |

CrabNebula DevTools の導入例（公式ドキュメントより）：

```sh
cd src-tauri && cargo add tauri-plugin-devtools@2.0.0
```

```rust
// lib.rs の run() 冒頭
#[cfg(debug_assertions)]
let devtools = tauri_plugin_devtools::init();

let mut builder = tauri::Builder::default();

#[cfg(debug_assertions)]
{
    builder = builder.plugin(devtools);
}
// 以降は builder.plugin(...).setup(...)... と続ける
```

## 出典

- [tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace)
- [Tauri v2: CrabNebula DevTools](https://v2.tauri.app/develop/debug/crabnebula-devtools/)
- [specta-rs/tauri-specta](https://github.com/specta-rs/tauri-specta)
- [awesome-tauri](https://github.com/tauri-apps/awesome-tauri)
