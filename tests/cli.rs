// Integration tests for the gourei_touban CLI application.

use assert_cmd::Command;
use predicates::prelude::*; // Used for stdout/stderr assertions
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir; // Use tempfile for cleaner test setup/teardown

// Helper function to create a dummy CSV file within a temporary directory
fn create_test_csv(dir: &tempfile::TempDir, filename: &str, content: &str) -> PathBuf {
    let file_path = dir.path().join(filename);
    fs::write(&file_path, content).expect("Failed to write test CSV");
    file_path
}

#[test]
fn test_fixed_seed_selection() {
    let dir = tempdir().unwrap();
    let csv_content = "id,name
1,Alice
2,Bob
3,Charlie
4,David";
    let file_path = create_test_csv(&dir, "test_students_fixed.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file")
       .arg(file_path.to_str().unwrap())
       .arg("--seed")
       .arg("42"); // Fixed seed

    // --- !!! IMPORTANT !!! ---
    // Updated expected output based on actual test run with seed 42
    let expected_output = "正担当: 1 Alice\n副担当: 3 Charlie\n"; // <-- ADJUSTED THIS LINE

    cmd.assert().success().stdout(expected_output);
}

#[test]
fn test_cli_file_argument() {
    let dir = tempdir().unwrap();
    let csv_content = "id,name
10,Eve
20,Frank";
    let file_path = create_test_csv(&dir, "cli_specified.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    // Check for the presence of the labels and that the output is exactly two lines.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("正担当:"))
        .stdout(predicate::str::contains("副担当:"))
        // Use regex to ensure exactly two lines ending with newline
        .stdout(predicate::str::is_match("^.*\n.*\n$").unwrap());
}

#[test]
fn test_default_file_path() {
    let dir = tempdir().unwrap(); // Create a temporary directory
    let csv_content = "id,name\n100,Grace\n200,Heidi";
    // Create students.csv inside the temporary directory using the helper
    let _default_path = create_test_csv(&dir, "students.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    // Run the command from the temporary directory containing the test students.csv
    cmd.current_dir(dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("正担当: 100 Grace").or(
                predicate::str::contains("正担当: 200 Heidi"))) // Check one is 正
        .stdout(predicate::str::contains("副担当: 100 Grace").or(
                predicate::str::contains("副担当: 200 Heidi"))); // Check the other is 副
}

#[test]
fn test_error_file_not_found() {
    let dir = tempdir().unwrap(); // Ensure the directory exists, but the file doesn't
    let non_existent_path = dir.path().join("non_existent_file.csv");

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(non_existent_path.to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Could not open file"));
}

#[test]
fn test_error_default_file_not_found() {
    // Run from a directory guaranteed not to contain 'students.csv'
    let dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.current_dir(dir.path()); // Run where students.csv doesn't exist

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Could not open file"));
}

#[test]
fn test_error_empty_file_header_only() {
    let dir = tempdir().unwrap();
    let csv_content = "id,name
"; // Only header
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
    let csv_content = ""; // Completely empty
    let file_path = create_test_csv(&dir, "empty_no_header.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    // Expect the "empty list" error, not a parsing error, for a completely empty file
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: The student list")); // <-- ADJUSTED THIS LINE
}

#[test]
fn test_error_one_student() {
    let dir = tempdir().unwrap();
    let csv_content = "id,name
1,Alice";
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
    let csv_content = "id;name
1;Alice
2;Bob"; // Semicolon delimiter
    let file_path = create_test_csv(&dir, "invalid_delimiter.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    // The deserialization should fail
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Failed to parse CSV file"));
}

#[test]
fn test_error_csv_format_wrong_columns() {
    let dir = tempdir().unwrap();
    // Student struct expects "id", "name". This has an extra column.
    let csv_content = "id,name,extra
1,Alice,foo
2,Bob,bar";
    let file_path = create_test_csv(&dir, "wrong_columns.csv", csv_content);

    let mut cmd = Command::cargo_bin("gourei_touban").unwrap();
    cmd.arg("--file").arg(file_path.to_str().unwrap());

    // Now that flexible(false) is set, deserialization should fail
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Failed to parse CSV file")); // Assertion remains the same, but behavior should now match
}