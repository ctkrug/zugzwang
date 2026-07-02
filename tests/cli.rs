use std::process::Command;

#[test]
fn prints_starting_board_by_default() {
    let output = Command::new(env!("CARGO_BIN_EXE_zugzwang"))
        .output()
        .expect("failed to run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Zugzwang chess engine"));
    assert!(stdout.contains("R N B Q K B N R"));
}
