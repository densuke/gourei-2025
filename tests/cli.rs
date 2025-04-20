// gourei_touban CLI アプリケーションの統合テスト。

use assert_cmd::Command;
use predicates::prelude::*; // stdout/stderr アサーションに使用
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir; // よりクリーンなテストのセットアップ/ティアダウンのために tempfile を使用

// 一時ディレクトリ内にダミーの CSV ファイルを作成するヘルパー関数
fn create_test_csv(dir: &tempfile::TempDir, filename: &str, content: &str) -> PathBuf {
    let file_path = dir.path().join(filename);
    fs::write(&file_path, content).expect("テスト CSV の書き込みに失敗しました");
    file_path
}

#[test]
fn test_fixed_seed_selection() {
    let dir = tempdir().unwrap();
    let csv_content = "id,name\n1,Alice\n2,Bob\n3,Charlie\n4,David";
    let file_path = create_test_csv(&dir, "test_students_fixed.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file")
       .arg(file_path.to_str().unwrap())
       .arg("--seed")
       .arg("42"); // 固定シード

    // --- !!! 重要 !!! ---
    // シード 42 での実際のテスト実行に基づいて期待される出力を更新
    let expected_output = "正担当: 1 Alice\n副担当: 3 Charlie\n"; // <-- この行を調整

    cmd.assert().success().stdout(expected_output);
}

#[test]
fn test_cli_file_argument() {
    let dir = tempdir().unwrap();
    let csv_content = "id,name\n10,Eve\n20,Frank";
    let file_path = create_test_csv(&dir, "cli_specified.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    // ラベルが存在し、出力が正確に 2 行であることを確認します。
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("正担当:"))
        .stdout(predicate::str::contains("副担当:"))
        // 正規表現を使用して、改行で終わる正確に 2 行であることを確認します
        .stdout(predicate::str::is_match("^.*\n.*\n$").unwrap());
}

#[test]
fn test_default_file_path() {
    let dir = tempdir().unwrap(); // 一時ディレクトリを作成
    let csv_content = "id,name\n100,Grace\n200,Heidi";
    // ヘルパーを使用して一時ディレクトリ内に students.csv を作成
    let _default_path = create_test_csv(&dir, "students.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    // テスト用の students.csv を含む一時ディレクトリからコマンドを実行
    cmd.current_dir(dir.path());

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