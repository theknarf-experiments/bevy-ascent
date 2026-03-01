use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;
use crate::datalog::enemy_name;
use crate::items::spawn_item;
use crate::level_gen::GeneratedFloors;

/// Simple deterministic hash from entity bits + salt for drop rolls.
fn entity_hash(entity: Entity, salt: u32) -> u32 {
    let bits = entity.to_bits();
    let mut h = (bits as u32) ^ ((bits >> 32) as u32);
    h = h.wrapping_add(salt.wrapping_mul(2654435761));
    h ^= h >> 16;
    h = h.wrapping_mul(2246822519);
    h ^= h >> 13;
    h
}

pub fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<PendingAction>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
) {
    // Movement keys
    let dir = if keys.just_pressed(KeyCode::ArrowUp) {
        Some(IVec2::new(0, -1))
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        Some(IVec2::new(0, 1))
    } else if keys.just_pressed(KeyCode::ArrowLeft) {
        Some(IVec2::new(-1, 0))
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        Some(IVec2::new(1, 0))
    } else {
        None
    };

    if let Some(d) = dir {
        pending.0 = Some(PlayerAction::Move(d));
        next_phase.set(TurnPhase::PlayerResolve);
        return;
    }

    // Consumable keys
    let slot = if keys.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else if keys.just_pressed(KeyCode::Digit4) {
        Some(3)
    } else {
        None
    };

    if let Some(s) = slot {
        pending.0 = Some(PlayerAction::UseConsumable(s));
        next_phase.set(TurnPhase::PlayerResolve);
    }
}

pub fn tick_flash_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FlashTimer)>,
) {
    for (entity, mut flash) in query.iter_mut() {
        flash.0.tick(time.delta());
        if flash.0.is_finished() {
            commands.entity(entity).remove::<FlashTimer>();
        }
    }
}

