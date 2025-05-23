# 号令当番生成 (gourei_touban)

学生リストが記載されたCSVファイルから、ランダムに号令当番（正担当・副担当の計2名）を選出するコマンドラインツールです。

## Vibe codingについて

このプロジェクトは、Vibe codingの課題として作成されました。Vibe codingは、AIに仕様書やその場の入力などで仕様を渡して、コードを作成していくと行った、AI側にコーディングを任せることの多い手法です。
このプロジェクトは、Vibe codingの手法を用いて、Rustで実装されています。仕様については、[spec.md](spec.md) を参照してください。

## 概要

指定されたCSVファイルを読み込み、リストの中から重複なく2名の学生をランダムに選択し、標準出力に表示します。

## 使い方

### 1. ビルド

プロジェクトディレクトリ内で以下のコマンドを実行してビルドします。

```bash
cargo build
```

実行ファイルは `target/debug/gourei_touban` に生成されます。リリースビルドを行う場合は `cargo build --release` を使用してください。

### 2. 実行

ビルド後、以下のコマンドで実行できます。

```bash
# プロジェクトルートから実行する場合
./target/debug/gourei_touban

# cargo run を使う場合 (ビルドも自動で行われます)
cargo run
```

デフォルトでは、実行ディレクトリにある `students.csv` ファイルを読み込みます。

### 3. CSVファイルの指定

`-f` または `--file` オプションで、使用するCSVファイルのパスを指定できます。

```bash
# cargo run を使う場合
cargo run -- --file /path/to/your/students.csv

# 直接実行する場合
./target/debug/gourei_touban --file /path/to/your/students.csv
```

### 4. シード値の指定 (テスト用)

`-s` または `--seed` オプションで乱数生成器のシード値を指定できます。同じシード値を使えば、常に同じ結果が得られます（テストや再現性の確認に便利です）。

```bash
cargo run -- --seed 42
```

## CSVファイルの形式

入力するCSVファイルは以下の形式である必要があります。

*   **エンコーディング:** UTF-8
*   **ヘッダー:** 1行目に `id,name` というヘッダーが必要です。
*   **列:** 1列目が学生ID (文字列)、2列目が学生名 (文字列) である必要があります。列の区切り文字はカンマ (`,`) です。

**例 (`students.csv`):**

```csv
id,name
1,佐藤
2,鈴木
3,高橋
4,田中
5,渡辺
```

## テスト

プロジェクトディレクトリ内で以下のコマンドを実行すると、単体テストおよび統合テストが実行されます。

```bash
cargo test
```

## ライセンス

このプロジェクトは GNU General Public License v3.0 の下でライセンスされています。
詳細については、リポジトリに含まれる `LICENSE` ファイルを参照してください。
