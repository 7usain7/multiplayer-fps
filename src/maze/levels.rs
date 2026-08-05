use super::map::{MazeMap, TileType};

/// Retrieve a pre-defined level by its number (1, 2, or 3).
/// Returns Level 1 as fallback for out-of-range numbers.
pub fn get_level(level_number: u8) -> MazeMap {
    match level_number {
        1 => level_1(),
        2 => level_2(),
        3 => level_3(),
        _ => level_1(),
    }
}

/// Helper function to convert ASCII map representation into a `MazeMap`
/// '#' = Wall, ' ' = Empty, 'S' = Spawn, 'E' = Exit
pub fn parse_ascii_map(ascii: &[&str]) -> MazeMap {
    let height = ascii.len();
    let width = ascii.iter().map(|row| row.len()).max().unwrap_or(0);

    let mut map = MazeMap::new(width, height, TileType::Wall);

    for (y, row) in ascii.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let tile = match ch {
                '#' => TileType::Wall,
                'S' => TileType::Spawn,
                'E' => TileType::Exit,
                _ => TileType::Empty,
            };
            map.set_tile(x, y, tile);
        }
    }
    map
}

/// Level 1: Simple layout with open central areas
fn level_1() -> MazeMap {
    let ascii = vec![
        "##########",
        "#S  #   S#",
        "# # # ## #",
        "# #    # #",
        "#  ## #  #",
        "# #   ## #",
        "# # #    #",
        "#   # ## #",
        "#S     #E#",
        "##########",
    ];
    parse_ascii_map(&ascii)
}

/// Level 2: Medium layout with winding corridors
fn level_2() -> MazeMap {
    let ascii = vec![
        "##############",
        "#S   #      S#",
        "#### # #######",
        "#    # #   # #",
        "# #### # # # #",
        "# #    # #   #",
        "# # #### #####",
        "# #   S  #   #",
        "# # #### # # #",
        "# #      # # #",
        "# # ###### # #",
        "#S    S     E#",
        "##############",
    ];
    parse_ascii_map(&ascii)
}

/// Level 3: Complex layout with dead ends and maze corridors
fn level_3() -> MazeMap {
    let ascii = vec![
        "##################",
        "#S #   #   #    S#",
        "#  # # # # # ### #",
        "# ## #   # #   # #",
        "#    # ### ### # #",
        "# ## #   #   #   #",
        "#    # # ##### ###",
        "# #### # #   #   #",
        "#S     # # # ### #",
        "# ###### # #   # #",
        "#   #    # ### # #",
        "# # # ####   # # #",
        "# # # #  S## # # #",
        "# # # # ##   #   #",
        "# # # #    ##### #",
        "# # # ######  S  #",
        "#S  #          E #",
        "##################",
    ];
    parse_ascii_map(&ascii)
}
