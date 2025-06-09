use fleetcore::{BaseInputs, BaseJournal};
use risc0_zkvm::guest::env;
use risc0_zkvm::Digest;
use sha2::{Digest as ShaDigest, Sha256};

fn validate_board(board: &[u8]) -> bool {
    const BOAT_SIZES: [(usize, usize); 5] = [
        (5, 1), // Carrier: 1 boat, size 5
        (4, 1), // Battleship: 1 boat, size 4
        (3, 1), // Destroyer: 1 boat, size 3
        (2, 2), // Cruiser: 2 boats, size 2
        (1, 2), // Submarine: 2 boats, size 1
    ];

    // Print the received board values (cell coordinates) for debugging
    eprintln!("Board values: {:?}", board);
    
    // Create a 10x10 boolean grid to represent the board
    let mut grid = [[false; 10]; 10];
    
    // Mark occupied cells based on the coordinates provided
    for &pos in board {
        let y = (pos / 10) as usize; // Row (first digit)
        let x = (pos % 10) as usize; // Column (second digit)
        
        if y < 10 && x < 10 {
            grid[y][x] = true;
        } else {
            eprintln!("Invalid position: {}", pos);
            return false;
        }
    }
    
    // Print the reconstructed grid for debugging
    let mut grid_str = String::new();
    for y in 0..10 {
        grid_str.push_str(&format!("{}: ", y));
        for x in 0..10 {
            grid_str.push_str(if grid[y][x] { "1 " } else { "0 " });
        }
        grid_str.push('\n');
    }
    eprintln!("Reconstructed grid:\n{}", grid_str);
    
    // Count total ship cells
    let mut ship_count = 0;
    for y in 0..10 {
        for x in 0..10 {
            if grid[y][x] {
                ship_count += 1;
            }
        }
    }
    
    eprintln!("Number of ship cells: {}", ship_count);

    // Detect ships by finding connected components
    let mut visited = [[false; 10]; 10];
    let mut ship_sizes = Vec::new();

    for y in 0..10 {
        for x in 0..10 {
            if grid[y][x] && !visited[y][x] {
                // Found a new ship
                let mut size = 0;
                let mut queue = vec![(y, x)];
                visited[y][x] = true;
                
                // Check if it's a valid ship formation (in a line)
                let mut min_x = x;
                let mut max_x = x;
                let mut min_y = y;
                let mut max_y = y;
                
                while let Some((cy, cx)) = queue.pop() {
                    size += 1;
                    min_x = min_x.min(cx);
                    max_x = max_x.max(cx);
                    min_y = min_y.min(cy);
                    max_y = max_y.max(cy);
                    
                    // Check adjacent cells (only horizontally and vertically)
                    let neighbors = [
                        (cy > 0, cy - 1, cx),      // up
                        (cy < 9, cy + 1, cx),      // down
                        (cx > 0, cy, cx - 1),      // left
                        (cx < 9, cy, cx + 1),      // right
                    ];
                    
                    for (valid, ny, nx) in neighbors {
                        if valid && grid[ny][nx] && !visited[ny][nx] {
                            queue.push((ny, nx));
                            visited[ny][nx] = true;
                        }
                    }
                }
                
                // Validate that the ship is a straight line
                let is_horizontal = min_y == max_y;
                let is_vertical = min_x == max_x;
                
                if !is_horizontal && !is_vertical {
                    eprintln!("Found a ship that is not a straight line: ({},{})-({},{}), size: {}", 
                              min_x, min_y, max_x, max_y, size);
                    return false;
                }
                
                // For horizontal ships, check that all cells between min_x and max_x are part of the ship
                if is_horizontal {
                    for cx in min_x..=max_x {
                        if !grid[min_y][cx] {
                            eprintln!("Horizontal ship has a gap at ({},{})", cx, min_y);
                            return false;
                        }
                    }
                }
                
                // For vertical ships, check that all cells between min_y and max_y are part of the ship
                if is_vertical {
                    for cy in min_y..=max_y {
                        if !grid[cy][min_x] {
                            eprintln!("Vertical ship has a gap at ({},{})", min_x, cy);
                            return false;
                        }
                    }
                }
                
                // Ship size validation
                if size > 5 {
                    eprintln!("Ship is too large: size {}", size);
                    return false;
                }
                
                ship_sizes.push(size);
            }
        }
    }
    
    eprintln!("Found {} ships with sizes: {:?}", ship_sizes.len(), ship_sizes);

    // Check that we have the correct number of ships of each size
    let mut size_counts = [0; 6]; // Index 0 unused, sizes 1-5 tracked
    for &size in &ship_sizes {
        size_counts[size] += 1;
    }
    
    for &(size, expected_count) in &BOAT_SIZES {
        if size_counts[size] != expected_count {
            eprintln!("Expected {} ships of size {}, but found {}", 
                     expected_count, size, size_counts[size]);
            return false;
        }
    }

    // Check if ships are too close (touching diagonally or adjacent)
    let mut proximity_grid = [[false; 10]; 10];
    
    // Mark all cells occupied by ships
    for y in 0..10 {
        for x in 0..10 {
            proximity_grid[y][x] = grid[y][x];
        }
    }
    
    // Mark "proximity zones" around ships
    for y in 0..10 {
        for x in 0..10 {
            if grid[y][x] {
                // Mark all 8 surrounding cells as "proximity"
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dy == 0 && dx == 0 {
                            continue; // Skip the cell itself
                        }
                        
                        let ny = y as i32 + dy;
                        let nx = x as i32 + dx;
                        
                        if ny >= 0 && ny < 10 && nx >= 0 && nx < 10 {
                            proximity_grid[ny as usize][nx as usize] = true;
                        }
                    }
                }
            }
        }
    }
    
    // Check if each ship's cells are all connected
    // This is a separate check from the proximity check
    for y in 0..10 {
        for x in 0..10 {
            // Skip empty cells
            if !grid[y][x] {
                continue;
            }
            
            // Count the number of adjacent ship cells
            let mut adjacent_count = 0;
            if y > 0 && grid[y-1][x] { adjacent_count += 1; }
            if y < 9 && grid[y+1][x] { adjacent_count += 1; }
            if x > 0 && grid[y][x-1] { adjacent_count += 1; }
            if x < 9 && grid[y][x+1] { adjacent_count += 1; }
            
            // Ships should form straight lines, so each cell should have:
            // - 0 or 1 adjacent cells if it's at an end of a ship
            // - 2 adjacent cells if it's in the middle of a ship
            if adjacent_count > 2 {
                eprintln!("Ship cell at ({},{}) has too many adjacent cells: {}", x, y, adjacent_count);
                return false;
            }
        }
    }
    
    // Calculate how many cells should be covered by all ships
    let expected_total_cells = BOAT_SIZES.iter()
        .map(|&(size, count)| size * count)
        .sum::<usize>();
    
    if ship_count != expected_total_cells {
        eprintln!("Expected {} total ship cells, but found {}", expected_total_cells, ship_count);
        return false;
    }

    // We've passed all validations
    eprintln!("Board layout is valid!");
    true
}

fn main() {
    // read the input
    let input: BaseInputs = env::read();

    // Extract variables
    let board = input.board.clone();
    let random = input.random.clone();

    // Validate the board
    let is_valid = validate_board(&board);

    assert!(is_valid);

    // Hash the random nonce and the board together as evidence
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

    env::commit(&output);
}
