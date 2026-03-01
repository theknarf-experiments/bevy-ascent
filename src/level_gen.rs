use bevy::prelude::*;

#[derive(Resource)]
pub struct GeneratedFloors {
    pub floors: [String; 3],
}

const ASP_PROGRAM: &str = r#"
% --- Grid ---
dim(0..11).
border(X,Y) :- dim(X), dim(Y), X = 0.
border(X,Y) :- dim(X), dim(Y), X = 11.
border(X,Y) :- dim(X), dim(Y), Y = 0.
border(X,Y) :- dim(X), dim(Y), Y = 11.
wall(X,Y) :- border(X,Y).

% Interior cells
interior(X,Y) :- dim(X), dim(Y), X > 0, X < 11, Y > 0, Y < 11.

% Choice: place interior walls
{ iwall(X,Y) : interior(X,Y) }.

% Interior wall count between 6 and 16
:- #count{ X,Y : iwall(X,Y) } < 6.
:- #count{ X,Y : iwall(X,Y) } > 16.

% Clustering: each interior wall must have at least one adjacent wall
adj_wall(X,Y) :- iwall(X,Y), wall(X-1,Y).
adj_wall(X,Y) :- iwall(X,Y), wall(X+1,Y).
adj_wall(X,Y) :- iwall(X,Y), wall(X,Y-1).
adj_wall(X,Y) :- iwall(X,Y), wall(X,Y+1).
adj_wall(X,Y) :- iwall(X,Y), iwall(X-1,Y).
adj_wall(X,Y) :- iwall(X,Y), iwall(X+1,Y).
adj_wall(X,Y) :- iwall(X,Y), iwall(X,Y-1).
adj_wall(X,Y) :- iwall(X,Y), iwall(X,Y+1).
:- iwall(X,Y), not adj_wall(X,Y).

wall(X,Y) :- iwall(X,Y).
floor(X,Y) :- interior(X,Y), not wall(X,Y).

% --- Entity placement ---
% Floor 1 (tutorial): player, stairs_down, 2 goblins, 1 water puddle, 1 poison gas
1 { player(X,Y) : floor(X,Y), Y >= 6 } 1 :- floor_num(1).
1 { stairs_down(X,Y) : floor(X,Y), X >= 6 } 1 :- floor_num(1).
2 { goblin(X,Y) : floor(X,Y) } 2 :- floor_num(1).
1 { water(X,Y) : floor(X,Y) } 1 :- floor_num(1).
1 { poison_gas(X,Y) : floor(X,Y) } 1 :- floor_num(1).

% Floor 2 (elemental): fire imp, poison spider, explosive barrel, poison mushroom, spark
1 { stairs_up(X,Y) : floor(X,Y), X <= 5 } 1 :- floor_num(2).
1 { stairs_down(X,Y) : floor(X,Y), X >= 6 } 1 :- floor_num(2).
1 { fire_imp(X,Y) : floor(X,Y) } 1 :- floor_num(2).
1 { poison_spider(X,Y) : floor(X,Y) } 1 :- floor_num(2).
1 { explosive(X,Y) : floor(X,Y) } 1 :- floor_num(2).
1 { mushroom(X,Y) : floor(X,Y) } 1 :- floor_num(2).
1 { spark(X,Y) : floor(X,Y) } 1 :- floor_num(2).

% Floor 3 (boss): ice golem, shock eel, goblin, lightning rod, metal crate, water puddle
1 { stairs_up(X,Y) : floor(X,Y), X <= 5 } 1 :- floor_num(3).
1 { exit(X,Y) : floor(X,Y), Y >= 6 } 1 :- floor_num(3).
1 { ice_golem(X,Y) : floor(X,Y) } 1 :- floor_num(3).
1 { shock_eel(X,Y) : floor(X,Y) } 1 :- floor_num(3).
1 { goblin(X,Y) : floor(X,Y) } 1 :- floor_num(3).
1 { lightning_rod(X,Y) : floor(X,Y) } 1 :- floor_num(3).
1 { metal_crate(X,Y) : floor(X,Y) } 1 :- floor_num(3).
1 { water(X,Y) : floor(X,Y) } 1 :- floor_num(3).

