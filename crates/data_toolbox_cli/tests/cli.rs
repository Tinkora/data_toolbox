use std::fs;
use std::fs::File;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMPORARY_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_data-toolbox")
}

fn run(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(binary())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("CLI starts");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(input.as_bytes())
        .expect("stdin accepts input");
    child.wait_with_output().expect("CLI exits")
}

fn run_without_input(args: &[&str]) -> Output {
    Command::new(binary())
        .args(args)
        .output()
        .expect("CLI exits")
}

fn temporary_path() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let sequence = TEMPORARY_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "tinkora-data-toolbox-{}-{nonce}-{sequence}.csv",
        std::process::id(),
    ))
}

#[test]
fn inspect_reads_stdin_and_writes_versioned_json() {
    let output = run(
        &["inspect", "--delimiter", "comma", "--headers", "present"],
        "name,role\nAlice,owner\n",
    );

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON result");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["headers"], serde_json::json!(["name", "role"]));
    assert_eq!(value["row_count"], 1);
}

#[test]
fn inspect_reads_an_explicit_file() {
    let path = temporary_path();
    fs::write(&path, "name,role\nAlice,owner\n").expect("fixture written");
    let output = Command::new(binary())
        .args(["inspect", "--delimiter", "comma"])
        .arg(&path)
        .output()
        .expect("CLI exits");
    fs::remove_file(path).expect("fixture removed");

    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON result");
    assert_eq!(
        value["preview_rows"][0],
        serde_json::json!(["Alice", "owner"])
    );
}

#[test]
fn convert_writes_only_the_requested_content() {
    let output = run(
        &[
            "convert",
            "--to",
            "tsv",
            "--delimiter",
            "comma",
            "--headers",
            "present",
        ],
        "name,role\nAlice,owner\n",
    );

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, b"name\trole\r\nAlice\towner\r\n");
}

#[test]
fn malformed_input_uses_json_stderr_and_nonzero_exit() {
    let output = run(&["inspect", "--delimiter", "comma"], "a,b\n\"broken\n");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("valid JSON error");
    assert_eq!(value["code"], "INVALID_CSV");
    assert!(value["message"].is_string());
}

#[test]
fn invalid_arguments_use_a_stable_json_error() {
    let output = run_without_input(&["convert", "--to", "yaml"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("valid JSON error");
    assert_eq!(value["code"], "INVALID_OPTIONS");
}

#[test]
fn repeated_options_are_rejected_instead_of_silently_overridden() {
    let output = run_without_input(&["inspect", "--delimiter", "comma", "--delimiter", "tab"]);

    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("valid JSON error");
    assert_eq!(value["code"], "INVALID_OPTIONS");
}

#[test]
fn oversized_regular_files_are_rejected_before_reading() {
    let path = temporary_path();
    let file = File::create(&path).expect("fixture created");
    file.set_len(10 * 1024 * 1024 + 1).expect("fixture resized");
    let output = Command::new(binary())
        .args(["inspect", "--delimiter", "comma"])
        .arg(&path)
        .output()
        .expect("CLI exits");
    fs::remove_file(path).expect("fixture removed");

    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("valid JSON error");
    assert_eq!(value["code"], "INPUT_TOO_LARGE");
}

#[test]
fn non_utf8_input_has_a_stable_error() {
    let mut child = Command::new(binary())
        .args(["inspect", "--delimiter", "comma"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("CLI starts");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(&[0xff])
        .expect("stdin accepts input");
    let output = child.wait_with_output().expect("CLI exits");

    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("valid JSON error");
    assert_eq!(value["code"], "INVALID_UTF8");
}
