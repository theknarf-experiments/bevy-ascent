use bevy::picking::Pickable;
use bevy::prelude::*;

use crate::components::*;
use crate::items::{item_color, item_glyph, item_name};

pub const CELL_SIZE: f32 = 40.0;
const GRID_W: f32 = 12.0;
const GRID_H: f32 = 12.0;

pub fn setup_camera(mut commands: Commands) {
    let center_x = (GRID_W - 1.0) * CELL_SIZE / 2.0;
    let center_y = -(GRID_H - 1.0) * CELL_SIZE / 2.0;
    commands.spawn((Camera2d, Transform::from_xyz(center_x, center_y, 999.0)));
}

pub fn glyph_for(
    player: Option<&Player>,
    enemy: Option<&Enemy>,
    exit: Option<&Exit>,
    pushable: Option<&Pushable>,
    blocking: Option<&Blocking>,
    tags: Option<&Tags>,
    stairs_down: Option<&StairsDown>,
    stairs_up: Option<&StairsUp>,
    item_kind: Option<&ItemKind>,
    chest: Option<&Chest>,
    boss: Option<&Boss>,
) -> &'static str {
    if player.is_some() {
        return "@";
    }
    if boss.is_some() {
        return "D";
    }
    // Enemy subtypes based on tags
    if enemy.is_some() {
        if let Some(tags) = tags {
            if tags.0.contains(&Tag::OnFire) {
                return "f"; // Fire Imp
            }
            if tags.0.contains(&Tag::Ice) {
                return "i"; // Ice Golem
            }
            if tags.0.contains(&Tag::Poisoned) {
                return "s"; // Poison Spider
            }
            if tags.0.contains(&Tag::Electrified) {
                return "e"; // Shock Eel
            }
        }
        return "g"; // Goblin (default)
    }
    if let Some(kind) = item_kind {
        return item_glyph(kind);
    }
    if chest.is_some() {
        return "C";
    }
    if exit.is_some() {
        return "E";
    }
    if stairs_down.is_some() {
        return ">";
    }
    if stairs_up.is_some() {
        return "<";
    }
    if let Some(tags) = tags {
        if tags.0.contains(&Tag::Stone) {
            return "#";
        }
        if tags.0.contains(&Tag::Explosive) {
            return "X"; // Explosive Barrel
        }
        if tags.0.contains(&Tag::Metal) && tags.0.contains(&Tag::Electrified) && blocking.is_some()
        {
            return "!"; // Lightning Rod
        }
        if tags.0.contains(&Tag::Metal) && tags.0.contains(&Tag::Electrified) {
            return "z"; // Spark
        }
        if tags.0.contains(&Tag::Metal) && pushable.is_some() {
            return "M"; // Metal Crate
        }
        if tags.0.contains(&Tag::Ice) {
            return "I";
        }
        if tags.0.contains(&Tag::Oil) {
            return "o";
        }
        if tags.0.contains(&Tag::Wood) && tags.0.contains(&Tag::Poisoned) {
            return "m"; // Poison Mushroom
        }
        if tags.0.contains(&Tag::Wood) {
            if (tags.0.contains(&Tag::OnFire) || tags.0.contains(&Tag::FireSource))
                && pushable.is_some()
                && blocking.is_none()
            {
                return "T";
            }
            if blocking.is_some() {
                return "B";
            }
            return "T";
        }
        if tags.0.contains(&Tag::Poisoned) && !tags.0.contains(&Tag::Flesh) {
            return "p"; // Poison Gas
        }
        if tags.0.contains(&Tag::Wet)
            && !tags.0.contains(&Tag::Flesh)
            && !tags.0.contains(&Tag::Ice)
        {
            return "~";
        }
    }
    "?"
}

