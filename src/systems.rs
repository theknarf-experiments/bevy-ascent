use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;
use crate::level_gen::GeneratedFloors;

pub fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut player_query: Query<&mut GridPos, With<Player>>,
    blocking_query: Query<(Entity, &GridPos), (With<Blocking>, Without<Player>, Without<Pushable>, Without<Enemy>)>,
    mut pushable_query: Query<
        (Entity, &mut GridPos, Option<&Blocking>),
        (With<Pushable>, Without<Player>),
    >,
    all_pos_query: Query<&GridPos, (Without<Player>, Without<Pushable>)>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    stairs_down: Query<&GridPos, (With<StairsDown>, Without<Player>, Without<Pushable>)>,
    stairs_up: Query<&GridPos, (With<StairsUp>, Without<Player>, Without<Pushable>)>,
    mut floor_transition: ResMut<FloorTransition>,
    mut enemy_combat: Query<(Entity, &GridPos, &mut Health), (With<Enemy>, Without<Player>, Without<Pushable>)>,
) {
    let dir = if keys.just_pressed(KeyCode::ArrowUp) {
        IVec2::new(0, -1)
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        IVec2::new(0, 1)
    } else if keys.just_pressed(KeyCode::ArrowLeft) {
        IVec2::new(-1, 0)
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        IVec2::new(1, 0)
    } else {
        return;
    };

    let Ok(mut player_pos) = player_query.single_mut() else {
        return;
    };

    let target = player_pos.0 + dir;

    // Check if there's an enemy at the target — melee attack
    if let Some((enemy_entity, _, mut enemy_hp)) =
        enemy_combat.iter_mut().find(|(_, gp, _)| gp.0 == target)
    {
        enemy_hp.0 -= 1;
        if enemy_hp.0 <= 0 {
            commands.entity(enemy_entity).despawn();
        }
        next_phase.set(TurnPhase::EnemyTurn);
        return;
    }

    // Check if there's a pushable entity at the target
    let pushable_at_target = pushable_query
        .iter()
        .find(|(_, pos, _)| pos.0 == target)
        .map(|(e, _, _)| e);

    if let Some(push_entity) = pushable_at_target {
        let push_dest = target + dir;
        // Check push destination is free of all blocking (walls + other pushable-blocking + enemies)
        let dest_blocked_wall = blocking_query.iter().any(|(_, gp)| gp.0 == push_dest);
        let dest_blocked_enemy = enemy_combat.iter().any(|(_, gp, _)| gp.0 == push_dest);
        let dest_blocked_pushable = pushable_query
            .iter()
            .any(|(e, pos, _)| e != push_entity && pos.0 == push_dest);
        let dest_occupied = all_pos_query.iter().any(|gp| gp.0 == push_dest);

        if !dest_blocked_wall && !dest_blocked_enemy && !dest_blocked_pushable && !dest_occupied {
            if let Ok((_, mut push_pos, _)) = pushable_query.get_mut(push_entity) {
                push_pos.0 = push_dest;
            }
            player_pos.0 = target;
        } else {
            // Can't push; check if pushable is non-blocking (can walk through it)
            let pushable_is_blocking = pushable_query
                .get(push_entity)
                .map(|(_, _, b)| b.is_some())
                .unwrap_or(false);
            if !pushable_is_blocking {
                player_pos.0 = target;
            } else {
                // Blocked — flash the blocking pushable
                flash_entity(&mut commands, push_entity);
                return;
            }
        }
    } else {
        // No pushable at target — check if blocked by wall
        let blocked_wall = blocking_query.iter().find(|(_, gp)| gp.0 == target);
        let blocked_pushable = pushable_query
            .iter()
            .find(|(_, pos, b)| pos.0 == target && b.is_some());
        if blocked_wall.is_none() && blocked_pushable.is_none() {
            player_pos.0 = target;
        } else {
            // Flash the blocker
            if let Some((entity, _)) = blocked_wall {
                flash_entity(&mut commands, entity);
            }
            if let Some((entity, _, _)) = blocked_pushable {
                flash_entity(&mut commands, entity);
            }
            return;
        }
    }

    // Player moved to target — check for stairs
    let on_stairs_down = stairs_down.iter().any(|gp| gp.0 == target);
    let on_stairs_up = stairs_up.iter().any(|gp| gp.0 == target);

    if on_stairs_down {
        floor_transition.0 = Some(true);
    } else if on_stairs_up {
        floor_transition.0 = Some(false);
    } else {
        next_phase.set(TurnPhase::EnemyTurn);
    }
}

