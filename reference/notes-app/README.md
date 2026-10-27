# Tauri Notes（完成版）

チュートリアル Part 2（第6〜10章）で作るメモアプリの完成形です。章を進めていて詰まったときの答え合わせに使ってください。

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
cargo test --manifest-path src-tauri/Cargo.toml
```

## 章とファイルの対応

| ファイル | 登場する章 |
|---|---|
| `src-tauri/src/error.rs` | 第7章（作成）→ 第8章 `Tauri` → 第9章 `InvalidPath` を追加 |
| `src-tauri/src/notes.rs` | 第7章（このまま丸ごとコピー） |
| `src-tauri/src/commands.rs` | 第7章 CRUD → 第8章 `emit` / `analyze_notes` → 第9章 `export_note` |
| `src-tauri/src/lib.rs` | 第7章 State 登録 → 第8〜9章 command 登録 → 第10章 メニュー・トレイ |
| `src-tauri/capabilities/default.json` | 第9章（`dialog:default`） |
| `src/api.ts` | 第7章 → 第8章 `analyzeNotes` → 第9章 `exportNote` |
| `src/App.tsx` | 第7章 → 第8章 イベント購読・解析ボタン → 第9章 書き出し → 第10章 メニューイベント |

第10章の「閉じるボタンでトレイに格納」「設定ウィンドウ」と、第9章の CSP は「やってみよう」の課題なので、ここには含めていません。
