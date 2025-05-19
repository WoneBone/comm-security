use fleetcore::{FireInputs, ReportJournal};
use risc0_zkvm::guest::env;
use risc0_zkvm::Digest;
use sha2::{Digest as ShaDigest, Sha256};

fn main() {
    // read the input
    let input: FireInputs = env::read();

    // Hash the random nonce and the board together as evidence
    let mut hasher = Sha256::new();
    hasher.update(input.random.as_bytes());
    hasher.update(&input.board);
    let hash_result = hasher.finalize();
    let board_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Check if the shot is a hit or miss
    let is_hit = input.board.get(input.pos as usize).copied().unwrap_or(0) != 0;
    let report = if is_hit { "hit".to_string() } else { "miss".to_string() };

    // For simplicity, next_board is the same as board (unless you want to update it)
    let next_board_digest = board_digest.clone();

    // Fill the output journal
    let output = ReportJournal {
        gameid: input.gameid,
        fleetid: input.fleetid,
        report,
        pos: input.pos,
        board: board_digest,
        next_board: next_board_digest,
    };

    env::commit(&output);
}
