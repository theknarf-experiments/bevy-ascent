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
                    ));
                }
                '@' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Flesh])),
                        DerivedTags::default(),
                        Player,
                        Health(3),
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
                    ));
                }
                'T' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Wood, Tag::OnFire])),
                        DerivedTags::default(),
                        Pushable,
                    ));
                }
                'B' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Wood])),
                        DerivedTags::default(),
                        Pushable,
                        Blocking,
                    ));
                }
                'o' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Oil])),
                        DerivedTags::default(),
                    ));
                }
                'I' => {
                    commands.spawn((
                        GridPos(pos),
                        Tags(BTreeSet::from([Tag::Ice])),
                        DerivedTags::default(),
                        Pushable,
                        Blocking,
                    ));
                }
                'E' => {
                    commands.spawn((
                        GridPos(pos),
                        Exit,
                    ));
                }
                _ => {} // '.' or anything else = empty floor
            }
        }
    }
}
