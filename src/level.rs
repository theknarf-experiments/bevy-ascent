use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;

const FLOOR_1: &str = "\
############
#..........#
#.B..o.....#
#..........#
#.o.##.T...#
#...##...g.#
#..B...o...#
#.....##.B.#
#..T..##...#
#.g.......>#
#....@.....#
############";

const FLOOR_2: &str = "\
############
#<.........#
#..........#
#..T...o...#
#...##.....#
#...##..B..#
#.o........#
#..........#
#.....B....#
#..........#
#.........>#
############";

const FLOOR_3: &str = "\
############
#<.........#
#..........#
#....o.....#
#...##..g..#
#...##.....#
#..B...T...#
#..........#
#..T..##...#
#.g...##...#
#........E.#
############";

fn floor_layout(floor: u32) -> &'static str {
    match floor {
        1 => FLOOR_1,
        2 => FLOOR_2,
        3 => FLOOR_3,
        _ => panic!("Invalid floor number: {}", floor),
    }
}

pub struct FloorSpawnResult {
    pub player_spawn: Option<IVec2>,
    pub stairs_up_pos: Option<IVec2>,
    pub stairs_down_pos: Option<IVec2>,
}

pub fn spawn_floor(commands: &mut Commands, floor: u32) -> FloorSpawnResult {
    let layout = floor_layout(floor);
    let mut result = FloorSpawnResult {
        player_spawn: None,
        stairs_up_pos: None,
        stairs_down_pos: None,
    };

    for (row, line) in layout.lines().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let pos = IVec2::new(col as i32, row as i32);
            match ch {
                '#' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Stone])),
                        Blocking,
                        FloorEntity,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                '@' => {
                    result.player_spawn = Some(pos);
                }
                'g' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Flesh])),
                        DerivedTags::default(),
                        Enemy,
                        Health(2),
                        Blocking,
                        FloorEntity,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                'T' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Wood, Tag::OnFire])),
                        DerivedTags::default(),
                        Pushable,
                        FloorEntity,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                'B' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Wood])),
                        DerivedTags::default(),
                        Pushable,
                        Blocking,
                        FloorEntity,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                'o' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Oil])),
                        DerivedTags::default(),
                        FloorEntity,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                'I' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Ice])),
                        DerivedTags::default(),
                        Pushable,
                        Blocking,
                        FloorEntity,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                'E' => {
                    commands.spawn((
                        GridPos(pos),
                        Exit,
                        FloorEntity,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                '>' => {
                    result.stairs_down_pos = Some(pos);
                    commands.spawn((
                        GridPos(pos),
                        StairsDown,
                        FloorEntity,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                '<' => {
                    result.stairs_up_pos = Some(pos);
                    commands.spawn((
                        GridPos(pos),
                        StairsUp,
                        FloorEntity,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                _ => {} // '.' or anything else = empty floor
            }
        }
    }

    result
}

pub fn spawn_initial_floor(mut commands: Commands, mut floor: ResMut<CurrentFloor>) {
    floor.0 = 1;
    let result = spawn_floor(&mut commands, 1);
    let player_pos = result.player_spawn.expect("Floor 1 must have a player spawn (@)");
    commands.spawn((
        GridPos(player_pos),
        Tags(BTreeSet::from([Tag::Flesh])),
        DerivedTags::default(),
        Player,
        Health(5),
        DespawnOnExit(GameState::Playing),
    ));
}