pub fn name_for(
    player: Option<&Player>,
    enemy: Option<&Enemy>,
    exit: Option<&Exit>,
    pushable: Option<&Pushable>,
    blocking: Option<&Blocking>,
    tags: Option<&Tags>,
    stairs_down: Option<&StairsDown>,
    stairs_up: Option<&StairsUp>,
    item_kind: Option<&ItemKind>,
    chest: Option<&Chest>,
    boss: Option<&Boss>,
) -> &'static str {
    if player.is_some() {
        return "Player";
    }
    if boss.is_some() {
        return "Dragon";
    }
    if enemy.is_some() {
        if let Some(tags) = tags {
            if tags.0.contains(&Tag::OnFire) {
                return "Fire Imp";
            }
            if tags.0.contains(&Tag::Ice) {
                return "Ice Golem";
            }
            if tags.0.contains(&Tag::Poisoned) {
                return "Poison Spider";
            }
            if tags.0.contains(&Tag::Electrified) {
                return "Shock Eel";
            }
        }
        return "Goblin";
    }
    if let Some(kind) = item_kind {
        return item_name(kind);
    }
    if chest.is_some() {
        return "Chest";
    }
    if exit.is_some() {
        return "Exit";
    }
    if stairs_down.is_some() {
        return "Stairs Down";
    }
    if stairs_up.is_some() {
        return "Stairs Up";
    }
    if let Some(tags) = tags {
        if tags.0.contains(&Tag::Stone) {
            return "Wall";
        }
        if tags.0.contains(&Tag::Explosive) {
            return "Explosive Barrel";
        }
        if tags.0.contains(&Tag::Metal) && tags.0.contains(&Tag::Electrified) && blocking.is_some()
        {
            return "Lightning Rod";
        }
        if tags.0.contains(&Tag::Metal) && tags.0.contains(&Tag::Electrified) {
            return "Spark";
        }
        if tags.0.contains(&Tag::Metal) && pushable.is_some() {
            return "Metal Crate";
        }
        if tags.0.contains(&Tag::Ice) {
            return "Ice";
        }
        if tags.0.contains(&Tag::Oil) {
            return "Oil";
        }
        if tags.0.contains(&Tag::Wood) && tags.0.contains(&Tag::Poisoned) {
            return "Poison Mushroom";
        }
        if tags.0.contains(&Tag::Wood) {
            if (tags.0.contains(&Tag::OnFire) || tags.0.contains(&Tag::FireSource))
                && pushable.is_some()
                && blocking.is_none()
            {
                return "Torch";
            }
            if blocking.is_some() {
                return "Barrel";
            }
            return "Torch";
        }
        if tags.0.contains(&Tag::Poisoned) && !tags.0.contains(&Tag::Flesh) {
            return "Poison Gas";
        }
        if tags.0.contains(&Tag::Wet)
            && !tags.0.contains(&Tag::Flesh)
            && !tags.0.contains(&Tag::Ice)
        {
            return "Water";
        }
    }
    "Unknown"
}

