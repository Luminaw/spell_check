use std::process::Command;
use std::path::PathBuf;

fn get_bin_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target/release/spell_check.exe");
    path
}

#[test]
fn test_cli_check_proj1() {
    let bin = get_bin_path();
    let mut cmd = Command::new(bin);
    cmd.arg("check").arg(".");
    
    // Set current dir to proj1
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/proj1");
    cmd.current_dir(path);

    let output = cmd.output().expect("failed to execute process");

    // We expect exit code 1 because there are errors (referance, occurance)
    assert!(!output.status.success(), "Process should have failed due to spelling errors");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("referance"), "Stdout did not contain 'referance': {}", stdout);
    assert!(stdout.contains("occurance"), "Stdout did not contain 'occurance': {}", stdout);
    assert!(!stdout.contains("mispelled"), "Stdout contained 'mispelled' unexpectedly: {}", stdout);
    assert!(!stdout.contains("garantee"), "Stdout contained 'garantee' unexpectedly: {}", stdout);
}

#[test]
fn test_cli_init() {
    let bin = get_bin_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let mut cmd = Command::new(bin);
    cmd.arg("init");
    cmd.current_dir(temp_dir.path());

    let output = cmd.output().expect("failed to execute process");
    assert!(output.status.success(), "Init command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let config_file = temp_dir.path().join("spellcheck.toml");
    assert!(config_file.exists());
}