pub fn apply_consequences(
    mut commands: Commands,
    mut health_query: Query<(
        Entity,
        &mut Health,
        &DerivedTags,
        Option<&DropTable>,
        Option<&GridPos>,
        Option<&Boss>,
    )>,
    mut tags_query: Query<(Entity, &mut Tags, &DerivedTags, &GridPos, Option<&Boss>)>,
    ice_query: Query<(Entity, &DerivedTags, &GridPos), With<Blocking>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    player_inventory: Query<&Inventory, With<Player>>,
    armor_query: Query<&ArmorDefense, With<Item>>,
    player_query: Query<Entity, With<Player>>,
    mut game_log: ResMut<GameLog>,
    mut death_cause: ResMut<DeathCause>,
) {
    // Pre-collect entity names for log messages (before mutable borrows)
    let entity_names: Vec<(Entity, &'static str)> = tags_query
        .iter()
        .map(|(e, tags, _, _, boss)| (e, enemy_name(boss, Some(tags))))
        .collect();

    // 1. Explosions first: collect explosion centers, then apply OnFire
    let exploding: Vec<(Entity, IVec2)> = tags_query
        .iter()
        .filter(|(_, _, derived, _, _)| derived.0.contains(&Tag::Exploding))
        .map(|(entity, _, _, pos, _)| (entity, pos.0))
        .collect();

    if !exploding.is_empty() {
        let all_positions: Vec<(Entity, IVec2)> = tags_query
            .iter()
            .map(|(e, _, _, pos, _)| (e, pos.0))
            .collect();

        for (exploding_entity, explosion_pos) in &exploding {
            commands.entity(*exploding_entity).despawn();
            for (other_entity, other_pos) in &all_positions {
                if *other_entity == *exploding_entity {
                    continue;
                }
                let dist =
                    (other_pos.x - explosion_pos.x).abs() + (other_pos.y - explosion_pos.y).abs();
                if dist <= 2 {
                    if let Ok((_, mut tags, _, _, _)) = tags_query.get_mut(*other_entity) {
                        tags.0.insert(Tag::OnFire);
                    }
                }
            }
        }
    }

    // 2. Melted → despawn ice, spawn water at same position
    let melted: Vec<(Entity, IVec2)> = ice_query
        .iter()
        .filter(|(_, derived, _)| derived.0.contains(&Tag::Melted))
        .map(|(entity, _, pos)| (entity, pos.0))
        .collect();

    for (entity, pos) in melted {
        commands.entity(entity).despawn();
        commands.spawn((
            GridPos(pos),
            Tags(BTreeSet::from([Tag::Wet])),
            DerivedTags::default(),
            DespawnOnExit(GameState::Playing),
        ));
    }

    // Get player's armor defense for damage reduction
    let player_entity = player_query.single().ok();
    let armor_def = player_inventory
        .single()
        .ok()
        .and_then(|inv| inv.armor)
        .and_then(|e| armor_query.get(e).ok())
        .map(|a| a.0)
        .unwrap_or(0);

    // 3. All damage types → decrement Health (stacking)
    for (entity, mut health, derived, drop_table, grid_pos, _boss) in health_query.iter_mut() {
        let has_fire = derived.0.contains(&Tag::FireDamage);
        let has_poison = derived.0.contains(&Tag::PoisonDamage);
        let has_electric = derived.0.contains(&Tag::ElectricDamage);
        let mut dmg = 0;
        if has_fire {
            dmg += 1;
        }
        if has_poison {
            dmg += 1;
        }
        if has_electric {
            dmg += 1;
        }
        if dmg > 0 {
            let is_player = player_entity.is_some_and(|pe| pe == entity);
            if is_player {
                dmg = (dmg - armor_def).max(0);
                if has_fire {
                    game_log.push("You take fire damage!");
                }
                if has_poison {
                    game_log.push("You take poison damage!");
                }
                if has_electric {
                    game_log.push("You take electric damage!");
                }
            }
            health.0 -= dmg;
            if health.0 <= 0 {
                if is_player {
                    if has_fire {
                        death_cause.0 = Some("Burned to death".to_string());
                    } else if has_poison {
                        death_cause.0 = Some("Killed by poison".to_string());
                    } else if has_electric {
                        death_cause.0 = Some("Electrocuted".to_string());
                    }
                } else {
                    let name = entity_names
                        .iter()
                        .find(|(e, _)| *e == entity)
                        .map(|(_, n)| *n)
                        .unwrap_or("an enemy");
                    if has_fire {
                        game_log.push(format!("{} burns to death!", name));
                    } else if has_poison {
                        game_log.push(format!("{} succumbs to poison!", name));
                    } else if has_electric {
                        game_log.push(format!("{} is electrocuted!", name));
                    }
                }
                if let (Some(dt), Some(pos)) = (drop_table, grid_pos) {
                    let drop_pos = pos.0;
                    let dt_clone = dt.0.clone();
                    for (i, (kind, chance)) in dt_clone.iter().enumerate() {
                        let roll = entity_hash(entity, i as u32) % 100;
                        if roll < *chance {
                            spawn_item(&mut commands, *kind, drop_pos);
                        }
                    }
                }
                // Don't despawn the player — keep them visible on the grid
                if !is_player {
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    // 4. Extinguished → remove OnFire from base Tags
    // 5. PoisonBurned → remove Poisoned from base Tags
    for (_, mut tags, derived, _, _) in tags_query.iter_mut() {
        if derived.0.contains(&Tag::Extinguished) {
            tags.0.remove(&Tag::OnFire);
            tags.0.remove(&Tag::FireSource);
        }
        if derived.0.contains(&Tag::PoisonBurned) {
            tags.0.remove(&Tag::Poisoned);
        }
    }

    next_phase.set(TurnPhase::WaitingForInput);
}

pub fn check_win(
    player_query: Query<&GridPos, With<Player>>,
    exit_query: Query<&GridPos, With<Exit>>,
    mut victory: ResMut<VictoryAchieved>,
) {
    let Ok(player_pos) = player_query.single() else {
        return;
    };

    for exit_pos in exit_query.iter() {
        if player_pos.0 == exit_pos.0 {
            victory.0 = true;
            return;
        }
    }
}

pub fn handle_floor_transition(
    mut commands: Commands,
    mut transition: ResMut<FloorTransition>,
    floor_entities: Query<Entity, With<FloorEntity>>,
    mut player_query: Query<(Entity, &mut GridPos), With<Player>>,
    mut current_floor: ResMut<CurrentFloor>,
    generated: Res<GeneratedFloors>,
    mut fog_map: ResMut<FogMap>,
    mut game_log: ResMut<GameLog>,
) {
    let Some(going_down) = transition.0.take() else {
        return;
    };

    let new_floor = if going_down {
        current_floor.0 + 1
    } else {
        current_floor.0.saturating_sub(1).max(1)
    };

    fog_map.reset();

    for entity in floor_entities.iter() {
        commands.entity(entity).despawn();
    }

    let result =
        crate::level::spawn_floor(&mut commands, &generated.floors[(new_floor - 1) as usize]);

    if let Ok((player_entity, mut player_pos)) = player_query.single_mut() {
        commands.entity(player_entity).remove::<FireResistBuff>();
        if going_down {
            if let Some(pos) = result.stairs_up_pos {
                player_pos.0 = pos;
            }
        } else {
            if let Some(pos) = result.stairs_down_pos {
                player_pos.0 = pos;
            }
        }
    }

    current_floor.0 = new_floor;

    if going_down {
        game_log.push(format!("Descended to floor {}", new_floor));
    } else {
        game_log.push(format!("Ascended to floor {}", new_floor));
    }
}

pub fn reset_game_resources(
    mut victory: ResMut<VictoryAchieved>,
    mut floor: ResMut<CurrentFloor>,
    mut gold: ResMut<GoldCount>,
    mut fog_map: ResMut<FogMap>,
    mut game_log: ResMut<GameLog>,
    mut death_cause: ResMut<DeathCause>,
    mut pending: ResMut<PendingAction>,
) {
    victory.0 = false;
    floor.0 = 0;
    gold.0 = 0;
    fog_map.reset();
    game_log.clear();
    death_cause.0 = None;
    pending.0 = None;
}

pub fn check_loss(
    player_query: Query<&Health, With<Player>>,
    mut next_overlay: ResMut<NextState<MenuOverlay>>,
) {
    let Ok(health) = player_query.single() else {
        return;
    };
    if health.0 <= 0 {
        next_overlay.set(MenuOverlay::GameOver);
    }
}