% All floors: 2 torches, 2 barrels, 2-4 oil, 1-2 chests, 0-2 gold, 0-1 health potion
2 { torch(X,Y) : floor(X,Y) } 2.
2 { barrel(X,Y) : floor(X,Y) } 2.
2 { oil(X,Y) : floor(X,Y) } 4.
1 { chest(X,Y) : floor(X,Y) } 2.
0 { gold(X,Y) : floor(X,Y) } 2.
0 { health_pot(X,Y) : floor(X,Y) } 1.

% --- Integrity constraints ---
% Entity cell: a cell that has any entity
entity_cell(X,Y) :- player(X,Y).
entity_cell(X,Y) :- goblin(X,Y).
entity_cell(X,Y) :- torch(X,Y).
entity_cell(X,Y) :- barrel(X,Y).
entity_cell(X,Y) :- oil(X,Y).
entity_cell(X,Y) :- stairs_down(X,Y).
entity_cell(X,Y) :- stairs_up(X,Y).
entity_cell(X,Y) :- exit(X,Y).
entity_cell(X,Y) :- water(X,Y).
entity_cell(X,Y) :- poison_gas(X,Y).
entity_cell(X,Y) :- fire_imp(X,Y).
entity_cell(X,Y) :- poison_spider(X,Y).
entity_cell(X,Y) :- explosive(X,Y).
entity_cell(X,Y) :- mushroom(X,Y).
entity_cell(X,Y) :- spark(X,Y).
entity_cell(X,Y) :- ice_golem(X,Y).
entity_cell(X,Y) :- shock_eel(X,Y).
entity_cell(X,Y) :- lightning_rod(X,Y).
entity_cell(X,Y) :- metal_crate(X,Y).
entity_cell(X,Y) :- chest(X,Y).
entity_cell(X,Y) :- gold(X,Y).
entity_cell(X,Y) :- health_pot(X,Y).

% At most one entity per cell
entity_count(X,Y,N) :- floor(X,Y), N = #count{
    1 : player(X,Y);
    2 : goblin(X,Y);
    3 : torch(X,Y);
    4 : barrel(X,Y);
    5 : oil(X,Y);
    6 : stairs_down(X,Y);
    7 : stairs_up(X,Y);
    8 : exit(X,Y);
    9 : water(X,Y);
    10 : poison_gas(X,Y);
    11 : fire_imp(X,Y);
    12 : poison_spider(X,Y);
    13 : explosive(X,Y);
    14 : mushroom(X,Y);
    15 : spark(X,Y);
    16 : ice_golem(X,Y);
    17 : shock_eel(X,Y);
    18 : lightning_rod(X,Y);
    19 : metal_crate(X,Y);
    20 : chest(X,Y);
    21 : gold(X,Y);
    22 : health_pot(X,Y)
}.
:- entity_count(X,Y,N), N > 1.

% Enemies not adjacent to entry point
entry(X,Y) :- player(X,Y), floor_num(1).
entry(X,Y) :- stairs_up(X,Y), floor_num(2).
entry(X,Y) :- stairs_up(X,Y), floor_num(3).

enemy_at(X,Y) :- goblin(X,Y).
enemy_at(X,Y) :- fire_imp(X,Y).
enemy_at(X,Y) :- poison_spider(X,Y).
enemy_at(X,Y) :- ice_golem(X,Y).
enemy_at(X,Y) :- shock_eel(X,Y).

:- enemy_at(GX,GY), entry(EX,EY), |GX-EX| + |GY-EY| <= 1.

% --- Reachability ---
reachable(X,Y) :- entry(X,Y).
reachable(X2,Y) :- reachable(X1,Y), floor(X2,Y), X2 = X1+1.
reachable(X2,Y) :- reachable(X1,Y), floor(X2,Y), X2 = X1-1.
reachable(X,Y2) :- reachable(X,Y1), floor(X,Y2), Y2 = Y1+1.
reachable(X,Y2) :- reachable(X,Y1), floor(X,Y2), Y2 = Y1-1.

