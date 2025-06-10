use fleetcore::{ReportInputs, ReportJournal};
use risc0_zkvm::guest::env;
use risc0_zkvm::Digest;
use sha2::{Digest as ShaDigest, Sha256};

fn main() {
    // read the input
    let input: ReportInputs = env::read();

    // Extract values that match game_actions.rs variable names
    let board = input.board.clone();
    let random = input.random.clone();
    let pos = input.pos;

    // In game_actions.rs, the report value would be passed separately
    let _report = input.report.clone();

    // Hash the current board
    let mut hasher = Sha256::new();
    hasher.update(random.as_bytes());
    hasher.update(&board);
    let hash_result = hasher.finalize();
    let old_board_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Check if the shot is a hit or miss based on board containing the position value
    let is_hit = board.contains(&pos);
    let report = if is_hit { "Hit".to_string() } else { "Miss".to_string() };    // Compare _report and report
    if _report != report {
        panic!("Provided report does not match actual result");
    }

    // Debug: Print board before modification
    eprintln!("Board before hit processing: {:?}", board);

    // Alter the board if it was a hit
    let mut altered_board = board.clone();
    if is_hit {
        // Remove the hit position from the board
        if let Some(pos_index) = altered_board.iter().position(|&p| p == pos) {
            altered_board.remove(pos_index);
        }
    }

    // Debug: Print board after modification
    eprintln!("Board after hit processing: {:?}", altered_board);

    // Hash the altered board
    let mut hasher2 = Sha256::new();
    hasher2.update(random.as_bytes());
    hasher2.update(&altered_board);
    let hash_result = hasher2.finalize();
    let new_board_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Fill the output journal with the required fields
    let output = ReportJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        report,
        pos: input.pos,
        board: old_board_digest,
        next_board: new_board_digest,
    };

    env::commit(&output);
}
