// gourei_touban CLI アプリケーションの統合テスト。

// `use` で外部クレートやモジュールの機能を取り込みます。
use assert_cmd::Command; // コマンド実行とアサーション（検証）用クレート
use predicates::prelude::*; // アサーション用の便利な関数やトレイト群
use std::fs; // ファイルシステム操作用
use std::path::PathBuf; // ファイルパス操作用
use tempfile::{tempdir, TempDir}; // 一時ディレクトリ作成用クレート

// ヘルパー関数: 一時ディレクトリとテスト用CSVファイルを作成
fn setup_test_env(filename: &str, content: &str) -> (TempDir, PathBuf) {
    let dir = tempdir().expect("一時ディレクトリの作成に失敗しました");
    let file_path = dir.path().join(filename);
    fs::write(&file_path, content).expect("テスト CSV の書き込みに失敗しました");
    (dir, file_path) // TempDir と PathBuf をタプルで返す
}

#[test]
fn test_fixed_seed_selection() {
    let csv_content = "id,name\n1,Alice\n2,Bob\n3,Charlie\n4,David";
    // ヘルパー関数でセットアップ
    let (_dir, file_path) = setup_test_env("test_students_fixed.csv", csv_content); // dir は使わないので _dir とする

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file")
       .arg(file_path.to_str().unwrap()) // `PathBuf` を文字列スライス `&str` に変換
       .arg("--seed")
       .arg("42"); // 固定シード

    let expected_output = "正担当: 1 Alice\n副担当: 3 Charlie\n"; // <-- この行を調整

    cmd.assert().success().stdout(expected_output);
}

#[test]
fn test_cli_file_argument() {
    let csv_content = "id,name\n10,Eve\n20,Frank";
    // ヘルパー関数でセットアップ
    let (_dir, file_path) = setup_test_env("cli_specified.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("正担当:"))
        .stdout(predicate::str::contains("副担当:"))
        .stdout(predicate::str::is_match("^.*\n.*\n$").unwrap());
}

#[test]
fn test_default_file_path() {
    let csv_content = "id,name\n100,Grace\n200,Heidi";
    // ヘルパー関数でセットアップ。デフォルトファイル名 "students.csv" を使用
    let (dir, _default_path) = setup_test_env("students.csv", csv_content); // 今度は dir を使う

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.current_dir(dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("正担当: 100 Grace").or(
                predicate::str::contains("正担当: 200 Heidi")))
        .stdout(predicate::str::contains("副担当: 100 Grace").or(
                predicate::str::contains("副担当: 200 Heidi")));
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
    let dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.current_dir(dir.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Could not open file"));
}

#[test]
fn test_error_empty_file_header_only() {
    let csv_content = "id,name\n"; // ヘッダーのみ
    // ヘルパー関数でセットアップ
    let (_dir, file_path) = setup_test_env("empty_header.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: The student list"));
}

#[test]
fn test_error_empty_file_no_header() {
    let csv_content = ""; // 完全に空
    // ヘルパー関数でセットアップ
    let (_dir, file_path) = setup_test_env("empty_no_header.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: The student list"));
}

#[test]
fn test_error_one_student() {
    let csv_content = "id,name\n1,Alice";
    // ヘルパー関数でセットアップ
    let (_dir, file_path) = setup_test_env("one_student.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Not enough students"));
}

#[test]
fn test_error_csv_format_invalid_delimiter() {
    let csv_content = "id;name\n1;Alice\n2;Bob"; // セミコロン区切り
    // ヘルパー関数でセットアップ
    let (_dir, file_path) = setup_test_env("invalid_delimiter.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Failed to parse CSV file"));
}

#[test]
fn test_error_csv_format_wrong_columns() {
    let csv_content = "id,name,extra\n1,Alice,foo\n2,Bob,bar";
    // ヘルパー関数でセットアップ
    let (_dir, file_path) = setup_test_env("wrong_columns.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Failed to parse CSV file"));
}

#[test]
fn test_positional_file_argument() {
    let csv_content = "id,name\npos1,PositionalArgUser1\npos2,PositionalArgUser2";
    // ヘルパー関数でセットアップ
    let (_dir, file_path) = setup_test_env("positional_test.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg(file_path.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(
            predicate::str::contains("正担当: pos1 PositionalArgUser1")
                .or(predicate::str::contains("正担当: pos2 PositionalArgUser2")),
        )
        .stdout(
            predicate::str::contains("副担当: pos1 PositionalArgUser1")
                .or(predicate::str::contains("副担当: pos2 PositionalArgUser2")),
        );
}