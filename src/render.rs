use bevy::prelude::*;

use crate::components::*;

const CELL_SIZE: f32 = 40.0;
const GRID_W: f32 = 12.0;
const GRID_H: f32 = 12.0;

pub fn setup_camera(mut commands: Commands) {
    let center_x = (GRID_W - 1.0) * CELL_SIZE / 2.0;
    let center_y = -(GRID_H - 1.0) * CELL_SIZE / 2.0;
    commands.spawn((
        Camera2d,
        Transform::from_xyz(center_x, center_y, 999.0),
    ));
}

fn glyph_for(
    player: Option<&Player>,
    enemy: Option<&Enemy>,
    exit: Option<&Exit>,
    pushable: Option<&Pushable>,
    blocking: Option<&Blocking>,
    tags: Option<&Tags>,
) -> &'static str {
    if player.is_some() {
        return "@";
    }
    if enemy.is_some() {
        return "g";
    }
    if exit.is_some() {
        return "E";
    }
    if let Some(tags) = tags {
        if tags.0.contains(&Tag::Stone) {
            return "#";
        }
        if tags.0.contains(&Tag::Ice) {
            return "I";
        }
        if tags.0.contains(&Tag::Oil) {
            return "o";
        }
        if tags.0.contains(&Tag::Wood) {
            if tags.0.contains(&Tag::OnFire) && pushable.is_some() && blocking.is_none() {
                return "T";
            }
            if blocking.is_some() {
                return "B";
            }
            return "T";
        }
        if tags.0.contains(&Tag::Wet) && !tags.0.contains(&Tag::Flesh) {
            return "~";
        }
    }
    "?"
}

fn color_for(
    tags: Option<&Tags>,
    derived: Option<&DerivedTags>,
    player: Option<&Player>,
    enemy: Option<&Enemy>,
    exit: Option<&Exit>,
) -> Color {
    // Check derived tags first for dynamic state
    if let Some(dt) = derived {
        if dt.0.contains(&Tag::TakingDamage) {
            return Color::srgb(1.0, 0.2, 0.2); // bright red
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
        if tags.0.contains(&Tag::OnFire) {
            return Color::srgb(1.0, 0.6, 0.0); // orange
        }
    }

    if player.is_some() {
        return Color::srgb(0.2, 1.0, 0.2); // bright green
    }
    if enemy.is_some() {
        return Color::srgb(1.0, 0.3, 0.3); // red
    }
    if exit.is_some() {
        return Color::srgb(1.0, 1.0, 0.0); // yellow
    }

    if let Some(tags) = tags {
        if tags.0.contains(&Tag::Stone) {
            return Color::srgb(0.5, 0.5, 0.5); // gray
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
pub struct Sprite;

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
        ),
        Without<Sprite>,
    >,
) {
    for (entity, grid_pos, player, enemy, exit, pushable, blocking, tags, derived) in
        query.iter()
    {
        let glyph = glyph_for(player, enemy, exit, pushable, blocking, tags);
        let color = color_for(tags, derived, player, enemy, exit);
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
            Sprite,
        ));
    }
}

pub fn sync_transforms(mut query: Query<(&GridPos, &mut Transform), Changed<GridPos>>) {
    for (grid_pos, mut transform) in query.iter_mut() {
        transform.translation.x = grid_pos.0.x as f32 * CELL_SIZE;
        transform.translation.y = -(grid_pos.0.y as f32) * CELL_SIZE;
    }
}

pub fn sync_colors(
    mut query: Query<(
        &mut TextColor,
        &mut Text2d,
        Option<&Tags>,
        Option<&DerivedTags>,
        Option<&Player>,
        Option<&Enemy>,
        Option<&Exit>,
        Option<&Pushable>,
        Option<&Blocking>,
    )>,
) {
    for (mut text_color, mut text, tags, derived, player, enemy, exit, pushable, blocking) in
        query.iter_mut()
    {
        text_color.0 = color_for(tags, derived, player, enemy, exit);
        let glyph = glyph_for(player, enemy, exit, pushable, blocking, tags);
        **text = glyph.to_string();
    }
}
