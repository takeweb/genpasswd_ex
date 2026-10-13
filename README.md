# genpasswd_ex

パスワード生成 + サービスごとの履歴管理ツール。

パスワードはランダムに生成され、サービス名・ユーザ名と紐づけて SQLite に保存できます。

## インストール

`~/bin` にインストールする場合:

```
cargo install --path . --root ~/
```

`~/bin` が `$PATH` に含まれていれば、どこからでも `genpasswd_ex` で実行できます。

## ビルド（インストールせずに試す場合）

```
cargo build --release
```

バイナリは `target/release/genpasswd_ex` に生成されます。

## データ保存先

```
~/.local/share/genpasswd_ex/history.db
```

ディレクトリは `700`、DBファイルは `600`（所有者のみ読み書き可）に設定されます。既存のファイルも実行時に同じ権限へ変更されます。

## 暗号化

パスワードは AES-256-GCM で暗号化して DB に保存されます。暗号化鍵は macOS のキーチェーン（サービス `genpasswd_ex` / アカウント `db-key`）に保存され、初回の保存時に自動で作成されます。

- `history` では復号して表示するため、使い方は変わりません
- 再インストール後などは、初回アクセス時にキーチェーンへのアクセス許可ダイアログが表示されます（「常に許可」を選ぶと以後は表示されません）
- **キーチェーンの鍵を削除すると、保存済みのパスワードは復号できなくなります**
- macOS 以外では、環境変数 `GENPASSWD_EX_KEY` に base64 の 32 バイト鍵を指定してください（`openssl rand -base64 32` で生成可）

旧バージョンで保存した平文のパスワードもそのまま表示できます。暗号化するには次を実行してください（念のため事前に DB をバックアップしてください）。

```
$ genpasswd_ex encrypt
Encrypted 12 password(s).
```

## 使い方

### パスワードを生成する（保存しない）

```
genpasswd_ex [--length <長さ>] [--symbols] [--no-copy]
```

| オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--length` | `-l` | 16 | パスワードの文字数 |
| `--symbols` | `-k` | off | 記号を含める |
| `--no-copy` | `-n` | off | クリップボードにコピーしない |

```
# 16文字（デフォルト）
$ genpasswd_ex
Generated password: TDeXIVCda7bEWQts
Copied to clipboard.

# 24文字・記号あり
$ genpasswd_ex --length 24 --symbols
Generated password: ]]:dwmstZjQ9scuU1qLRLgLQ
Copied to clipboard.
```

生成されたパスワードは自動でクリップボードにコピーされます（macOS は `pbcopy`、Linux は `wl-copy` / `xclip` / `xsel` を使用）。コピーに失敗した場合は警告を表示し、処理は続行します。

---

### パスワードを生成して履歴に保存する

```
genpasswd_ex save --service <SERVICE> [--username <ユーザ名>] [--length <長さ>] [--symbols] [--no-copy]
```

| 引数/オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--service` | `-s` | 必須 | サービス名 |
| `--username` | `-u` | 履歴の最新ユーザ名 | サービスのユーザ名（省略時は同サービスの履歴にある最新のユーザ名。履歴になければ空） |
| `--length` | `-l` | 16 | パスワードの文字数 |
| `--symbols` | `-k` | off | 記号を含める |
| `--no-copy` | `-n` | off | クリップボードにコピーしない |

```
# ユーザ名なし
$ genpasswd_ex save -s github
Generated password: CW5HsZlYxIib2Ut4
Copied to clipboard.
Saved to history for service "github".

# ユーザ名あり・20文字・記号あり
$ genpasswd_ex save -s github --username octocat --length 20 --symbols
Generated password: (*>asM70MQrmdd1mM5bT
Copied to clipboard.
Saved to history for service "github" (user: octocat).
```

生成されたパスワードは標準出力、コピー・保存メッセージは標準エラー出力に書き出されます。

---

### サービスの履歴を確認する

```
genpasswd_ex history --service <SERVICE>
```

```
$ genpasswd_ex history -s github
History for "github":
  ID  Username              Password              Created At
--------------------------------------------------------------------------------
   7  octocat               YWXWA3xS402Wl5JKMmsQ  2026-07-07T08:07:54.151484+09:00
   5  octocat               9Jn2UoOc5EMMofvB      2026-07-07T08:06:01.893545+09:00
   1  -                     (*>asM70MQrmdd1mM5bT  2026-07-07T08:01:39.249806+09:00
```

新しいパスワードが先頭に表示されます。ユーザ名未設定の場合は `-` と表示されます。

---

### 登録済みサービス一覧を表示する

```
genpasswd_ex list
```

```
$ genpasswd_ex list
Service  Count
---------------
aws          2
github       4
```

---

### 既存のパスワードを履歴に登録する

```
genpasswd_ex register --service <SERVICE> <PASSWORD> [--username <ユーザ名>]
```

| 引数/オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--service` | `-s` | 必須 | サービス名 |
| `PASSWORD` | — | 必須 | 登録するパスワード |
| `--username` | `-u` | 履歴の最新ユーザ名 | サービスのユーザ名（省略時は同サービスの履歴にある最新のユーザ名。履歴になければ空） |

```
$ genpasswd_ex register -s github myP@ssw0rd --username octocat
Registered to history for service "github" (user: octocat).
```

---

### 履歴のユーザ名・パスワードを更新する

```
genpasswd_ex update <ID> [--username <ユーザ名>] [--password <パスワード>]
```

| 引数/オプション | 短縮 | 説明 |
|---|---|---|
| `ID` | — | 更新する履歴のID（`history` で確認） |
| `--username` | `-u` | 新しいユーザ名 |
| `--password` | `-p` | 新しいパスワード |

`-u` と `-p` の少なくとも一方が必要です。指定しなかった項目は変更されません。

```
$ genpasswd_ex update 16 -u octocat
Updated history entry ID 16.

$ genpasswd_ex update 15 -p 'C$Y3xvqiRsHF2H.'
Updated history entry ID 15.
```

`$` などを含むパスワードはシェルに展開されないようシングルクォートで囲んでください。

---

### サービスの履歴を削除する

```
genpasswd_ex delete --service <SERVICE>
```

```
$ genpasswd_ex delete -s aws
Deleted 2 record(s) for service "aws".
```

指定したサービスのすべての履歴が削除されます。

## ライセンス

MIT License

Copyright (c) 2026 Taketomo Oishi

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
