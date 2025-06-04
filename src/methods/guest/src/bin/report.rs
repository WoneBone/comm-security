use fleetcore::{FireInputs, ReportJournal};
use risc0_zkvm::guest::env;
use risc0_zkvm::Digest;
use sha2::{Digest as ShaDigest, Sha256};

fn main() {
    // read the input
    let input: FireInputs = env::read();

    // Extract values that match game_actions.rs variable names
    let board = input.board.clone();
    let random = input.random.clone();
    let _report = input.report_.clone();

    // Extract x and y from the pos field
    let x = (input.pos >> 4) & 0xF; // Higher 4 bits
    let y = input.pos & 0xF;        // Lower 4 bits

    // Hash the current board
    let mut hasher = Sha256::new();
    hasher.update(&board);
    let hash_result = hasher.finalize();
    let old_board_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Check if the shot is a hit or miss based on board and coordinates
    let index = (y * 10 + x) as usize;
    let is_hit = board.get(index).copied().unwrap_or(0) != 0;
    let report = if is_hit { "hit".to_string() } else { "miss".to_string() };

    // Compare _report and report
    if _report != report {
        panic!("Provided report does not match actual result");
    }

    // Alter the board if it was a hit
    let mut altered_board = board.clone();
    if is_hit {
        altered_board[index] = 0; // Mark the position as hit
    }

    // Hash the altered board
    let mut hasher = Sha256::new();
    hasher.update(&altered_board);
    let hash_result = hasher.finalize();
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