:- entity_cell(X,Y), not reachable(X,Y).

#show player/2.
#show goblin/2.
#show torch/2.
#show barrel/2.
#show oil/2.
#show stairs_down/2.
#show stairs_up/2.
#show exit/2.
#show wall/2.
#show water/2.
#show poison_gas/2.
#show fire_imp/2.
#show poison_spider/2.
#show explosive/2.
#show mushroom/2.
#show spark/2.
#show ice_golem/2.
#show shock_eel/2.
#show lightning_rod/2.
#show metal_crate/2.
#show chest/2.
#show gold/2.
#show health_pot/2.
"#;

#[derive(Debug)]
pub enum LevelGenError {
    ClingoError(String),
    NoModel,
}

impl std::fmt::Display for LevelGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LevelGenError::ClingoError(msg) => write!(f, "Clingo error: {}", msg),
            LevelGenError::NoModel => write!(f, "Clingo found no answer set"),
        }
    }
}

fn generate_floor(floor_num: u32, seed: u32) -> Result<String, LevelGenError> {
    let args = vec![
        format!("--rand-freq=0.5"),
        format!("--seed={}", seed),
    ];

    let mut ctl = clingo::control(args)
        .map_err(|e| LevelGenError::ClingoError(format!("{:?}", e)))?;

    let floor_fact = format!("floor_num({}).", floor_num);
    let full_program = format!("{}\n{}", ASP_PROGRAM, floor_fact);

    ctl.add("base", &[], &full_program)
        .map_err(|e| LevelGenError::ClingoError(format!("{:?}", e)))?;

    let parts = vec![clingo::Part::new("base", vec![])
        .map_err(|e| LevelGenError::ClingoError(format!("{:?}", e)))?];

    ctl.ground(&parts)
        .map_err(|e| LevelGenError::ClingoError(format!("{:?}", e)))?;

    let mut handle = ctl
        .solve(clingo::SolveMode::YIELD, &[])
        .map_err(|e| LevelGenError::ClingoError(format!("{:?}", e)))?;

    let model_atoms = loop {
        handle.resume()
            .map_err(|e| LevelGenError::ClingoError(format!("{:?}", e)))?;
        if let Some(model) = handle.model()
            .map_err(|e| LevelGenError::ClingoError(format!("{:?}", e)))?
        {
            let atoms = model
                .symbols(clingo::ShowType::SHOWN)
                .map_err(|e| LevelGenError::ClingoError(format!("{:?}", e)))?;
            // Collect atom strings before dropping the model
            let atom_strings: Vec<String> = atoms
                .iter()
                .map(|s| s.to_string())
                .collect();
            break atom_strings;
        } else {
            return Err(LevelGenError::NoModel);
        }
    };

    handle.close()
        .map_err(|e| LevelGenError::ClingoError(format!("{:?}", e)))?;

    Ok(build_grid_from_atoms(&model_atoms))
}

fn build_grid_from_atoms(atoms: &[String]) -> String {
    let mut grid = [['.'; 12]; 12];

    for atom in atoms {
        let atom = atom.trim();
        if let Some((name, x, y)) = parse_atom(atom) {
            if x < 12 && y < 12 {
                let ch = match name {
                    "wall" => '#',
                    "player" => '@',
                    "goblin" => 'g',
                    "torch" => 'T',
                    "barrel" => 'B',
                    "oil" => 'o',
                    "stairs_down" => '>',
                    "stairs_up" => '<',
                    "exit" => 'E',
                    "water" => '~',
                    "poison_gas" => 'p',
                    "fire_imp" => 'f',
                    "poison_spider" => 's',
                    "explosive" => 'X',
                    "mushroom" => 'm',
                    "spark" => 'z',
                    "ice_golem" => 'i',
                    "shock_eel" => 'e',
                    "lightning_rod" => '!',
                    "metal_crate" => 'M',
                    "chest" => 'C',
                    "gold" => '$',
                    "health_pot" => 'H',
                    _ => continue,
                };
                // Wall should only overwrite empty cells; entities take priority
                if ch == '#' && grid[y][x] != '.' {
                    continue;
                }
                grid[y][x] = ch;
            }
        }
    }

    let lines: Vec<String> = grid.iter().map(|row| row.iter().collect()).collect();
    lines.join("\n")
}

