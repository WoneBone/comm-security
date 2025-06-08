use fleetcore::{BaseInputs, BaseJournal};
use risc0_zkvm::guest::env;
use risc0_zkvm::Digest;
use sha2::{Digest as _, Sha256};

fn main() {
    // Read the input
    let input: BaseInputs = env::read();

    // Extract variables to match those in game_actions.rs
    let board = input.board.clone();
    let random = input.random.clone();

    // In a zero-knowledge implementation, we verify that we have at least one ship remaining (our fleet is not sunk) as part of claiming victory
    let has_unsunk_ship = board.iter().any(|&cell| cell != 0);
    assert!(has_unsunk_ship, "Cannot claim victory with a completely sunk fleet");

    // Hash your board state as evidence that your fleet still exists
    let mut hasher = Sha256::new();
    hasher.update(random.as_bytes());
    hasher.update(&board);
    let hash_result = hasher.finalize();
    let board_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Fill the output journal with the required fields
    let output = BaseJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        board: board_digest,
    };

    // Commit the output to the journal
    env::commit(&output);
}
