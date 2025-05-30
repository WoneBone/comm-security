use fleetcore::{FireInputs, ReportJournal};
use risc0_zkvm::guest::env;
use risc0_zkvm::Digest;
use sha2::{Digest as ShaDigest, Sha256};

fn main() {
    // read the input
    let input: FireInputs = env::read();

    // Compute the hash h using the random nonce and fleetid
    let mut hasher = Sha256::new();
    hasher.update(input.random.as_bytes());
    hasher.update(input.fleetid.as_bytes());
    let hash_result = hasher.finalize();
    let fleet_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Use the provided report_ value directly
    let _report = input._report.clone();
    let board = input.board.clone();
    let gameid = input.gameid;
    let fleetid = input.fleetid;
    let x = input.pos.0;
    let y = input.pos.1;

    // Hash the current board
    let mut hasher = Sha256::new();
    hasher.update(&board);
    let hash_result = hasher.finalize();
    let old_board_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Check if the shot is a hit or miss
    let is_hit = board.get((y * 10 + x) as usize).copied().unwrap_or(0) != 0;
    let report = if is_hit { "hit".to_string() } else { "miss".to_string() };

    // Compare _report and report
    if _report != report {
        panic!("Provided report does not match actual result");
    }

    // Alter the board if it was a hit
    let mut altered_board = board.clone();
    if is_hit {
        altered_board[(y * 10 + x) as usize] = 0; // Mark the position as hit
    }

    // Hash the altered board
    let mut hasher = Sha256::new();
    hasher.update(&altered_board);
    let hash_result = hasher.finalize();
    let new_board_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Fill the output journal with the required fields
    let output = ReportJournal {
        gameid,
        fleetid,
        report,
        pos: (x, y),
        board: old_board_digest,
        next_board: new_board_digest,
    };

    env::commit(&output);
}