fn parse_atom(atom: &str) -> Option<(&str, usize, usize)> {
    let paren_start = atom.find('(')?;
    let paren_end = atom.find(')')?;
    let name = &atom[..paren_start];
    let args = &atom[paren_start + 1..paren_end];
    let parts: Vec<&str> = args.split(',').collect();
    if parts.len() != 2 {
        return None;
    }
    let x: usize = parts[0].trim().parse().ok()?;
    let y: usize = parts[1].trim().parse().ok()?;
    Some((name, x, y))
}

pub fn generate_levels(world: &mut World) {
    let seed_base = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u32)
        .unwrap_or(42);

    let mut floors = Vec::with_capacity(3);
    for i in 0..3 {
        let floor_num = (i + 1) as u32;
        let seed = seed_base.wrapping_add(i as u32 * 1000);
        match generate_floor(floor_num, seed) {
            Ok(layout) => {
                info!("Generated floor {} (seed={})", floor_num, seed);
                floors.push(layout);
            }
            Err(e) => {
                warn!("Level generation failed for floor {}: {}. Using fallback.", floor_num, e);
                let fb = fallback_floors();
                world.insert_resource(fb);
                return;
            }
        }
    }

    world.insert_resource(GeneratedFloors {
        floors: [
            floors.remove(0),
            floors.remove(0),
            floors.remove(0),
        ],
    });
}

