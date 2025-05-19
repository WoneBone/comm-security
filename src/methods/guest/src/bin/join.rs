use fleetcore::{BaseInputs, BaseJournal};
use risc0_zkvm::guest::env;
use risc0_zkvm::Digest;
use sha2::{Digest as ShaDigest, Sha256};

fn main() {
    // read the input
    let input: BaseInputs = env::read();

    // Hash the random nonce and the board together as evidence
    let mut hasher = Sha256::new();
    hasher.update(input.random.as_bytes());
    hasher.update(&input.board);
    let hash_result = hasher.finalize();
    let board_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Fill the output journal with the required fields
    let output = BaseJournal {
        gameid: input.gameid,
        fleetid: input.fleetid,
        board: board_digest,
    };

    env::commit(&output);
}