fn color_for(
    tags: Option<&Tags>,
    derived: Option<&DerivedTags>,
    player: Option<&Player>,
    enemy: Option<&Enemy>,
    exit: Option<&Exit>,
    stairs_down: Option<&StairsDown>,
    stairs_up: Option<&StairsUp>,
    item_kind: Option<&ItemKind>,
    chest: Option<&Chest>,
    boss: Option<&Boss>,
) -> Color {
    // Check derived tags first for dynamic state
    if let Some(dt) = derived {
        if dt.0.contains(&Tag::FireDamage)
            || dt.0.contains(&Tag::PoisonDamage)
            || dt.0.contains(&Tag::ElectricDamage)
        {
            return Color::srgb(1.0, 0.2, 0.2); // bright red (any damage)
        }
        if dt.0.contains(&Tag::Exploding) {
            return Color::srgb(1.0, 0.4, 0.1); // orange-red
        }
        if dt.0.contains(&Tag::Electrified) {
            return Color::srgb(0.3, 0.8, 1.0); // electric blue
        }
        if dt.0.contains(&Tag::Poisoned) {
            return Color::srgb(0.2, 0.7, 0.1); // green
        }
        if dt.0.contains(&Tag::OnFire) {
            return Color::srgb(1.0, 0.6, 0.0); // orange
        }
        if dt.0.contains(&Tag::Extinguished) {
            return Color::srgb(0.5, 0.5, 0.7); // blue-gray
        }
        if dt.0.contains(&Tag::Melted) {
            return Color::srgb(0.3, 0.5, 1.0); // blue
        }
    }

    if let Some(tags) = tags {
        if tags.0.contains(&Tag::OnFire) || tags.0.contains(&Tag::FireSource) {
            return Color::srgb(1.0, 0.6, 0.0); // orange
        }
        if tags.0.contains(&Tag::Electrified) {
            return Color::srgb(0.3, 0.8, 1.0); // electric blue
        }
        if tags.0.contains(&Tag::Poisoned) && !tags.0.contains(&Tag::Flesh) {
            return Color::srgb(0.2, 0.7, 0.1); // green
        }
        if tags.0.contains(&Tag::Explosive) {
            return Color::srgb(1.0, 0.4, 0.1); // orange-red
        }
    }

    // Item-specific colors
    if let Some(kind) = item_kind {
        return item_color(kind);
    }

    if chest.is_some() {
        return Color::srgb(0.6, 0.4, 0.2); // brown
    }

    if player.is_some() {
        return Color::srgb(0.2, 1.0, 0.2); // bright green
    }
    if boss.is_some() {
        return Color::srgb(0.9, 0.2, 0.0); // deep red-orange
    }
    if enemy.is_some() {
        if let Some(tags) = tags {
            if tags.0.contains(&Tag::Poisoned) {
                return Color::srgb(0.5, 1.0, 0.2); // bright green
            }
        }
        return Color::srgb(1.0, 0.3, 0.3); // red
    }
    if exit.is_some() {
        return Color::srgb(1.0, 1.0, 0.0); // yellow
    }
    if stairs_down.is_some() || stairs_up.is_some() {
        return Color::srgb(0.3, 0.9, 0.9); // cyan
    }

    if let Some(tags) = tags {
        if tags.0.contains(&Tag::Stone) {
            return Color::srgb(0.5, 0.5, 0.5); // gray
        }
        if tags.0.contains(&Tag::Metal) {
            return Color::srgb(0.7, 0.7, 0.8); // steel gray
        }
        if tags.0.contains(&Tag::Ice) {
            return Color::srgb(0.6, 0.8, 1.0); // light blue
        }
        if tags.0.contains(&Tag::Oil) {
            return Color::srgb(0.4, 0.2, 0.5); // purple
        }
        if tags.0.contains(&Tag::Wood) {
            return Color::srgb(0.6, 0.4, 0.2); // brown
        }
        if tags.0.contains(&Tag::Wet) {
            return Color::srgb(0.3, 0.5, 1.0); // blue
        }
    }

    Color::WHITE
}

/// Marker for entities that already have a Text2d sprite
#[derive(Component)]
pub struct HasSprite;

fn apply_fog(color: Color, visibility: TileVisibility, is_enemy: bool) -> Color {
    match visibility {
        TileVisibility::Visible => color,
        TileVisibility::Explored => {
            if is_enemy {
                Color::srgba(0.0, 0.0, 0.0, 0.0)
            } else {
                let Srgba {
                    red,
                    green,
                    blue,
                    alpha,
                } = color.to_srgba();
                Color::srgba(red * 0.5, green * 0.5, blue * 0.5, alpha * 0.3)
            }
        }
        TileVisibility::Unexplored => Color::srgba(0.0, 0.0, 0.0, 0.0),
    }
}

pub fn spawn_sprites(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &GridPos,
            Option<&Player>,
            Option<&Enemy>,
            Option<&Exit>,
            Option<&Pushable>,
            Option<&Blocking>,
            Option<&Tags>,
            Option<&DerivedTags>,
            Option<&StairsDown>,
            Option<&StairsUp>,
            Option<&ItemKind>,
            Option<&Chest>,
            Option<&Boss>,
        ),
        (Without<HasSprite>, Without<TileBackground>),
    >,
    fog_map: Res<FogMap>,
) {
    for (
        entity,
        grid_pos,
        player,
        enemy,
        exit,
        pushable,
        blocking,
        tags,
        derived,
        stairs_down,
        stairs_up,
        item_kind,
        chest,
        boss,
    ) in query.iter()
    {
        let glyph = glyph_for(
            player,
            enemy,
            exit,
            pushable,
            blocking,
            tags,
            stairs_down,
            stairs_up,
            item_kind,
            chest,
            boss,
        );
        let mut color = color_for(
            tags,
            derived,
            player,
            enemy,
            exit,
            stairs_down,
            stairs_up,
            item_kind,
            chest,
            boss,
        );

        // Apply fog (player is always visible)
        if player.is_none() {
            let vis = fog_map.get(grid_pos.0.x, grid_pos.0.y);
            color = apply_fog(color, vis, enemy.is_some() || boss.is_some());
        }

        let x = grid_pos.0.x as f32 * CELL_SIZE;
        let y = -(grid_pos.0.y as f32) * CELL_SIZE;

        commands.entity(entity).insert((
            Text2d::new(glyph),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(color),
            Transform::from_xyz(x, y, 0.0),
            // Transparent sprite for picking hit-test
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.01),
                custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                ..default()
            },
            Pickable::default(),
            HasSprite,
        ));
    }
}