pub fn fallback_floors() -> GeneratedFloors {
    GeneratedFloors {
        floors: [
            "\
############
#..........#
#.B..o..C..#
#..........#
#.o.##.T..~#
#...##...g.#
#..B...o.$.#
#.....##.B.#
#..T..##.p.#
#.g.......>#
#....@.....#
############"
                .to_string(),
            "\
############
#<.........#
#.....H....#
#..T...o.z.#
#...##.....#
#...##..X..#
#.o....m.C.#
#.....s....#
#.....B....#
#..f...$...#
#.........>#
############"
                .to_string(),
            "\
############
#<.........#
#..........#
#....o..M..#
#...##..g..#
#...##.!.C.#
#..B...T.~.#
#.....e....#
#..T..##...#
#.i...##.$.#
#........E.#
############"
                .to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashSet, VecDeque};

    fn validate_layout(layout: &str, floor_num: u32) {
        let lines: Vec<&str> = layout.lines().collect();
        assert_eq!(lines.len(), 12, "layout should have 12 rows");
        for line in &lines {
            assert_eq!(line.len(), 12, "each row should have 12 columns");
        }

        // Border walls
        for x in 0..12 {
            assert_eq!(
                lines[0].chars().nth(x).unwrap(),
                '#',
                "top border at x={} should be wall",
                x
            );
            assert_eq!(
                lines[11].chars().nth(x).unwrap(),
                '#',
                "bottom border at x={} should be wall",
                x
            );
        }
        for y in 0..12 {
            assert_eq!(
                lines[y].chars().nth(0).unwrap(),
                '#',
                "left border at y={} should be wall",
                y
            );
            assert_eq!(
                lines[y].chars().nth(11).unwrap(),
                '#',
                "right border at y={} should be wall",
                y
            );
        }

        // Count entities
        let mut player_count = 0;
        let mut goblin_count = 0;
        let mut torch_count = 0;
        let mut barrel_count = 0;
        let mut oil_count = 0;
        let mut stairs_down_count = 0;
        let mut stairs_up_count = 0;
        let mut exit_count = 0;

        for line in &lines {
            for ch in line.chars() {
                match ch {
                    '@' => player_count += 1,
                    'g' => goblin_count += 1,
                    'T' => torch_count += 1,
                    'B' => barrel_count += 1,
                    'o' => oil_count += 1,
                    '>' => stairs_down_count += 1,
                    '<' => stairs_up_count += 1,
                    'E' => exit_count += 1,
                    _ => {}
                }
            }
        }

        assert!(torch_count >= 1, "should have at least 1 torch, got {}", torch_count);
        assert!(barrel_count >= 1, "should have at least 1 barrel, got {}", barrel_count);
        assert!(oil_count >= 1, "should have at least 1 oil, got {}", oil_count);

        match floor_num {
            1 => {
                assert_eq!(player_count, 1, "floor 1 should have 1 player");
                assert_eq!(stairs_down_count, 1, "floor 1 should have 1 stairs down");
                assert_eq!(goblin_count, 2, "floor 1 should have 2 goblins");
            }
            2 => {
                assert_eq!(stairs_up_count, 1, "floor 2 should have 1 stairs up");
                assert_eq!(stairs_down_count, 1, "floor 2 should have 1 stairs down");
            }
            3 => {
                assert_eq!(stairs_up_count, 1, "floor 3 should have 1 stairs up");
                assert_eq!(exit_count, 1, "floor 3 should have 1 exit");
            }
            _ => panic!("unexpected floor_num"),
        }
    }

    fn check_reachability(layout: &str) {
        let lines: Vec<&str> = layout.lines().collect();
        let grid: Vec<Vec<char>> = lines.iter().map(|l| l.chars().collect()).collect();

        // Find start position
        let mut start = None;
        let mut entity_positions = HashSet::new();

        for (y, row) in grid.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                match ch {
                    '@' | '<' => {
                        if start.is_none() {
                            start = Some((x, y));
                        }
                        entity_positions.insert((x, y));
                    }
                    'g' | 'T' | 'B' | 'o' | '>' | 'E' | '~' | 'p' | 'f' | 'i' | 's' | 'e' | 'X' | 'M' | 'm' | 'z' | '!' | 'C' | '$' | 'H' => {
                        entity_positions.insert((x, y));
                    }
                    _ => {}
                }
            }
        }

        let start = start.expect("layout must have a start position (@ or <)");

        // BFS flood fill
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(start);
        queue.push_back(start);

        while let Some((x, y)) = queue.pop_front() {
            for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= 12 || ny >= 12 {
                    continue;
                }
                let (nx, ny) = (nx as usize, ny as usize);
                if visited.contains(&(nx, ny)) {
                    continue;
                }
                if grid[ny][nx] != '#' {
                    visited.insert((nx, ny));
                    queue.push_back((nx, ny));
                }
            }
        }

        for pos in &entity_positions {
            assert!(
                visited.contains(pos),
                "entity at ({}, {}) is not reachable from start {:?}",
                pos.0,
                pos.1,
                start
            );
        }
    }

    #[test]
    fn generate_floor_1_produces_valid_layout() {
        let layout = generate_floor(1, 42).expect("floor 1 generation should succeed");
        validate_layout(&layout, 1);
    }

    #[test]
    fn generate_floor_2_produces_valid_layout() {
        let layout = generate_floor(2, 42).expect("floor 2 generation should succeed");
        validate_layout(&layout, 2);
    }

    #[test]
    fn generate_floor_3_produces_valid_layout() {
        let layout = generate_floor(3, 42).expect("floor 3 generation should succeed");
        validate_layout(&layout, 3);
    }

    #[test]
    fn generated_floors_are_reachable() {
        for floor_num in 1..=3 {
            let layout =
                generate_floor(floor_num, 123).expect(&format!("floor {} should generate", floor_num));
            check_reachability(&layout);
        }
    }

    #[test]
    fn different_seeds_produce_different_layouts() {
        let layout_a = generate_floor(1, 1).expect("should generate");
        let layout_b = generate_floor(1, 9999).expect("should generate");
        assert_ne!(layout_a, layout_b, "different seeds should produce different layouts");
    }

    #[test]
    fn fallback_floors_are_valid() {
        let fb = fallback_floors();
        for (i, floor) in fb.floors.iter().enumerate() {
            let floor_num = (i + 1) as u32;
            validate_layout(floor, floor_num);
            check_reachability(floor);
        }
    }
}
