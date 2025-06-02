use fleetcore::{BaseInputs, BaseJournal};
use risc0_zkvm::guest::env;
use risc0_zkvm::Digest;
use sha2::{Digest as ShaDigest, Sha256};

fn validate_board(board: &[u8]) -> bool {
    const BOARD_SIZE: usize = 100;
    const BOAT_SIZES: [(u8, usize); 5] = [
        (5, 1), // Carrier: 1 boat, size 5
        (4, 1), // Battleship: 1 boat, size 4
        (3, 1), // Destroyer: 1 boat, size 3
        (2, 2), // Cruiser: 2 boats, size 2
        (1, 2), // Submarine: 2 boats, size 1
    ];

    if board.len() != BOARD_SIZE {
        return false;
    }

    let mut boat_counts = [0; 6]; // Index 0 unused, sizes 1-5 tracked

    for y in 0..10 {
        for x in 0..10 {
            let idx = y * 10 + x;
            if board[idx] > 0 {
                let size = board[idx] as usize;
                if size > 5 {
                    return false;
                }
                boat_counts[size] += 1;

                // Check horizontal and vertical continuity
                if x + size <= 10 && board[idx..idx + size].iter().all(|&v| v == board[idx]) {
                    for i in 0..size {
                        board[idx + i] = 0; // Mark as visited
                    }
                } else if y + size <= 10 && (0..size).all(|i| board[idx + i * 10] == board[idx]) {
                    for i in 0..size {
                        board[idx + i * 10] = 0; // Mark as visited
                    }
                } else {
                    return false;
                }
            }
        }
    }

    for &(size, count) in &BOAT_SIZES {
        if boat_counts[size as usize] != count {
            return false;
        }
    }

    true
}

fn main() {
    // read the input
    let input: BaseInputs = env::read();

    // Validate the board and include the result in the journal
    let is_valid = validate_board(&input.board);

    assert!(is_valid);

    // Hash the random nonce and the board together as evidence
    let mut hasher = Sha256::new();
    hasher.update(input.random.as_bytes());
    hasher.update(&input.board);
    let hash_result = hasher.finalize();
    let board_digest = Digest::try_from(hash_result.as_slice()).expect("Digest conversion failed");

    // Fill the output journal with the required fields
    let output = BaseJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        board: board_digest,
        //is_valid,
    };

    env::commit(&output);
}
