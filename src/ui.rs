use bevy::prelude::*;
use bevy::ecs::spawn::Spawn;
use bevy::feathers::controls::{button, ButtonProps};
use bevy::picking::Pickable;
use bevy::ui_widgets::Activate;

use crate::components::*;
use crate::items::item_name;
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

            // Button container (fixed width so all buttons match)
            parent.spawn(Node {
                width: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            }).with_children(|buttons| {
                buttons
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

                buttons
                    .spawn(button(
                        ButtonProps::default(),
                        (),
                        (Spawn((
                            Text::new("Settings"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                        )),),
                    ))
                    .observe(
                        |_trigger: On<Activate>,
                         mut next_overlay: ResMut<NextState<MenuOverlay>>,
                         mut origin: ResMut<SettingsOrigin>| {
                            origin.0 = SettingsFrom::MainMenu;
                            next_overlay.set(MenuOverlay::Settings);
                        },
                    );

                buttons
                    .spawn(button(
                        ButtonProps::default(),
                        (),
                        (Spawn((
                            Text::new("Quit"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                        )),),
                    ))
                    .observe(
                        |_trigger: On<Activate>,
                         mut exit: MessageWriter<AppExit>| {
                            exit.write(AppExit::Success);
                        },
                    );
            });
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

pub fn spawn_game_over_screen(mut commands: Commands, death_cause: Res<DeathCause>) {
    let cause_text = death_cause
        .0
        .as_deref()
        .unwrap_or("You perished in the dungeon.");
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
                Text::new(cause_text.to_string()),
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
        Option<&ItemKind>,
        Option<&Chest>,
        Option<&Boss>,
    )>,
    window_query: Query<&Window>,
    fog_map: Res<FogMap>,
) {
    let Ok((mut node, mut visibility, children)) = tooltip_query.single_mut() else {
        return;
    };

    let Some(cell) = hovered.0 else {
        *visibility = Visibility::Hidden;
        return;
    };

    // Check fog visibility
    let tile_vis = fog_map.get(cell.x, cell.y);
    if tile_vis == TileVisibility::Unexplored {
        *visibility = Visibility::Hidden;
        return;
    }

    // Collect entities at this cell
    let mut lines: Vec<String> = Vec::new();
    for (grid_pos, player, enemy, exit, pushable, blocking, tags, health, stairs_down, stairs_up, item_kind, chest, boss) in entity_query.iter() {
        if grid_pos.0 != cell {
            continue;
        }

        // In explored (not visible) tiles, hide enemies
        if tile_vis == TileVisibility::Explored && (enemy.is_some() || boss.is_some()) {
            continue;
        }

        let name = name_for(player, enemy, exit, pushable, blocking, tags, stairs_down, stairs_up, item_kind, chest, boss);
        let glyph = glyph_for(player, enemy, exit, pushable, blocking, tags, stairs_down, stairs_up, item_kind, chest, boss);

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
                    Tag::Metal => Some("Metal"),
                    Tag::Poisoned => Some("Poisoned"),
                    Tag::Electrified => Some("Electrified"),
                    Tag::Explosive => Some("Explosive"),
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

// ---- Stats Panel (right side) ----

pub fn spawn_stats_panel(mut commands: Commands, floor: Res<CurrentFloor>) {
    commands
        .spawn((
            StatsPanel,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(170.0),
                padding: UiRect::all(Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            DespawnOnExit(GameState::Playing),
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Player"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
                Pickable::IGNORE,
            ));

            // HP
            parent.spawn((
                StatsHpText,
                Text::new("HP: 5"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 1.0, 0.2)),
                Pickable::IGNORE,
            ));

            // Floor
            parent.spawn((
                StatsFloorText,
                Text::new(format!("Floor: {}", floor.0)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Pickable::IGNORE,
            ));

            // Gold
            parent.spawn((
                StatsGoldText,
                Text::new("Gold: 0"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.0)),
                Pickable::IGNORE,
            ));

            // Equipment header
            parent.spawn((
                Text::new("--- Equipment ---"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Pickable::IGNORE,
            ));

            // Weapon
            parent.spawn((
                StatsWeaponText,
                Text::new("Wpn: (none)"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.9)),
                Pickable::IGNORE,
            ));

            // Armor
            parent.spawn((
                StatsArmorText,
                Text::new("Arm: (none)"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.9)),
                Pickable::IGNORE,
            ));

            // Consumables header
            parent.spawn((
                Text::new("--- Items [1-4] ---"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Pickable::IGNORE,
            ));

            // Consumables
            parent.spawn((
                StatsConsumablesText,
                Text::new("1: (empty)\n2: (empty)\n3: (empty)\n4: (empty)"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Pickable::IGNORE,
            ));

            // Log header
            parent.spawn((
                Text::new("--- Log ---"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Pickable::IGNORE,
            ));

            // Log entries
            parent.spawn((
                StatsLogText,
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Pickable::IGNORE,
            ));
        });
}

pub fn update_stats_panel(
    player_query: Query<(&Health, &Inventory), With<Player>>,
    floor: Res<CurrentFloor>,
    gold: Res<GoldCount>,
    game_log: Res<GameLog>,
    mut hp_text: Query<
        &mut Text,
        (With<StatsHpText>, Without<StatsFloorText>, Without<StatsGoldText>, Without<StatsWeaponText>, Without<StatsArmorText>, Without<StatsConsumablesText>, Without<StatsLogText>),
    >,
    mut floor_text: Query<
        &mut Text,
        (With<StatsFloorText>, Without<StatsHpText>, Without<StatsGoldText>, Without<StatsWeaponText>, Without<StatsArmorText>, Without<StatsConsumablesText>, Without<StatsLogText>),
    >,
    mut gold_text: Query<
        &mut Text,
        (With<StatsGoldText>, Without<StatsHpText>, Without<StatsFloorText>, Without<StatsWeaponText>, Without<StatsArmorText>, Without<StatsConsumablesText>, Without<StatsLogText>),
    >,
    mut weapon_text: Query<
        &mut Text,
        (With<StatsWeaponText>, Without<StatsHpText>, Without<StatsFloorText>, Without<StatsGoldText>, Without<StatsArmorText>, Without<StatsConsumablesText>, Without<StatsLogText>),
    >,
    mut armor_text: Query<
        &mut Text,
        (With<StatsArmorText>, Without<StatsHpText>, Without<StatsFloorText>, Without<StatsGoldText>, Without<StatsWeaponText>, Without<StatsConsumablesText>, Without<StatsLogText>),
    >,
    mut consumables_text: Query<
        &mut Text,
        (With<StatsConsumablesText>, Without<StatsHpText>, Without<StatsFloorText>, Without<StatsGoldText>, Without<StatsWeaponText>, Without<StatsArmorText>, Without<StatsLogText>),
    >,
    mut log_text: Query<
        &mut Text,
        (With<StatsLogText>, Without<StatsHpText>, Without<StatsFloorText>, Without<StatsGoldText>, Without<StatsWeaponText>, Without<StatsArmorText>, Without<StatsConsumablesText>),
    >,
    item_query: Query<(&ItemKind, Option<&WeaponDamage>, Option<&ArmorDefense>), With<Item>>,
) {
    if let Ok((health, inventory)) = player_query.single() {
        for mut text in hp_text.iter_mut() {
            **text = format!("HP: {}", health.0);
        }

        // Weapon
        for mut text in weapon_text.iter_mut() {
            if let Some(weapon_entity) = inventory.weapon {
                if let Ok((kind, wpn_dmg, _)) = item_query.get(weapon_entity) {
                    let dmg_str = wpn_dmg.map(|d| format!(" +{}", d.0)).unwrap_or_default();
                    **text = format!("Wpn: {}{}", item_name(kind), dmg_str);
                } else {
                    **text = "Wpn: (none)".to_string();
                }
            } else {
                **text = "Wpn: (none)".to_string();
            }
        }

        // Armor
        for mut text in armor_text.iter_mut() {
            if let Some(armor_entity) = inventory.armor {
                if let Ok((kind, _, arm_def)) = item_query.get(armor_entity) {
                    let def_str = arm_def.map(|d| format!(" +{}", d.0)).unwrap_or_default();
                    **text = format!("Arm: {}{}", item_name(kind), def_str);
                } else {
                    **text = "Arm: (none)".to_string();
                }
            } else {
                **text = "Arm: (none)".to_string();
            }
        }

        // Consumables
        for mut text in consumables_text.iter_mut() {
            let mut lines = Vec::new();
            for i in 0..4 {
                if i < inventory.consumables.len() {
                    let entity = inventory.consumables[i];
                    if let Ok((kind, _, _)) = item_query.get(entity) {
                        lines.push(format!("{}: {}", i + 1, item_name(kind)));
                    } else {
                        lines.push(format!("{}: (empty)", i + 1));
                    }
                } else {
                    lines.push(format!("{}: (empty)", i + 1));
                }
            }
            **text = lines.join("\n");
        }
    }

    for mut text in floor_text.iter_mut() {
        **text = format!("Floor: {}", floor.0);
    }

    for mut text in gold_text.iter_mut() {
        **text = format!("Gold: {}", gold.0);
    }

    // Update log
    for mut text in log_text.iter_mut() {
        let recent: Vec<&str> = game_log.recent(8).iter().rev().map(|s| s.as_str()).collect();
        **text = recent.join("\n");
    }
}

// ---- Pause Menu ----

pub fn spawn_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            DespawnOnExit(MenuOverlay::Paused),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Paused"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Button container (fixed width so all buttons match)
            parent.spawn(Node {
                width: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            }).with_children(|buttons| {
                buttons
                    .spawn(button(
                        ButtonProps::default(),
                        (),
                        (Spawn((
                            Text::new("Continue"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                        )),),
                    ))
                    .observe(
                        |_trigger: On<Activate>,
                         mut next_overlay: ResMut<NextState<MenuOverlay>>| {
                            next_overlay.set(MenuOverlay::None);
                        },
                    );

                buttons
                    .spawn(button(
                        ButtonProps::default(),
                        (),
                        (Spawn((
                            Text::new("Settings"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                        )),),
                    ))
                    .observe(
                        |_trigger: On<Activate>,
                         mut next_overlay: ResMut<NextState<MenuOverlay>>,
                         mut origin: ResMut<SettingsOrigin>| {
                            origin.0 = SettingsFrom::Paused;
                            next_overlay.set(MenuOverlay::Settings);
                        },
                    );

                buttons
                    .spawn(button(
                        ButtonProps::default(),
                        (),
                        (Spawn((
                            Text::new("Quit to Menu"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                        )),),
                    ))
                    .observe(
                        |_trigger: On<Activate>,
                         mut next_overlay: ResMut<NextState<MenuOverlay>>,
                         mut next_state: ResMut<NextState<GameState>>| {
                            next_overlay.set(MenuOverlay::None);
                            next_state.set(GameState::MainMenu);
                        },
                    );
            });
        });
}

// ---- Settings Menu ----

pub fn spawn_settings_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            DespawnOnExit(MenuOverlay::Settings),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Settings"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
            ));

            parent.spawn((
                Text::new("Nothing here yet..."),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Button container (fixed width for consistency)
            parent.spawn(Node {
                width: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            }).with_children(|buttons| {
                buttons
                    .spawn(button(
                        ButtonProps::default(),
                        (),
                        (Spawn((
                            Text::new("Back"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                        )),),
                    ))
                    .observe(
                        |_trigger: On<Activate>,
                         mut next_overlay: ResMut<NextState<MenuOverlay>>,
                         origin: Res<SettingsOrigin>| {
                            match origin.0 {
                                SettingsFrom::Paused => next_overlay.set(MenuOverlay::Paused),
                                SettingsFrom::MainMenu => next_overlay.set(MenuOverlay::None),
                            }
                        },
                    );
            });
        });
}

// ---- ESC key handler ----

pub fn handle_esc_key(
    keys: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    overlay_state: Res<State<MenuOverlay>>,
    mut next_overlay: ResMut<NextState<MenuOverlay>>,
    origin: Res<SettingsOrigin>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    match overlay_state.get() {
        MenuOverlay::None => {
            if *game_state.get() == GameState::Playing {
                next_overlay.set(MenuOverlay::Paused);
            }
        }
        MenuOverlay::Paused => {
            next_overlay.set(MenuOverlay::None);
        }
        MenuOverlay::Settings => match origin.0 {
            SettingsFrom::Paused => next_overlay.set(MenuOverlay::Paused),
            SettingsFrom::MainMenu => next_overlay.set(MenuOverlay::None),
        },
    }
}
