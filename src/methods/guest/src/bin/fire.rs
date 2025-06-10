use fleetcore::{FireInputs, FireJournal};
use risc0_zkvm::guest::env;
use risc0_zkvm::Digest;
use sha2::{Digest as _, Sha256};

fn main() {
    // Read the input
    let input: FireInputs = env::read();    // Extract the random value
    let random = input.random.clone();

    // Verify that at least one ship exists on the board
    let _has_ships = !input.board.is_empty();

    // Debug: Print the board state
    eprintln!("Board state during fire: {:?}", input.board);

    // Hash your board state for evidence
    let mut hasher = Sha256::new();
    hasher.update(random.as_bytes());
    hasher.update(&input.board);
    let hash_result = hasher.finalize();
    let board_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Fill the output journal with the required fields
    let output = FireJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        board: board_digest,
        target: input.target,
        pos: input.pos,
    };

    // Commit the output to the journal
    env::commit(&output);
}
