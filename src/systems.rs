use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;
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
    mut commands: Commands,
    mut player_query: Query<(&mut GridPos, &Inventory), With<Player>>,
    blocking_query: Query<
        (Entity, &GridPos),
        (With<Blocking>, Without<Player>, Without<Pushable>, Without<Enemy>, Without<Chest>),
    >,
    mut pushable_query: Query<
        (Entity, &mut GridPos, Option<&Blocking>),
        (With<Pushable>, Without<Player>),
    >,
    all_pos_query: Query<&GridPos, (Without<Player>, Without<Pushable>)>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    stairs_down: Query<&GridPos, (With<StairsDown>, Without<Player>, Without<Pushable>)>,
    stairs_up: Query<&GridPos, (With<StairsUp>, Without<Player>, Without<Pushable>)>,
    mut floor_transition: ResMut<FloorTransition>,
    mut player_moved: ResMut<PlayerMoved>,
    mut enemy_combat: Query<
        (Entity, &GridPos, &mut Health, Option<&mut Tags>, Option<&DropTable>),
        (With<Enemy>, Without<Player>, Without<Pushable>, Without<Chest>, Without<Item>),
    >,
    weapon_query: Query<(&WeaponDamage, Option<&Tags>), (With<Item>, Without<Enemy>, Without<Player>)>,
    chest_query: Query<(Entity, &GridPos), (With<Chest>, Without<Player>, Without<Pushable>, Without<Enemy>, Without<Item>)>,
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

    let Ok((mut player_pos, inventory)) = player_query.single_mut() else {
        return;
    };

    let target = player_pos.0 + dir;

    // Check if there's an enemy at the target — melee attack
    if let Some((enemy_entity, _, mut enemy_hp, mut enemy_tags, drop_table)) =
        enemy_combat.iter_mut().find(|(_, gp, _, _, _)| gp.0 == target)
    {
        // Calculate damage: base 1 + weapon bonus
        let mut total_damage = 1;
        let mut weapon_tags_to_apply: Vec<Tag> = Vec::new();

        if let Some(weapon_entity) = inventory.weapon {
            if let Ok((wpn_dmg, wpn_tags)) = weapon_query.get(weapon_entity) {
                total_damage += wpn_dmg.0;
                if let Some(tags) = wpn_tags {
                    if tags.0.contains(&Tag::Poisoned) {
                        weapon_tags_to_apply.push(Tag::Poisoned);
                    }
                    if tags.0.contains(&Tag::OnFire) {
                        weapon_tags_to_apply.push(Tag::OnFire);
                    }
                }
            }
        }

        enemy_hp.0 -= total_damage;

        // Apply on-hit effects
        if let Some(ref mut enemy_tags) = enemy_tags {
            for tag in &weapon_tags_to_apply {
                enemy_tags.0.insert(*tag);
            }
        }

        if enemy_hp.0 <= 0 {
            // Enemy drops
            let drop_pos = target;
            if let Some(dt) = drop_table {
                let dt_clone = dt.0.clone();
                for (i, (kind, chance)) in dt_clone.iter().enumerate() {
                    let roll = entity_hash(enemy_entity, i as u32) % 100;
                    if roll < *chance {
                        spawn_item(&mut commands, *kind, drop_pos);
                    }
                }
            }
            commands.entity(enemy_entity).despawn();
        }
        next_phase.set(TurnPhase::EnemyTurn);
        return;
    }

    // Check if there's a chest at the target — open it
    if let Some((chest_entity, _)) = chest_query.iter().find(|(_, gp)| gp.0 == target) {
        // Open chest: despawn and spawn items
        let chest_pos = target;
        let loot_count = 1 + (entity_hash(chest_entity, 0) % 2) as usize; // 1-2 items
        let item_pool = [
            ItemKind::HealthPotion,
            ItemKind::Antidote,
            ItemKind::FireResistPotion,
            ItemKind::IronSword,
            ItemKind::LeatherArmor,
            ItemKind::Gold,
        ];
        for i in 0..loot_count {
            let idx = entity_hash(chest_entity, (i + 1) as u32) as usize % item_pool.len();
            spawn_item(&mut commands, item_pool[idx], chest_pos);
        }
        commands.entity(chest_entity).despawn();
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
        let dest_blocked_enemy = enemy_combat.iter().any(|(_, gp, _, _, _)| gp.0 == push_dest);
        let dest_blocked_pushable = pushable_query
            .iter()
            .any(|(e, pos, _)| e != push_entity && pos.0 == push_dest);
        let dest_occupied = all_pos_query.iter().any(|gp| gp.0 == push_dest);
        let dest_blocked_chest = chest_query.iter().any(|(_, gp)| gp.0 == push_dest);

        if !dest_blocked_wall && !dest_blocked_enemy && !dest_blocked_pushable && !dest_occupied && !dest_blocked_chest {
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
        // No pushable at target — check if blocked by wall or chest
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

    // Player moved to target — flag for pickup system
    player_moved.0 = true;

    // Check for stairs
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

pub fn pickup_items(
    mut commands: Commands,
    mut player_query: Query<(&GridPos, &mut Inventory), With<Player>>,
    item_query: Query<(Entity, &GridPos, &ItemKind, Option<&Equippable>, Option<&Consumable>), (With<Item>, Without<Player>)>,
    mut gold_count: ResMut<GoldCount>,
    mut player_moved: ResMut<PlayerMoved>,
) {
    if !player_moved.0 {
        return;
    }
    player_moved.0 = false;

    let Ok((player_pos, mut inventory)) = player_query.single_mut() else {
        return;
    };

    let items_here: Vec<_> = item_query
        .iter()
        .filter(|(_, gp, _, _, _)| gp.0 == player_pos.0)
        .map(|(e, _, kind, equip, consumable)| (e, *kind, equip.map(|eq| eq.0), consumable.is_some()))
        .collect();

    for (entity, kind, equip_slot, is_consumable) in items_here {
        if kind == ItemKind::Gold {
            gold_count.0 += 1;
            commands.entity(entity).despawn();
            continue;
        }

        if let Some(slot) = equip_slot {
            match slot {
                EquipSlot::Weapon => {
                    if let Some(old_weapon) = inventory.weapon.take() {
                        // Drop old weapon at player position
                        commands.entity(old_weapon).insert((
                            GridPos(player_pos.0),
                            FloorEntity,
                        ));
                    }
                    inventory.weapon = Some(entity);
                    commands.entity(entity).remove::<GridPos>();
                    commands.entity(entity).remove::<FloorEntity>();
                }
                EquipSlot::Armor => {
                    if let Some(old_armor) = inventory.armor.take() {
                        commands.entity(old_armor).insert((
                            GridPos(player_pos.0),
                            FloorEntity,
                        ));
                    }
                    inventory.armor = Some(entity);
                    commands.entity(entity).remove::<GridPos>();
                    commands.entity(entity).remove::<FloorEntity>();
                }
            }
            continue;
        }

        if is_consumable {
            if inventory.consumables.len() < 4 {
                inventory.consumables.push(entity);
                commands.entity(entity).remove::<GridPos>();
                commands.entity(entity).remove::<FloorEntity>();
            }
            // If full, item stays on ground
            continue;
        }
    }
}

pub fn use_consumable(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Inventory, &mut Health, &mut Tags), (With<Player>, Without<Item>)>,
    item_kind_query: Query<&ItemKind, (With<Item>, Without<Player>)>,
) {
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

    let Some(slot_idx) = slot else {
        return;
    };

    let Ok((player_entity, mut inventory, mut health, mut tags)) = player_query.single_mut() else {
        return;
    };

    if slot_idx >= inventory.consumables.len() {
        return;
    }

    let item_entity = inventory.consumables[slot_idx];
    let Ok(kind) = item_kind_query.get(item_entity) else {
        return;
    };

    match kind {
        ItemKind::HealthPotion => {
            health.0 = (health.0 + 3).min(10);
        }
        ItemKind::Antidote => {
            tags.0.remove(&Tag::Poisoned);
        }
        ItemKind::FireResistPotion => {
            commands.entity(player_entity).insert(FireResistBuff);
        }
        _ => return,
    }

    inventory.consumables.remove(slot_idx);
    commands.entity(item_entity).despawn();
}

pub fn enemy_turn(
    player_query: Query<(&GridPos, &Inventory), (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(Entity, &mut GridPos, &mut Health), (With<Enemy>, Without<Player>)>,
    mut player_health: Query<&mut Health, (With<Player>, Without<Enemy>)>,
    blocking_query: Query<(Entity, &GridPos), (With<Blocking>, Without<Enemy>, Without<Player>)>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    armor_query: Query<&ArmorDefense, (With<Item>, Without<Player>, Without<Enemy>)>,
) {
    let Ok((player_pos, inventory)) = player_query.single() else {
        next_phase.set(TurnPhase::ResolveEnvironment);
        return;
    };
    let player_tile = player_pos.0;

    // Get player armor defense
    let armor_def = inventory.armor
        .and_then(|e| armor_query.get(e).ok())
        .map(|a| a.0)
        .unwrap_or(0);

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
                let dmg = (1 - armor_def).max(0);
                ph.0 -= dmg;
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
    mut health_query: Query<(Entity, &mut Health, &DerivedTags, Option<&DropTable>, Option<&GridPos>)>,
    mut tags_query: Query<(Entity, &mut Tags, &DerivedTags, &GridPos)>,
    ice_query: Query<(Entity, &DerivedTags, &GridPos), With<Blocking>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    player_inventory: Query<&Inventory, With<Player>>,
    armor_query: Query<&ArmorDefense, With<Item>>,
    player_query: Query<Entity, With<Player>>,
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

    // Get player's armor defense for damage reduction
    let player_entity = player_query.single().ok();
    let armor_def = player_inventory.single().ok()
        .and_then(|inv| inv.armor)
        .and_then(|e| armor_query.get(e).ok())
        .map(|a| a.0)
        .unwrap_or(0);

    // 3. All damage types → decrement Health (stacking)
    for (entity, mut health, derived, drop_table, grid_pos) in health_query.iter_mut() {
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
            // Apply armor defense if this is the player
            let is_player = player_entity.is_some_and(|pe| pe == entity);
            if is_player {
                dmg = (dmg - armor_def).max(0);
            }
            health.0 -= dmg;
            if health.0 <= 0 {
                // Drop loot if entity has a drop table
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
    mut player_query: Query<(Entity, &mut GridPos), With<Player>>,
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

    // Remove FireResistBuff on floor transition, reposition player
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
}

pub fn reset_game_resources(
    mut victory: ResMut<VictoryAchieved>,
    mut floor: ResMut<CurrentFloor>,
    mut gold: ResMut<GoldCount>,
) {
    victory.0 = false;
    floor.0 = 0;
    gold.0 = 0;
}

pub fn check_loss(
    player_query: Query<(), With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if player_query.iter().count() == 0 {
        next_state.set(GameState::GameOver);
    }
}