/// Reactively hide sprites that have no GridPos (e.g. items in inventory).
pub fn sync_visibility(
    mut with_pos: Query<&mut Visibility, (With<HasSprite>, With<GridPos>)>,
    mut without_pos: Query<&mut Visibility, (With<HasSprite>, Without<GridPos>)>,
) {
    for mut vis in without_pos.iter_mut() {
        if *vis != Visibility::Hidden {
            *vis = Visibility::Hidden;
        }
    }
    for mut vis in with_pos.iter_mut() {
        if *vis == Visibility::Hidden {
            *vis = Visibility::Inherited;
        }
    }
}

pub fn sync_transforms(mut query: Query<(&GridPos, &mut Transform), Changed<GridPos>>) {
    for (grid_pos, mut transform) in query.iter_mut() {
        transform.translation.x = grid_pos.0.x as f32 * CELL_SIZE;
        transform.translation.y = -(grid_pos.0.y as f32) * CELL_SIZE;
    }
}

pub fn sync_colors(
    mut query: Query<
        (
            &mut TextColor,
            &mut Text2d,
            &mut Sprite,
            Option<&Tags>,
            Option<&DerivedTags>,
            Option<&Player>,
            Option<&Enemy>,
            Option<&Exit>,
            Option<&Pushable>,
            Option<&Blocking>,
            Option<&FlashTimer>,
            (
                Option<&StairsDown>,
                Option<&StairsUp>,
                Option<&ItemKind>,
                Option<&Chest>,
                Option<&Boss>,
                Option<&GridPos>,
            ),
        ),
        Without<TileBackground>,
    >,
    fog_map: Res<FogMap>,
) {
    for (
        mut text_color,
        mut text,
        mut sprite,
        tags,
        derived,
        player,
        enemy,
        exit,
        pushable,
        blocking,
        flash,
        (stairs_down, stairs_up, item_kind, chest, boss, grid_pos),
    ) in query.iter_mut()
    {
        if flash.is_some() {
            text_color.0 = Color::WHITE;
        } else {
            let mut color = color_for(
                tags,
                derived,
                player,
                enemy,
                exit,
                stairs_down,
                stairs_up,
                item_kind,
                chest,
                boss,
            );

            // Apply fog (player is always visible)
            if player.is_none() {
                if let Some(gp) = grid_pos {
                    let vis = fog_map.get(gp.0.x, gp.0.y);
                    color = apply_fog(color, vis, enemy.is_some() || boss.is_some());
                }
            }

            text_color.0 = color;
        }
        let glyph = glyph_for(
            player,
            enemy,
            exit,
            pushable,
            blocking,
            tags,
            stairs_down,
            stairs_up,
            item_kind,
            chest,
            boss,
        );
        **text = glyph.to_string();

        // Set sprite background color based on environmental tags on this entity
        let bg = tile_bg_color(tags, derived);
        if player.is_none() {
            if let Some(gp) = grid_pos {
                let vis = fog_map.get(gp.0.x, gp.0.y);
                sprite.color = match vis {
                    TileVisibility::Visible => bg,
                    TileVisibility::Explored => {
                        let Srgba { red, green, blue, alpha } = bg.to_srgba();
                        Color::srgba(red * 0.5, green * 0.5, blue * 0.5, alpha * 0.3)
                    }
                    TileVisibility::Unexplored => Color::srgba(0.0, 0.0, 0.0, 0.0),
                };
            }
        } else {
            sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.01);
        }
    }
}

