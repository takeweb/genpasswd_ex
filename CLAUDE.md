# genpasswd_ex

パスワード生成 + サービスごとの履歴管理 CLI（Rust / clap / rusqlite）。使い方は README.md を参照。

## 構成

- `src/main.rs` — CLI 定義（clap derive）とサブコマンド処理
- `src/db.rs` — SQLite アクセス（`password_history` テーブル）
- `src/password.rs` — パスワード生成
- `src/crypto.rs` — パスワードの暗号化/復号（AES-256-GCM）と鍵の取得（macOS キーチェーン）
- `src/clipboard.rs` — クリップボードへのコピー（外部コマンド `pbcopy` 等を利用）

## DB

- テーブル: `password_history (id, service, username, password, created_at)`
- 保存先:
  - release ビルド: `~/.local/share/genpasswd_ex/history.db`
  - debug ビルド（`cargo run`）: リポジトリ直下の `history_dev.db`
  - 環境変数 `GENPASSWD_EX_DB`（`.env` も可）が設定されていればそれを優先
- DBファイルは開くたびに `600`、データディレクトリは `700` に設定する（unix のみ）
- 本番DB（`history.db`）を更新する場合は、コマンド/SQL を提示してユーザに実行してもらう

## 仕様メモ

- `save` / `register` で `-u` 省略時は、同じサービスの履歴にある最新の空でないユーザ名を使う（`resolve_username`）。なければ空。
- 引数なし / `save` で生成したパスワードはクリップボードにコピーする（`--no-copy` で無効化、失敗時は警告のみ）。
- パスワードは `enc:v1:` + base64(nonce 12B + 暗号文) で保存。プレフィックスなしは旧形式の平文として扱い、`encrypt` で一括暗号化（VACUUM で旧データも消去）。
- 鍵はキーチェーン（service `genpasswd_ex`、account は release `db-key` / debug `db-key-dev`）。`GENPASSWD_EX_KEY`（base64 32B）があれば優先。未登録（errSecItemNotFound）の場合のみ新規作成する。
- `update <ID> [-u] [-p]` で既存レコードのユーザ名・パスワードを更新できる（`-u`/`-p` の少なくとも一方が必須、未指定項目は変更しない）。
- `$` を含むパスワードはシェルで展開されるため、引数や SQL はシングルクォートで囲む（`sqlite3` はクォート付きヒアドキュメント `<<'EOF'` を使う）。

## 開発

- ビルド: `cargo build`
- 動作確認は本番DBを汚さないよう `GENPASSWD_EX_DB` にスクラッチの DB パスを指定して行う（鍵も `GENPASSWD_EX_KEY` でテスト用を指定するとキーチェーンを使わない）
- インストール: `cargo install --path . --root ~/`
