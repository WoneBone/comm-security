use fleetcore::{FireInputs, FireJournal};
use risc0_zkvm::guest::env;
use risc0_zkvm::Digest;
use sha2::{Digest as _, Sha256};

fn main() {
    // Read the input
    let input: FireInputs = env::read();

    // Verify that at least one ship is not sunk
    let has_unsunk_ship = input.board.iter().any(|&cell| cell != 0);

    // Hash your board state for evidence
    let mut hasher = Sha256::new();
    hasher.update(&input.board);
    let hash_result = hasher.finalize();
    let board_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Fill the output journal with the required fields
    let output = FireJournal {
        gameid: input.gameid,
        fleetid: input.fleetid,
        targetfleet: input.targetfleet,
        board: board_digest,
        has_unsunk_ship,
    };

    // Commit the output to the journal
    env::commit(&output);
}
