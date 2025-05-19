use fleetcore::{BaseInputs, BaseJournal};
use risc0_zkvm::guest::env;
//use risc0_zkvm::Digest;
//use sha2::{Digest as _, Sha256};

fn main() {
    // read the input
    let input: BaseInputs = env::read();

    // Fill the output journal with the required fields
    let output = BaseJournal {
        gameid: input.gameid,
        fleetid: input.fleetid,
        board: Default::default(), // Not needed for wave, but required by struct
    };

    env::commit(&output);
}