/// Determine the background tint for a tile based on environmental tags.
/// Returns transparent if no environmental effect is present.
fn tile_bg_color(tags: Option<&Tags>, derived: Option<&DerivedTags>) -> Color {
    let has = |tag: Tag| -> bool {
        tags.is_some_and(|t| t.0.contains(&tag))
            || derived.is_some_and(|d| d.0.contains(&tag))
    };
    // Priority: poison > fire > electric > wet
    if has(Tag::Poisoned) {
        Color::srgba(0.0, 0.6, 0.0, 0.55)
    } else if has(Tag::OnFire) || has(Tag::FireSource) {
        Color::srgba(0.7, 0.3, 0.0, 0.55)
    } else if has(Tag::Electrified) {
        Color::srgba(0.2, 0.5, 1.0, 0.5)
    } else if has(Tag::Wet) {
        Color::srgba(0.0, 0.3, 0.8, 0.45)
    } else {
        Color::srgba(0.0, 0.0, 0.0, 0.01)
    }
}

pub fn spawn_tile_backgrounds(mut commands: Commands) {
    for y in 0..12 {
        for x in 0..12 {
            let px = x as f32 * CELL_SIZE;
            let py = -(y as f32) * CELL_SIZE;
            commands.spawn((
                GridPos(IVec2::new(x, y)),
                TileBackground,
                DespawnOnExit(GameState::Playing),
                Text2d::new(" "),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                Sprite {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                    ..default()
                },
                Transform::from_xyz(px, py, -1.0),
                HasSprite,
            ));
        }
    }
}

pub fn sync_tile_backgrounds(
    entity_query: Query<
        (&GridPos, Option<&Tags>, Option<&DerivedTags>),
        Without<TileBackground>,
    >,
    mut bg_query: Query<(&GridPos, &mut Sprite), With<TileBackground>>,
    fog_map: Res<FogMap>,
) {
    use std::collections::HashMap;

    let mut tile_colors: HashMap<IVec2, (u8, Color)> = HashMap::new();

    let env_tags = [
        Tag::Poisoned,
        Tag::OnFire,
        Tag::FireSource,
        Tag::Electrified,
        Tag::Wet,
    ];

    for (gp, tags, derived) in entity_query.iter() {
        let pos = gp.0;
        for &tag in &env_tags {
            let present = tags.is_some_and(|t| t.0.contains(&tag))
                || derived.is_some_and(|d| d.0.contains(&tag));
            if !present {
                continue;
            }
            if let Some((priority, color)) = match tag {
                Tag::Poisoned => Some((0u8, Color::srgba(0.0, 0.6, 0.0, 0.55))),
                Tag::OnFire | Tag::FireSource => Some((1, Color::srgba(0.7, 0.3, 0.0, 0.55))),
                Tag::Electrified => Some((2, Color::srgba(0.2, 0.5, 1.0, 0.5))),
                Tag::Wet => Some((3, Color::srgba(0.0, 0.3, 0.8, 0.45))),
                _ => None,
            } {
                tile_colors
                    .entry(pos)
                    .and_modify(|(ep, ec)| {
                        if priority < *ep {
                            *ep = priority;
                            *ec = color;
                        }
                    })
                    .or_insert((priority, color));
            }
        }
    }

    for (gp, mut sprite) in bg_query.iter_mut() {
        let base_color = tile_colors
            .get(&gp.0)
            .map(|&(_, c)| c)
            .unwrap_or(Color::srgba(0.0, 0.0, 0.0, 0.0));

        let vis = fog_map.get(gp.0.x, gp.0.y);
        sprite.color = match vis {
            TileVisibility::Visible => base_color,
            TileVisibility::Explored => {
                let Srgba { red, green, blue, alpha } = base_color.to_srgba();
                Color::srgba(red * 0.5, green * 0.5, blue * 0.5, alpha * 0.3)
            }
            TileVisibility::Unexplored => Color::srgba(0.0, 0.0, 0.0, 0.0),
        };
    }
}