fn flash_entity(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .insert(FlashTimer(Timer::from_seconds(0.15, TimerMode::Once)));
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

pub fn enemy_turn(
    player_query: Query<&GridPos, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(Entity, &mut GridPos, &mut Health), With<Enemy>>,
    mut player_health: Query<&mut Health, (With<Player>, Without<Enemy>)>,
    blocking_query: Query<(Entity, &GridPos), (With<Blocking>, Without<Enemy>, Without<Player>)>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
) {
    let Ok(player_pos) = player_query.single() else {
        next_phase.set(TurnPhase::ResolveEnvironment);
        return;
    };
    let player_tile = player_pos.0;

    // Collect all blocking positions (non-enemy)
    let static_blocking: Vec<IVec2> = blocking_query.iter().map(|(_, pos)| pos.0).collect();

    // Collect current enemy positions for collision avoidance
    let enemy_positions: Vec<(Entity, IVec2)> = enemy_query
        .iter()
        .map(|(e, pos, _)| (e, pos.0))
        .collect();

    // Compute moves for each enemy
    let mut moves: Vec<(Entity, IVec2)> = Vec::new();

    for (entity, pos, _) in enemy_query.iter() {
        let epos = pos.0;
        let diff = player_tile - epos;

        // If adjacent to player, attack
        if diff.x.abs() + diff.y.abs() == 1 {
            if let Ok(mut ph) = player_health.single_mut() {
                ph.0 -= 1;
            }
            continue;
        }

        // Greedy chase
        let candidates = [
            IVec2::new(0, -1),
            IVec2::new(0, 1),
            IVec2::new(-1, 0),
            IVec2::new(1, 0),
        ];

        let current_dist = diff.x.abs() + diff.y.abs();
        let mut best_dir = None;
        let mut best_dist = current_dist;

        for dir in candidates {
            let new_pos = epos + dir;
            let new_dist =
                (player_tile - new_pos).x.abs() + (player_tile - new_pos).y.abs();
            if new_dist < best_dist {
                let blocked_static = static_blocking.iter().any(|bp| *bp == new_pos);
                let blocked_enemy = enemy_positions
                    .iter()
                    .any(|(e, bp)| *e != entity && *bp == new_pos);
                let already_claimed = moves.iter().any(|(_, to)| *to == new_pos);
                if !blocked_static && !blocked_enemy && !already_claimed && new_pos != player_tile {
                    best_dir = Some(dir);
                    best_dist = new_dist;
                }
            }
        }

        if let Some(dir) = best_dir {
            moves.push((entity, epos + dir));
        }
    }

    // Apply moves
    for (mut_entity, mut pos, _) in enemy_query.iter_mut() {
        if let Some((_, to)) = moves.iter().find(|(e, _)| *e == mut_entity) {
            pos.0 = *to;
        }
    }

    next_phase.set(TurnPhase::ResolveEnvironment);
}

pub fn apply_consequences(
    mut commands: Commands,
    mut health_query: Query<(Entity, &mut Health, &DerivedTags)>,
    mut tags_query: Query<(Entity, &mut Tags, &DerivedTags, &GridPos)>,
    ice_query: Query<(Entity, &DerivedTags, &GridPos), With<Blocking>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
) {
    // 1. Explosions first: collect explosion centers, then apply OnFire
    let exploding: Vec<(Entity, IVec2)> = tags_query
        .iter()
        .filter(|(_, _, derived, _)| derived.0.contains(&Tag::Exploding))
        .map(|(entity, _, _, pos)| (entity, pos.0))
        .collect();

    if !exploding.is_empty() {
        // Collect all entities + positions that could be affected
        let all_positions: Vec<(Entity, IVec2)> = tags_query
            .iter()
            .map(|(e, _, _, pos)| (e, pos.0))
            .collect();

        for (exploding_entity, explosion_pos) in &exploding {
            commands.entity(*exploding_entity).despawn();
            for (other_entity, other_pos) in &all_positions {
                if *other_entity == *exploding_entity {
                    continue;
                }
                let dist = (other_pos.x - explosion_pos.x).abs()
                    + (other_pos.y - explosion_pos.y).abs();
                if dist <= 2 {
                    if let Ok((_, mut tags, _, _)) = tags_query.get_mut(*other_entity) {
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

    // 3. All damage types → decrement Health (stacking)
    for (entity, mut health, derived) in health_query.iter_mut() {
        let mut dmg = 0;
        if derived.0.contains(&Tag::FireDamage) {
            dmg += 1;
        }
        if derived.0.contains(&Tag::PoisonDamage) {
            dmg += 1;
        }
        if derived.0.contains(&Tag::ElectricDamage) {
            dmg += 1;
        }
        if dmg > 0 {
            health.0 -= dmg;
            if health.0 <= 0 {
                commands.entity(entity).despawn();
            }
        }
    }

    // 4. Extinguished → remove OnFire from base Tags
    // 5. PoisonBurned → remove Poisoned from base Tags
    for (_, mut tags, derived, _) in tags_query.iter_mut() {
        if derived.0.contains(&Tag::Extinguished) {
            tags.0.remove(&Tag::OnFire);
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
    mut player_query: Query<&mut GridPos, With<Player>>,
    mut current_floor: ResMut<CurrentFloor>,
    generated: Res<GeneratedFloors>,
) {
    let Some(going_down) = transition.0.take() else {
        return;
    };

    let new_floor = if going_down {
        current_floor.0 + 1
    } else {
        current_floor.0.saturating_sub(1).max(1)
    };

    // Despawn all floor entities
    for entity in floor_entities.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn new floor
    let result = crate::level::spawn_floor(&mut commands, &generated.floors[(new_floor - 1) as usize]);

    // Reposition player: going down → appear at stairs up; going up → appear at stairs down
    if let Ok(mut player_pos) = player_query.single_mut() {
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
}

pub fn reset_game_resources(
    mut victory: ResMut<VictoryAchieved>,
    mut floor: ResMut<CurrentFloor>,
) {
    victory.0 = false;
    floor.0 = 0;
}

pub fn check_loss(
    player_query: Query<(), With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if player_query.iter().count() == 0 {
        next_state.set(GameState::GameOver);
    }
}
