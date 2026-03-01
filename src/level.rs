use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;

const LEVEL: &str = "\
############
#..........#
#.B..o..I..#
#..........#
#.o.##.T...#
#...##...g.#
#..B...o...#
#.....##.B.#
#..T..##...#
#.g......o.#
#....@...E.#
############";

pub fn spawn_level(mut commands: Commands) {
    for (row, line) in LEVEL.lines().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let pos = IVec2::new(col as i32, row as i32);
            match ch {
                '#' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Stone])),
                        Blocking,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                '@' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Flesh])),
                        DerivedTags::default(),
                        Player,
                        Health(3),
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                'g' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Flesh])),
                        DerivedTags::default(),
                        Enemy,
                        Health(2),
                        Blocking,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                'T' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Wood, Tag::OnFire])),
                        DerivedTags::default(),
                        Pushable,
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
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                'o' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Oil])),
                        DerivedTags::default(),
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
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                'E' => {
                    commands.spawn((
                        GridPos(pos),
                        Exit,
                        DespawnOnExit(GameState::Playing),
                    ));
                }
                _ => {} // '.' or anything else = empty floor
            }
        }
    }
}
