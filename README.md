# tree-viewer

postgresql-cst-parser の tree-sitter like tree viewer です。

## 機能

- SQLファイルを解析し、以下の2つの形式で表示できます：
  - CST（具象構文木）
  - トークン列

## インストール

```bash
cargo install --git https://github.com/lemonadern/tree-viewer.git
```

## 使い方

### 基本的な使い方

```bash
tree-viewer <SQLファイル> [コマンド]
```

### コマンド

#### `tree`

CSTを表示します。

```bash
tree-viewer sample.sql tree
```

オプション：
- `-d, --depth <DEPTH>`: 表示する木の深さ範囲を指定（例: 3, 1..3, 1..=3, ..3, ..=3, 3..）
- `--hide-range`: ノードの範囲情報を非表示
- `--show-text`: すべてのノードのテキストを表示
- `--show-node-text`: 非トークンノードのテキストを表示
- `--hide-token-text`: トークンのテキストを非表示
- `--show-node-type`: ノードの種類（NodeまたはToken）を表示

#### `tokens`

トークン列を表示します。

```bash
tree-viewer sample.sql tokens
```

オプション：
- `--hide-range`: トークンの範囲情報を非表示
- `--hide-text`: トークンのテキストを非表示

### 例

```sql
-- sample.sql
select a
from b
where c = "hi"
;
```

```bash
# CSTを表示
$ tree-viewer sample.sql tree
Root [(0, 0)-(4, 0)]
-+SelectStmt [(0, 0)-(2, 14)]
---+SELECT [(0, 0)-(0, 6)] "select"
---+target_list [(0, 7)-(0, 8)]
...

# トークン列を表示
$ tree-viewer sample.sql tokens
SELECT@[(0, 0)-(0, 6)] "select"
IDENT@[(0, 7)-(0, 8)] "a"
FROM@[(1, 0)-(1, 4)] "from"
...
```

## ライセンス

[Apache License 2.0](./LICENSE)
