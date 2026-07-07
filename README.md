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

## 使い方

### パスワードを生成する（保存しない）

```
genpasswd_ex [--length <長さ>] [--symbols]
```

| オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--length` | `-l` | 16 | パスワードの文字数 |
| `--symbols` | なし | off | 記号を含める |

```
# 16文字（デフォルト）
$ genpasswd_ex
Generated password: TDeXIVCda7bEWQts

# 24文字・記号あり
$ genpasswd_ex --length 24 --symbols
Generated password: ]]:dwmstZjQ9scuU1qLRLgLQ
```

---

### パスワードを生成して履歴に保存する

```
genpasswd_ex save <SERVICE> [--username <ユーザ名>] [--length <長さ>] [--symbols]
```

| 引数/オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `SERVICE` | — | 必須 | サービス名 |
| `--username` | `-u` | 空 | サービスのユーザ名 |
| `--length` | `-l` | 16 | パスワードの文字数 |
| `--symbols` | なし | off | 記号を含める |

```
# ユーザ名なし
$ genpasswd_ex save github
Generated password: CW5HsZlYxIib2Ut4
Saved to history for service "github".

# ユーザ名あり・20文字・記号あり
$ genpasswd_ex save github --username octocat --length 20 --symbols
Generated password: (*>asM70MQrmdd1mM5bT
Saved to history for service "github" (user: octocat).
```

生成されたパスワードは標準出力、保存メッセージは標準エラー出力に書き出されます。

---

### サービスの履歴を確認する

```
genpasswd_ex history <SERVICE>
```

```
$ genpasswd_ex history github
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
genpasswd_ex register <SERVICE> <PASSWORD> [--username <ユーザ名>]
```

| 引数/オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `SERVICE` | — | 必須 | サービス名 |
| `PASSWORD` | — | 必須 | 登録するパスワード |
| `--username` | `-u` | 空 | サービスのユーザ名 |

```
$ genpasswd_ex register github myP@ssw0rd --username octocat
Registered to history for service "github" (user: octocat).
```

---

### サービスの履歴を削除する

```
genpasswd_ex delete <SERVICE>
```

```
$ genpasswd_ex delete aws
Deleted 2 record(s) for service "aws".
```

指定したサービスのすべての履歴が削除されます。
