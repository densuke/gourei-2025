// gourei_touban CLI アプリケーションの統合テスト。

// `use` で外部クレートやモジュールの機能を取り込みます。
use assert_cmd::Command; // コマンド実行とアサーション（検証）用クレート
use predicates::prelude::*; // アサーション用の便利な関数やトレイト群
use std::fs; // ファイルシステム操作用
use std::path::PathBuf; // ファイルパス操作用
use tempfile::tempdir; // 一時ディレクトリ作成用クレート

// 一時ディレクトリ内にダミーの CSV ファイルを作成するヘルパー関数
// `dir: &tempfile::TempDir` は一時ディレクトリへの不変の参照を受け取ります。
// `filename: &str` はファイル名（文字列スライス）を受け取ります。
// `content: &str` はファイル内容（文字列スライス）を受け取ります。
// `-> PathBuf` は関数の戻り値が `PathBuf` 型（パスを表す構造体）であることを示します。
fn create_test_csv(dir: &tempfile::TempDir, filename: &str, content: &str) -> PathBuf {
    // `dir.path()` で一時ディレクトリのパスを取得し、`.join(filename)` でファイル名を結合します。
    let file_path = dir.path().join(filename);
    // `fs::write` でファイルに内容を書き込みます。
    // `expect(...)` は `Result` が `Err` の場合にプログラムをパニックさせ、指定したメッセージを表示します。
    fs::write(&file_path, content).expect("テスト CSV の書き込みに失敗しました");
    // 作成したファイルのパスを返します。
    file_path
}

// `#[test]` 属性は、この関数がテスト関数であることを示します。
// `cargo test` コマンドで実行されます。
#[test]
fn test_fixed_seed_selection() {
    // `tempdir().unwrap()` で一時ディレクトリを作成します。
    // `unwrap()` は `Result` が `Ok(value)` なら `value` を、`Err` ならパニックします。
    // テストにおいては、セットアップの失敗は即時パニックで問題ないことが多いです。
    let dir = tempdir().unwrap();
    let csv_content = "id,name\n1,Alice\n2,Bob\n3,Charlie\n4,David";
    // ヘルパー関数を使ってテスト用 CSV ファイルを作成します。
    let file_path = create_test_csv(&dir, "test_students_fixed.csv", csv_content);

    // `Command::cargo_bin("gourei_touban")` でテスト対象のバイナリ（実行可能ファイル）へのパスを取得します。
    // `.unwrap()` はバイナリが見つからない場合にパニックします。
    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    // `.arg()` でコマンドライン引数を追加します。
    cmd.arg("--file")
       .arg(file_path.to_str().unwrap()) // `PathBuf` を文字列スライス `&str` に変換
       .arg("--seed")
       .arg("42"); // 固定シード

    // --- !!! 重要 !!! ---
    // シード 42 での実際のテスト実行に基づいて期待される出力を更新
    let expected_output = "正担当: 1 Alice\n副担当: 3 Charlie\n"; // <-- この行を調整

    // `.assert()` でコマンドの実行結果に対するアサーションを開始します。
    // `.success()` はコマンドが正常終了（終了コード 0）したことを検証します。
    // `.stdout(...)` は標準出力が指定した内容と一致することを検証します。
    cmd.assert().success().stdout(expected_output);
}

#[test]
fn test_cli_file_argument() {
    let dir = tempdir().unwrap();
    let csv_content = "id,name\n10,Eve\n20,Frank";
    let file_path = create_test_csv(&dir, "cli_specified.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    // `predicate::str::contains(...)` は、出力に特定の文字列が含まれているかを検証する述語（predicate）です。
    // `.stdout(...)` に述語を渡すことで、より柔軟な検証が可能です。
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("正担当:"))
        .stdout(predicate::str::contains("副担当:"))
        // `predicate::str::is_match(...)` は正規表現にマッチするかを検証します。
        // `^.*\n.*\n$` は、何らかの文字（`.*`）が続き改行（`\n`）が2回あり、それで終わる（`$`）パターンです。
        .stdout(predicate::str::is_match("^.*\n.*\n$").unwrap());
}

#[test]
fn test_default_file_path() {
    let dir = tempdir().unwrap(); // 一時ディレクトリを作成
    let csv_content = "id,name\n100,Grace\n200,Heidi";
    // ヘルパーを使用して一時ディレクトリ内に students.csv を作成
    let _default_path = create_test_csv(&dir, "students.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    // `.current_dir(...)` でコマンドの実行ディレクトリを変更します。
    // これにより、デフォルトの "students.csv" が一時ディレクトリ内のものになります。
    cmd.current_dir(dir.path());

    // `.or(...)` は述語を組み合わせ、どちらか一方が真であれば成功とします。
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("正担当: 100 Grace").or(
                predicate::str::contains("正担当: 200 Heidi"))) // どちらかが正担当であることを確認
        .stdout(predicate::str::contains("副担当: 100 Grace").or(
                predicate::str::contains("副担当: 200 Heidi"))); // もう一方が副担当であることを確認
}

#[test]
fn test_error_file_not_found() {
    let dir = tempdir().unwrap(); // ディレクトリは存在するが、ファイルは存在しないことを確認
    let non_existent_path = dir.path().join("non_existent_file.csv");

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(non_existent_path.to_str().unwrap());

    // `.failure()` はコマンドが異常終了（終了コード 0 以外）したことを検証します。
    // `.stderr(...)` は標準エラー出力の内容を検証します。
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Could not open file"));
}

#[test]
fn test_error_default_file_not_found() {
    // 'students.csv' が含まれていないことが保証されているディレクトリから実行
    let dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.current_dir(dir.path()); // students.csv が存在しない場所で実行

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Could not open file"));
}

#[test]
fn test_error_empty_file_header_only() {
    let dir = tempdir().unwrap();
    let csv_content = "id,name\n"; // ヘッダーのみ
    let file_path = create_test_csv(&dir, "empty_header.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: The student list"));
}

#[test]
fn test_error_empty_file_no_header() {
    let dir = tempdir().unwrap();
    let csv_content = ""; // 完全に空
    let file_path = create_test_csv(&dir, "empty_no_header.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    // 完全に空のファイルの場合、解析エラーではなく「空のリスト」エラーを期待します
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: The student list")); // <-- この行を調整
}

#[test]
fn test_error_one_student() {
    let dir = tempdir().unwrap();
    let csv_content = "id,name\n1,Alice";
    let file_path = create_test_csv(&dir, "one_student.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Not enough students"));
}

#[test]
fn test_error_csv_format_invalid_delimiter() {
    let dir = tempdir().unwrap();
    let csv_content = "id;name\n1;Alice\n2;Bob"; // セミコロン区切り
    let file_path = create_test_csv(&dir, "invalid_delimiter.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    // デシリアライズは失敗するはずです
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Failed to parse CSV file"));
}

#[test]
fn test_error_csv_format_wrong_columns() {
    let dir = tempdir().unwrap();
    // Student 構造体は "id", "name" を期待します。これには余分な列があります。
    let csv_content = "id,name,extra\n1,Alice,foo\n2,Bob,bar";
    let file_path = create_test_csv(&dir, "wrong_columns.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    // flexible(false) が設定されたので、デシリアライズは失敗するはずです
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Failed to parse CSV file")); // アサーションは同じですが、動作は一致するはずです
}