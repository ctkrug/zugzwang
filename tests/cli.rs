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

#[test]
fn uci_mode_replies_with_a_legal_move_to_a_zero_movetime() {
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
        .write_all(b"position startpos\ngo movetime 0\nquit\n")
        .unwrap();

    let output = child.wait_with_output().expect("engine did not exit");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let bestmove_line = stdout
        .lines()
        .find(|line| line.starts_with("bestmove "))
        .expect("no bestmove line in output");
    // "0000" is the null move, which most GUIs read as a forfeit; a fresh
    // starting position always has legal moves, so it must never appear.
    assert_ne!(bestmove_line, "bestmove 0000");
}

#[test]
fn uci_mode_ignores_unknown_commands_without_crashing() {
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
        .write_all(b"uci\nbanana\nisready\nquit\n")
        .unwrap();

    let output = child.wait_with_output().expect("engine did not exit");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("uciok"));
    assert!(stdout.contains("readyok"));
}

#[test]
fn uci_mode_keeps_the_prior_position_after_a_malformed_position_command() {
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
        .write_all(b"position startpos moves e2e4\nposition fen not-a-fen\ngo movetime 50\nquit\n")
        .unwrap();

    let output = child.wait_with_output().expect("engine did not exit");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // A rejected `position fen` must not clobber the board with an empty
    // or partial position — the engine should still find a legal reply
    // from the position set by the last valid `position` command.
    assert!(stdout
        .lines()
        .any(|line| line.starts_with("bestmove ") && !line.contains("0000")));
}

#[test]
fn perft_command_prints_known_node_counts_for_the_starting_position() {
    let output = Command::new(env!("CARGO_BIN_EXE_zugzwang"))
        .args(["perft", "3"])
        .output()
        .expect("failed to run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        vec!["perft(1) = 20", "perft(2) = 400", "perft(3) = 8902"]
    );
}

#[test]
fn perft_command_accepts_an_arbitrary_fen() {
    let output = Command::new(env!("CARGO_BIN_EXE_zugzwang"))
        .args([
            "perft",
            "1",
            "4k3/8/8/3p4/4P3/8/8/4K3",
            "w",
            "-",
            "-",
            "0",
            "1",
        ])
        .output()
        .expect("failed to run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // From this position: 5 king moves plus 2 pawn moves (push to e5,
    // capture on d5) = 7.
    assert_eq!(stdout.trim(), "perft(1) = 7");
}

#[test]
fn perft_command_rejects_a_malformed_fen() {
    let output = Command::new(env!("CARGO_BIN_EXE_zugzwang"))
        .args(["perft", "1", "not-a-fen"])
        .output()
        .expect("failed to run binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid FEN"));
}

#[test]
fn perft_command_rejects_a_missing_depth() {
    let output = Command::new(env!("CARGO_BIN_EXE_zugzwang"))
        .args(["perft"])
        .output()
        .expect("failed to run binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("usage"));
}

#[test]
fn perft_command_rejects_a_non_numeric_depth() {
    let output = Command::new(env!("CARGO_BIN_EXE_zugzwang"))
        .args(["perft", "deep"])
        .output()
        .expect("failed to run binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("usage"));
}

#[test]
fn play_mode_rejects_an_illegal_move_then_plays_a_legal_one() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zugzwang"))
        .arg("play")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run binary");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"e2e5\ne2e4\nquit\n")
        .unwrap();

    let output = child.wait_with_output().expect("engine did not exit");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("illegal move: e2e5"));
    assert!(stdout.contains("Engine plays "));
}
