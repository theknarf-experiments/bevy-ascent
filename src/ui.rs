use bevy::prelude::*;
use bevy::ecs::spawn::Spawn;
use bevy::feathers::controls::{button, ButtonProps};
use bevy::picking::Pickable;
use bevy::ui_widgets::Activate;

use crate::components::*;
use crate::render::{glyph_for, name_for};

// ---- Hovered cell resource ----

#[derive(Resource, Default)]
pub struct HoveredCell(pub Option<IVec2>);

// ---- Main Menu ----

pub fn spawn_main_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            DespawnOnExit(GameState::MainMenu),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Datalog Roguelike"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
            ));

            // Subtitle
            parent.spawn((
                Text::new("A systemic dungeon with fire, ice & logic"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Start button
            parent
                .spawn(button(
                    ButtonProps::default(),
                    (),
                    (Spawn((
                        Text::new("Start Game"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                    )),),
                ))
                .observe(
                    |_trigger: On<Activate>,
                     mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::Playing);
                    },
                );
        });
}

// ---- Victory Banner (small corner banner, doesn't freeze gameplay) ----

pub fn show_victory_banner(
    mut commands: Commands,
    victory: Res<VictoryAchieved>,
    existing: Query<(), With<VictoryBanner>>,
) {
    if !victory.0 || !existing.is_empty() {
        return;
    }

    commands
        .spawn((
            VictoryBanner,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.3, 0.1, 0.9)),
            DespawnOnExit(GameState::Playing),
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Victory!"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 1.0, 0.3)),
                Pickable::IGNORE,
            ));

            parent
                .spawn(button(
                    ButtonProps::default(),
                    (),
                    (Spawn((
                        Text::new("Play Again"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                    )),),
                ))
                .observe(
                    |_trigger: On<Activate>,
                     mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::MainMenu);
                    },
                );
        });
}

// ---- Game Over Screen ----

pub fn spawn_game_over_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.3, 0.1, 0.1, 0.9)),
            DespawnOnExit(GameState::GameOver),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Game Over"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
            ));

            parent.spawn((
                Text::new("You perished in the dungeon."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            parent
                .spawn(button(
                    ButtonProps::default(),
                    (),
                    (Spawn((
                        Text::new("Try Again"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                    )),),
                ))
                .observe(
                    |_trigger: On<Activate>,
                     mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::Playing);
                    },
                );
        });
}

// ---- Hover observers (global) ----

pub fn on_hover_over(
    trigger: On<Pointer<Over>>,
    mut hovered: ResMut<HoveredCell>,
    grid_query: Query<&GridPos>,
) {
    if let Ok(grid_pos) = grid_query.get(trigger.entity) {
        hovered.0 = Some(grid_pos.0);
    }
}

pub fn on_hover_out(
    trigger: On<Pointer<Out>>,
    mut hovered: ResMut<HoveredCell>,
    grid_query: Query<&GridPos>,
) {
    if grid_query.get(trigger.entity).is_ok() {
        hovered.0 = None;
    }
}

// ---- Tooltip ----

pub fn spawn_tooltip(mut commands: Commands) {
    commands
        .spawn((
            TooltipRoot,
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            Visibility::Hidden,
            DespawnOnExit(GameState::Playing),
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Pickable::IGNORE,
            ));
        });
}

pub fn update_tooltip(
    hovered: Res<HoveredCell>,
    mut tooltip_query: Query<(&mut Node, &mut Visibility, &Children), With<TooltipRoot>>,
    mut text_query: Query<&mut Text>,
    entity_query: Query<(
        &GridPos,
        Option<&Player>,
        Option<&Enemy>,
        Option<&Exit>,
        Option<&Pushable>,
        Option<&Blocking>,
        Option<&Tags>,
        Option<&Health>,
        Option<&StairsDown>,
        Option<&StairsUp>,
    )>,
    window_query: Query<&Window>,
) {
    let Ok((mut node, mut visibility, children)) = tooltip_query.single_mut() else {
        return;
    };

    let Some(cell) = hovered.0 else {
        *visibility = Visibility::Hidden;
        return;
    };

    // Collect entities at this cell
    let mut lines: Vec<String> = Vec::new();
    for (grid_pos, player, enemy, exit, pushable, blocking, tags, health, stairs_down, stairs_up) in entity_query.iter() {
        if grid_pos.0 != cell {
            continue;
        }
        let name = name_for(player, enemy, exit, pushable, blocking, tags, stairs_down, stairs_up);
        let glyph = glyph_for(player, enemy, exit, pushable, blocking, tags, stairs_down, stairs_up);

        let mut line = format!("{} ({})", name, glyph);

        if let Some(h) = health {
            line.push_str(&format!("  HP: {}", h.0));
        }

        if let Some(tags) = tags {
            let tag_names: Vec<&str> = tags
                .0
                .iter()
                .filter_map(|t| match t {
                    Tag::Wood => Some("Wood"),
                    Tag::Oil => Some("Oil"),
                    Tag::Ice => Some("Ice"),
                    Tag::OnFire => Some("OnFire"),
                    Tag::Wet => Some("Wet"),
                    _ => None,
                })
                .collect();
            if !tag_names.is_empty() {
                line.push_str(&format!("  [{}]", tag_names.join(", ")));
            }
        }

        lines.push(line);
    }

    if lines.is_empty() {
        *visibility = Visibility::Hidden;
        return;
    }

    *visibility = Visibility::Inherited;

    // Update text
    if let Some(&text_entity) = children.first() {
        if let Ok(mut text) = text_query.get_mut(text_entity) {
            **text = lines.join("\n");
        }
    }

    // Position tooltip near cursor
    let Ok(window) = window_query.single() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        node.left = Val::Px(cursor_pos.x + 15.0);
        node.top = Val::Px(cursor_pos.y + 15.0);
    }
}

// ---- Floor Indicator ----

pub fn spawn_floor_indicator(mut commands: Commands, floor: Res<CurrentFloor>) {
    commands.spawn((
        FloorIndicator,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        DespawnOnExit(GameState::Playing),
        Pickable::IGNORE,
        Text::new(format!("Floor {}", floor.0)),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
    ));
}

pub fn update_floor_indicator(
    floor: Res<CurrentFloor>,
    mut query: Query<&mut Text, With<FloorIndicator>>,
) {
    if !floor.is_changed() {
        return;
    }
    for mut text in query.iter_mut() {
        **text = format!("Floor {}", floor.0);
    }
}
