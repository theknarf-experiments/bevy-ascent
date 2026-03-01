use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;

pub fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut player_query: Query<&mut GridPos, With<Player>>,
    blocking_query: Query<(Entity, &GridPos), (With<Blocking>, Without<Player>, Without<Pushable>)>,
    mut pushable_query: Query<
        (Entity, &mut GridPos, Option<&Blocking>),
        (With<Pushable>, Without<Player>),
    >,
    all_pos_query: Query<&GridPos, (Without<Player>, Without<Pushable>)>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
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

    // Check if there's a pushable entity at the target
    let pushable_at_target = pushable_query
        .iter()
        .find(|(_, pos, _)| pos.0 == target)
        .map(|(e, _, _)| e);

    if let Some(push_entity) = pushable_at_target {
        let push_dest = target + dir;
        // Check push destination is free of all blocking (walls + other pushable-blocking)
        let dest_blocked_wall = blocking_query.iter().any(|(_, gp)| gp.0 == push_dest);
        let dest_blocked_pushable = pushable_query
            .iter()
            .any(|(e, pos, _)| e != push_entity && pos.0 == push_dest);
        let dest_occupied = all_pos_query.iter().any(|gp| gp.0 == push_dest);

        if !dest_blocked_wall && !dest_blocked_pushable && !dest_occupied {
            if let Ok((_, mut push_pos, _)) = pushable_query.get_mut(push_entity) {
                push_pos.0 = push_dest;
            }
            player_pos.0 = target;
            next_phase.set(TurnPhase::EnemyTurn);
        } else {
            // Can't push; check if pushable is non-blocking (can walk through it)
            let pushable_is_blocking = pushable_query
                .get(push_entity)
                .map(|(_, _, b)| b.is_some())
                .unwrap_or(false);
            if !pushable_is_blocking {
                player_pos.0 = target;
                next_phase.set(TurnPhase::EnemyTurn);
            } else {
                // Blocked — flash the blocking pushable
                flash_entity(&mut commands, push_entity);
            }
        }
    } else {
        // No pushable at target — check if blocked by wall or enemy
        let blocked_wall = blocking_query.iter().find(|(_, gp)| gp.0 == target);
        let blocked_pushable = pushable_query
            .iter()
            .find(|(_, pos, b)| pos.0 == target && b.is_some());
        if blocked_wall.is_none() && blocked_pushable.is_none() {
            player_pos.0 = target;
            next_phase.set(TurnPhase::EnemyTurn);
        } else {
            // Flash the blocker
            if let Some((entity, _)) = blocked_wall {
                flash_entity(&mut commands, entity);
            }
            if let Some((entity, _, _)) = blocked_pushable {
                flash_entity(&mut commands, entity);
            }
        }
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
    mut tags_query: Query<(Entity, &mut Tags, &DerivedTags)>,
    ice_query: Query<(Entity, &DerivedTags, &GridPos), With<Blocking>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
) {
    // Melted → despawn ice, spawn water at same position
    // (do this first so we don't conflict with health despawns)
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

    // TakingDamage → decrement Health
    for (entity, mut health, derived) in health_query.iter_mut() {
        if derived.0.contains(&Tag::TakingDamage) {
            health.0 -= 1;
            if health.0 <= 0 {
                commands.entity(entity).despawn();
            }
        }
    }

    // Extinguished → remove OnFire from base Tags
    for (_, mut tags, derived) in tags_query.iter_mut() {
        if derived.0.contains(&Tag::Extinguished) {
            tags.0.remove(&Tag::OnFire);
        }
    }

    next_phase.set(TurnPhase::WaitingForInput);
}

pub fn check_win(
    player_query: Query<&GridPos, With<Player>>,
    exit_query: Query<&GridPos, With<Exit>>,
    enemy_query: Query<(), With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(player_pos) = player_query.single() else {
        return;
    };

    // Win: player on exit
    for exit_pos in exit_query.iter() {
        if player_pos.0 == exit_pos.0 {
            next_state.set(GameState::Victory);
            return;
        }
    }

    // Win: all enemies dead
    if enemy_query.iter().count() == 0 {
        next_state.set(GameState::Victory);
    }
}

pub fn check_loss(
    player_query: Query<(), With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if player_query.iter().count() == 0 {
        next_state.set(GameState::GameOver);
    }
}
