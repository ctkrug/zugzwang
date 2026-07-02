use std::io::Write;
use std::process::{Command, Stdio};

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

#[test]
fn uci_mode_answers_go_with_a_legal_bestmove() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zugzwang"))
        .arg("uci")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run binary");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"uci\nposition startpos moves e2e4 e7e5\ngo\nquit\n")
        .unwrap();

    let output = child.wait_with_output().expect("engine did not exit");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("uciok"));

    let bestmove_line = stdout
        .lines()
        .find(|line| line.starts_with("bestmove "))
        .expect("no bestmove line in output");
    let mv = bestmove_line.strip_prefix("bestmove ").unwrap();
    assert_eq!(mv.len(), 4);
    let mut chars = mv.chars();
    assert!(chars.next().unwrap().is_ascii_lowercase());
    assert!(chars.next().unwrap().is_ascii_digit());
}
