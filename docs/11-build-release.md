# 第11章 ビルドと配布

## 11.1 本番ビルド

```sh
pnpm tauri build
```

1. `beforeBuildCommand`（`pnpm build`）でフロントをビルド → `dist/`
2. Rust を release ビルド（`Cargo.toml` の `[profile.release]` で LTO・strip 済み）
3. OS ごとのインストーラを生成

出力先は `src-tauri/target/release/bundle/` です。

| OS | 生成物 |
|---|---|
| macOS | `.app`, `.dmg` |
| Windows | `.msi`（WiX）, `.exe`（NSIS） |
| Linux | `.deb`, `.rpm`, `.AppImage` |

Windows で `.msi` の作成時に `failed to run light.exe` が出る場合は、「設定 → アプリ → オプション機能 → Windows のその他の機能」で **VBSCRIPT** を有効にしてください。

**クロスコンパイルは基本できません**。各 OS 向けにはその OS 上でビルドします（CI なら `tauri-apps/tauri-action` の GitHub Actions がマトリクスビルドの定番）。

デバッグ用の速いビルド：`pnpm tauri build --debug`（DevTools も使える）

## 11.2 アイコン

```sh
pnpm tauri icon path/to/icon-1024.png   # 全サイズを src-tauri/icons/ に生成
```

## 11.3 コード署名

| OS | 必要なもの |
|---|---|
| macOS | Apple Developer ID 証明書 + notarization（公証）。環境変数 `APPLE_CERTIFICATE`, `APPLE_ID` 等をセットしてビルド |
| Windows | コード署名証明書（OV/EV）または Azure Trusted Signing |

署名なしでも動きますが、macOS の Gatekeeper / Windows の SmartScreen で警告が出ます。
詳細：https://v2.tauri.app/distribute/sign/macos/ ・ https://v2.tauri.app/distribute/sign/windows/

## 11.4 自動更新

```sh
pnpm tauri add updater
pnpm tauri signer generate -w ~/.tauri/notes.key    # 更新ファイル署名用の鍵ペア
```

- 公開鍵を `tauri.conf.json` の `plugins.updater.pubkey` に、更新情報 JSON の URL を `endpoints` に設定します
- `bundle.createUpdaterArtifacts: true` にしてビルドすると、署名付きの更新ファイルが生成されます。このとき秘密鍵を **環境変数** `TAURI_SIGNING_PRIVATE_KEY`（パスワードを付けた場合は `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` も）に入れておく必要があります。`.env` ファイルは読まれません
- capability に `"updater:default"` があることを確認します（`tauri add` で追加されます）
- フロントで `check()` → `downloadAndInstall()` → `relaunch()` の順に呼びます。`relaunch()` には `pnpm tauri add process` で入る `@tauri-apps/plugin-process` が必要です

electron-updater と違い **署名検証が必須** です（鍵をなくすと既存ユーザーに更新を配れなくなるので厳重に保管）。

## 11.5 テスト

| 対象 | 方法 |
|---|---|
| Rust のロジック | `cargo test --manifest-path src-tauri/Cargo.toml`。Tauri から切り離した struct（`NoteStore` など）をテストするのが最もコスパが良い |
| Rust ⇔ TS の JSON 形状 | `serde_json::to_string` の結果を assert（第8章） |
| フロント単体 | Vitest。`@tauri-apps/api/mocks` の `mockIPC` で invoke をモック |
| E2E | WebDriver（`tauri-driver`）。`tauri-driver` 単体は Windows / Linux のみ対応。macOS も対象にするなら `@wdio/tauri-service` などを検討（https://v2.tauri.app/develop/tests/webdriver/） |

```ts
// Vitest で invoke をモックする例
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";

mockIPC((cmd, args) => {
  if (cmd === "list_notes") return [{ id: 1, title: "a", body: "", updatedAt: 0 }];
});
```

## 11.6 Lint とフォーマット

```sh
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

clippy は「もっと Rust らしい書き方」を提案してくれるので、学習中こそ積極的に使いましょう。

## 修了

お疲れさまでした。次のステップの候補：

- `tauri-specta` で command の TS 型を自動生成する
- `tauri-plugin-sql` で SQLite に移行する（JSON ファイル全書き換えからの脱却）
- `pnpm tauri ios dev` / `android dev` でモバイル版を動かす
- [The Rust Book](https://doc.rust-jp.rs/book-ja/) の第15章（スマートポインタ）・第16章（並行性）で理解を深める
