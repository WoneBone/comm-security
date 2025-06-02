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
    hasher.update(input.fleet.as_bytes());
    let hash_result = hasher.finalize();
    let fleet_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Use the provided report_ value directly
    //let report = input.report_.clone();
    let report = false;

    //TODO: VERIFICAR O REPORT

    // Fill the output journal with the required fields
    let output = ReportJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        report,
        pos: input.pos,
        board: fleet_digest,
        next_board: fleet_digest.clone(),
    };

    env::commit(&output);
}
