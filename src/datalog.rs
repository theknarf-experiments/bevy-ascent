use ascent::ascent_run;
use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;
use crate::items::{item_name, spawn_item};

/// Return a display name for an enemy based on Boss marker and Tags.
pub fn enemy_name(boss: Option<&Boss>, tags: Option<&Tags>) -> &'static str {
    if boss.is_some() {
        return "Dragon";
    }
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
    "Goblin"
}

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

fn flash_entity(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .insert(FlashTimer(Timer::from_seconds(0.15, TimerMode::Once)));
}

/// Returns the cardinal direction (unit vector) that best aligns with diff.
fn best_cardinal_direction(diff: IVec2) -> IVec2 {
    if diff.x.abs() >= diff.y.abs() {
        IVec2::new(diff.x.signum(), 0)
    } else {
        IVec2::new(0, diff.y.signum())
    }
}

// ---------------------------------------------------------------------------
// resolve_environment — environmental Datalog rules
// ---------------------------------------------------------------------------

pub fn resolve_environment(
    mut commands: Commands,
    query: Query<(Entity, &Tags, &GridPos)>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    player_inventory: Query<(Entity, &Inventory), With<Player>>,
    item_tags: Query<&Tags, With<Item>>,
    fire_resist_query: Query<(), With<FireResistBuff>>,
) {
    // Gather all (Entity, Tag) pairs
    let mut has_tag: Vec<(Entity, Tag)> = query
        .iter()
        .flat_map(|(entity, tags, _)| tags.0.iter().map(move |&tag| (entity, tag)))
        .collect();

    // Inject equipped item tags onto the player entity
    if let Ok((player_entity, inventory)) = player_inventory.single() {
        for slot_entity in [inventory.weapon, inventory.armor].iter().flatten() {
            if let Ok(slot_tags) = item_tags.get(*slot_entity) {
                for &tag in &slot_tags.0 {
                    has_tag.push((player_entity, tag));
                }
            }
        }
        if fire_resist_query.get(player_entity).is_ok() {
            has_tag.push((player_entity, Tag::FireResist));
        }
    }

    // Gather all (Entity, x, y) positions
    let position: Vec<(Entity, i32, i32)> = query
        .iter()
        .map(|(entity, _, pos)| (entity, pos.0.x, pos.0.y))
        .collect();

    let result = ascent_run! {
        relation has_tag(Entity, Tag) = has_tag;
        relation position(Entity, i32, i32) = position;

        // Spatial relations: adjacent (4-directional, Manhattan distance 1)
        relation adjacent(Entity, Entity);
        adjacent(a, b) <--
            position(a, ax, ay),
            position(b, bx, by),
            if a != b && ((*ax - *bx).abs() + (*ay - *by).abs() == 1);

        // Same tile: entities at the same position
        relation same_tile(Entity, Entity);
        same_tile(a, b) <--
            position(a, ax, ay),
            position(b, bx, by),
            if a != b && ax == bx && ay == by;

        // Boolean decomposition
        relation is_flammable(Entity);
        relation is_on_fire(Entity);
        relation is_wet(Entity);
        relation is_flesh(Entity);
        relation is_ice(Entity);
        relation is_poisoned(Entity);
        relation is_electrified(Entity);
        relation is_conductive(Entity);
        relation is_metal(Entity);
        relation is_explosive(Entity);
        relation is_fire_resist(Entity);
        relation is_fire_source(Entity);

        // Base relations (input-only, used for negation / self-immunity)
        relation is_base_wet(Entity);
        relation is_base_on_fire(Entity);
        relation is_base_poisoned(Entity);
        relation is_base_electrified(Entity);

        is_base_wet(e) <-- has_tag(e, t) if *t == Tag::Wet;
        is_base_on_fire(e) <-- has_tag(e, t) if *t == Tag::OnFire;
        is_base_poisoned(e) <-- has_tag(e, t) if *t == Tag::Poisoned;
        is_base_electrified(e) <-- has_tag(e, t) if *t == Tag::Electrified;
        is_fire_resist(e) <-- has_tag(e, t) if *t == Tag::FireResist;
        is_fire_source(e) <-- has_tag(e, t) if *t == Tag::FireSource;

        // Full wet: includes derived wetness (from melting)
        is_wet(e) <-- is_base_wet(e);

        // Material decomposition
        is_flammable(e) <-- has_tag(e, t) if *t == Tag::Wood;
        is_flammable(e) <-- has_tag(e, t) if *t == Tag::Oil;
        is_flesh(e) <-- has_tag(e, t) if *t == Tag::Flesh;
        is_ice(e) <-- has_tag(e, t) if *t == Tag::Ice;
        is_metal(e) <-- has_tag(e, t) if *t == Tag::Metal;
        is_explosive(e) <-- has_tag(e, t) if *t == Tag::Explosive;

        // Base states
        is_on_fire(e) <-- has_tag(e, t) if *t == Tag::OnFire;
        is_poisoned(e) <-- has_tag(e, t) if *t == Tag::Poisoned;
        is_electrified(e) <-- has_tag(e, t) if *t == Tag::Electrified;

        // Conductivity: metal or wet
        is_conductive(e) <-- is_metal(e);
        is_conductive(e) <-- is_wet(e);

        // Derived results
        relation derived(Entity, Tag);
        derived(e, Tag::Flammable) <-- is_flammable(e);
        derived(e, Tag::Conductive) <-- is_conductive(e);

        // === FIRE RULES ===
        // Flammable materials (Wood, Oil) catch fire from adjacent OnFire or FireSource
        derived(e, Tag::OnFire) <-- is_flammable(e), adjacent(e, other), is_on_fire(other), !is_base_wet(e);
        derived(e, Tag::OnFire) <-- is_flammable(e), adjacent(e, other), is_fire_source(other), !is_base_wet(e);
        // Flesh catches fire ONLY from same-tile OnFire (not FireSource, not adjacency)
        derived(e, Tag::OnFire) <-- is_flesh(e), same_tile(e, other), is_on_fire(other), !is_base_wet(e), !is_base_on_fire(e);
        is_on_fire(e) <-- derived(e, t) if *t == Tag::OnFire;

        // Ice melts from adjacent OnFire or FireSource
        derived(e, Tag::Melted) <-- is_ice(e), adjacent(e, other), is_on_fire(other);
        derived(e, Tag::Melted) <-- is_ice(e), adjacent(e, other), is_fire_source(other);
        derived(e, Tag::Wet) <-- derived(e, t) if *t == Tag::Melted;
        is_wet(e) <-- derived(e, t) if *t == Tag::Wet;
        // Water extinguishes both OnFire and FireSource
        derived(e, Tag::Extinguished) <-- is_wet(e), is_on_fire(e);
        derived(e, Tag::Extinguished) <-- is_wet(e), is_fire_source(e);

        // Fire damage: only when flesh itself is on fire (derived, not base = immune)
        derived(e, Tag::FireDamage) <-- is_flesh(e), is_on_fire(e), !is_base_on_fire(e), !is_fire_resist(e);
        // Ice Golem (Ice + Flesh) takes fire damage from melting
        derived(e, Tag::FireDamage) <-- is_flesh(e), derived(e, t) if *t == Tag::Melted;

        // === POISON RULES ===
        // Poison spread blocked by OnFire and FireSource (fire burns away poison)
        derived(e, Tag::Poisoned) <-- adjacent(e, other), is_poisoned(other), !is_base_on_fire(e), !is_fire_source(e);
        is_poisoned(e) <-- derived(e, t) if *t == Tag::Poisoned;
        derived(e, Tag::PoisonDamage) <-- is_flesh(e), adjacent(e, other), is_poisoned(other), !is_base_poisoned(e);
        derived(e, Tag::PoisonDamage) <-- is_flesh(e), same_tile(e, other), is_poisoned(other), !is_base_poisoned(e);
        derived(e, Tag::PoisonBurned) <-- is_poisoned(e), is_on_fire(e);
        derived(e, Tag::PoisonBurned) <-- is_poisoned(e), adjacent(e, other), is_on_fire(other);
        derived(e, Tag::PoisonBurned) <-- is_poisoned(e), is_fire_source(e);
        derived(e, Tag::PoisonBurned) <-- is_poisoned(e), adjacent(e, other), is_fire_source(other);

        // === ELECTRICITY RULES ===
        derived(e, Tag::Electrified) <-- is_conductive(e), adjacent(e, other), is_electrified(other);
        derived(e, Tag::Electrified) <-- is_conductive(e), same_tile(e, other), is_electrified(other);
        is_electrified(e) <-- derived(e, t) if *t == Tag::Electrified;
        derived(e, Tag::ElectricDamage) <-- is_flesh(e), adjacent(e, other), is_electrified(other), !is_base_electrified(e);
        derived(e, Tag::ElectricDamage) <-- is_flesh(e), same_tile(e, other), is_electrified(other), !is_base_electrified(e);

        // === EXPLOSIVE RULES ===
        derived(e, Tag::Exploding) <-- is_explosive(e), is_on_fire(e);
    };

    // Collect derived tags per entity
    let mut derived_map: std::collections::HashMap<Entity, BTreeSet<Tag>> =
        std::collections::HashMap::new();

    for (entity, tag) in &result.derived {
        derived_map.entry(*entity).or_default().insert(*tag);
    }

    // Write back DerivedTags for all queried entities
    for (entity, _, _) in query.iter() {
        let tags = derived_map.remove(&entity).unwrap_or_default();
        commands.entity(entity).insert(DerivedTags(tags));
    }

    next_phase.set(TurnPhase::ApplyConsequences);
}

// ---------------------------------------------------------------------------
// resolve_player_turn — player action resolution
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn resolve_player_turn(
    mut commands: Commands,
    pending: Res<PendingAction>,
    mut player_query: Query<
        (Entity, &mut GridPos, &mut Inventory, &mut Health, &mut Tags),
        With<Player>,
    >,
    blocking_query: Query<
        (Entity, &GridPos),
        (
            With<Blocking>,
            Without<Player>,
            Without<Pushable>,
            Without<Enemy>,
            Without<Chest>,
        ),
    >,
    mut pushable_query: Query<
        (
            Entity,
            &mut GridPos,
            Option<&Blocking>,
            Option<&Tags>,
            Option<&DerivedTags>,
        ),
        (With<Pushable>, Without<Player>),
    >,
    all_pos_query: Query<&GridPos, (Without<Player>, Without<Pushable>)>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    stairs_down: Query<&GridPos, (With<StairsDown>, Without<Player>, Without<Pushable>)>,
    stairs_up: Query<&GridPos, (With<StairsUp>, Without<Player>, Without<Pushable>)>,
    mut floor_transition: ResMut<FloorTransition>,
    mut enemy_combat: Query<
        (
            Entity,
            &GridPos,
            &mut Health,
            Option<&mut Tags>,
            Option<&DropTable>,
            Option<&Boss>,
        ),
        (
            With<Enemy>,
            Without<Player>,
            Without<Pushable>,
            Without<Chest>,
            Without<Item>,
        ),
    >,
    item_stats_query: Query<
        (Option<&WeaponDamage>, Option<&ArmorDefense>, Option<&Tags>),
        (With<Item>, Without<Enemy>, Without<Player>),
    >,
    chest_query: Query<
        (Entity, &GridPos),
        (
            With<Chest>,
            Without<Player>,
            Without<Pushable>,
            Without<Enemy>,
            Without<Item>,
        ),
    >,
    item_query: Query<
        (
            Entity,
            Option<&GridPos>,
            &ItemKind,
            Option<&Equippable>,
            Option<&Consumable>,
        ),
        (With<Item>, Without<Player>, Without<Pushable>),
    >,
    mut gold_count: ResMut<GoldCount>,
    mut game_log: ResMut<GameLog>,
) {
    let Some(action) = pending.0 else {
        next_phase.set(TurnPhase::WaitingForInput);
        return;
    };

    // --- Handle consumable use (does NOT advance turn) ---
    if let PlayerAction::UseConsumable(slot_idx) = action {
        let Ok((player_entity, _, mut inventory, mut health, mut tags)) = player_query.single_mut()
        else {
            next_phase.set(TurnPhase::WaitingForInput);
            return;
        };

        if slot_idx < inventory.consumables.len() {
            let item_entity = inventory.consumables[slot_idx];
            if let Ok((_, _, kind, _, _)) = item_query.get(item_entity) {
                match kind {
                    ItemKind::HealthPotion => {
                        health.0 = (health.0 + 3).min(10);
                        game_log.push("Used Health Potion (+3 HP)");
                    }
                    ItemKind::Antidote => {
                        tags.0.remove(&Tag::Poisoned);
                        game_log.push("Used Antidote");
                    }
                    ItemKind::FireResistPotion => {
                        commands.entity(player_entity).insert(FireResistBuff);
                        game_log.push("Used Fire Resist Potion");
                    }
                    _ => {
                        next_phase.set(TurnPhase::WaitingForInput);
                        return;
                    }
                }
                inventory.consumables.remove(slot_idx);
                commands.entity(item_entity).despawn();
            }
        }
        next_phase.set(TurnPhase::WaitingForInput);
        return;
    }

    // --- Handle movement ---
    let PlayerAction::Move(dir) = action else {
        next_phase.set(TurnPhase::WaitingForInput);
        return;
    };

    let Ok((_, mut player_pos, inventory, _, _)) = player_query.single_mut() else {
        next_phase.set(TurnPhase::EnemyResolve);
        return;
    };

    let target = player_pos.0 + dir;

    // Check if there's an enemy at the target — melee attack
    if let Some((enemy_entity, _, mut enemy_hp, mut enemy_tags, drop_table, boss)) = enemy_combat
        .iter_mut()
        .find(|(_, gp, _, _, _, _)| gp.0 == target)
    {
        let name = enemy_name(boss, enemy_tags.as_deref());
        let mut total_damage = 1;
        let mut weapon_tags_to_apply: Vec<Tag> = Vec::new();

        if let Some(weapon_entity) = inventory.weapon {
            if let Ok((wpn_dmg, _, wpn_tags)) = item_stats_query.get(weapon_entity) {
                if let Some(wpn_dmg) = wpn_dmg {
                    total_damage += wpn_dmg.0;
                }
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

        if let Some(ref mut enemy_tags) = enemy_tags {
            for tag in &weapon_tags_to_apply {
                enemy_tags.0.insert(*tag);
            }
        }

        if enemy_hp.0 <= 0 {
            game_log.push(format!("Killed {}!", name));
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
        } else {
            game_log.push(format!("Hit {} for {} damage", name, total_damage));
        }

        next_phase.set(TurnPhase::EnemyResolve);
        return;
    }

    // Check if there's a chest at the target — open it
    if let Some((chest_entity, _)) = chest_query.iter().find(|(_, gp)| gp.0 == target) {
        game_log.push("Opened a chest!");
        let chest_pos = target;
        let loot_count = 1 + (entity_hash(chest_entity, 0) % 2) as usize;
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

        next_phase.set(TurnPhase::EnemyResolve);
        return;
    }

    // Check if there's a pushable entity at the target
    let pushable_at_target = pushable_query
        .iter()
        .find(|(_, pos, _, _, _)| pos.0 == target)
        .map(|(e, _, _, _, _)| e);

    if let Some(push_entity) = pushable_at_target {
        let push_dest = target + dir;
        let dest_blocked_wall = blocking_query.iter().any(|(_, gp)| gp.0 == push_dest);
        let dest_blocked_enemy = enemy_combat
            .iter()
            .any(|(_, gp, _, _, _, _)| gp.0 == push_dest);
        let dest_blocked_pushable = pushable_query
            .iter()
            .any(|(e, pos, _, _, _)| e != push_entity && pos.0 == push_dest);
        let dest_occupied = all_pos_query.iter().any(|gp| gp.0 == push_dest);
        let dest_blocked_chest = chest_query.iter().any(|(_, gp)| gp.0 == push_dest);

        if !dest_blocked_wall
            && !dest_blocked_enemy
            && !dest_blocked_pushable
            && !dest_occupied
            && !dest_blocked_chest
        {
            // Check if pushed entity is on fire before moving it
            let push_on_fire = pushable_query
                .get(push_entity)
                .map(|(_, _, _, tags, derived)| {
                    tags.is_some_and(|t| t.0.contains(&Tag::OnFire))
                        || derived.is_some_and(|d| d.0.contains(&Tag::OnFire))
                })
                .unwrap_or(false);

            if let Ok((_, mut push_pos, _, _, _)) = pushable_query.get_mut(push_entity) {
                push_pos.0 = push_dest;
            }
            player_pos.0 = target;

            // Pushing a burning entity deals 1 damage (reduced by armor)
            if push_on_fire {
                let Ok((_, _, ref inventory, ref mut health, _)) = player_query.single_mut()
                else {
                    next_phase.set(TurnPhase::EnemyResolve);
                    return;
                };
                let armor_def = inventory
                    .armor
                    .and_then(|e| item_stats_query.get(e).ok())
                    .and_then(|(_, armor, _)| armor)
                    .map(|a| a.0)
                    .unwrap_or(0);
                let dmg = (1 - armor_def).max(0);
                if dmg > 0 {
                    health.0 -= dmg;
                    game_log.push("Burned while pushing!");
                }
            }
        } else {
            let pushable_is_blocking = pushable_query
                .get(push_entity)
                .map(|(_, _, b, _, _)| b.is_some())
                .unwrap_or(false);
            if !pushable_is_blocking {
                player_pos.0 = target;
            } else {
                flash_entity(&mut commands, push_entity);
                next_phase.set(TurnPhase::WaitingForInput);
                return;
            }
        }
    } else {
        let blocked_wall = blocking_query.iter().find(|(_, gp)| gp.0 == target);
        let blocked_pushable = pushable_query
            .iter()
            .find(|(_, pos, b, _, _)| pos.0 == target && b.is_some());
        if blocked_wall.is_none() && blocked_pushable.is_none() {
            player_pos.0 = target;
        } else {
            if let Some((entity, _)) = blocked_wall {
                flash_entity(&mut commands, entity);
            }
            if let Some((entity, _, _, _, _)) = blocked_pushable {
                flash_entity(&mut commands, entity);
            }
            next_phase.set(TurnPhase::WaitingForInput);
            return;
        }
    }

    // Player moved to target — pick up items
    pickup_items_at(
        &mut commands,
        &mut player_query,
        &item_query,
        &mut gold_count,
        &mut game_log,
    );

    // Check for stairs
    let on_stairs_down = stairs_down.iter().any(|gp| gp.0 == target);
    let on_stairs_up = stairs_up.iter().any(|gp| gp.0 == target);

    if on_stairs_down {
        floor_transition.0 = Some(true);
        // Stairs don't advance the turn
        next_phase.set(TurnPhase::WaitingForInput);
    } else if on_stairs_up {
        floor_transition.0 = Some(false);
        next_phase.set(TurnPhase::WaitingForInput);
    } else {
        next_phase.set(TurnPhase::EnemyResolve);
    }
}

/// Pick up items at the player's current position.
fn pickup_items_at(
    commands: &mut Commands,
    player_query: &mut Query<
        (Entity, &mut GridPos, &mut Inventory, &mut Health, &mut Tags),
        With<Player>,
    >,
    item_query: &Query<
        (
            Entity,
            Option<&GridPos>,
            &ItemKind,
            Option<&Equippable>,
            Option<&Consumable>,
        ),
        (With<Item>, Without<Player>, Without<Pushable>),
    >,
    gold_count: &mut ResMut<GoldCount>,
    game_log: &mut ResMut<GameLog>,
) {
    let Ok((_, player_pos, mut inventory, _, _)) = player_query.single_mut() else {
        return;
    };

    let items_here: Vec<_> = item_query
        .iter()
        .filter(|(_, gp, _, _, _)| gp.is_some_and(|gp| gp.0 == player_pos.0))
        .map(|(e, _, kind, equip, consumable)| {
            (e, *kind, equip.map(|eq| eq.0), consumable.is_some())
        })
        .collect();

    for (entity, kind, equip_slot, is_consumable) in items_here {
        if kind == ItemKind::Gold {
            gold_count.0 += 1;
            game_log.push("Picked up Gold");
            commands.entity(entity).despawn();
            continue;
        }

        if let Some(slot) = equip_slot {
            game_log.push(format!("Equipped {}", item_name(&kind)));
            match slot {
                EquipSlot::Weapon => {
                    if let Some(old_weapon) = inventory.weapon.take() {
                        commands
                            .entity(old_weapon)
                            .insert((GridPos(player_pos.0), FloorEntity));
                    }
                    inventory.weapon = Some(entity);
                    commands.entity(entity).remove::<GridPos>();
                    commands.entity(entity).remove::<FloorEntity>();
                }
                EquipSlot::Armor => {
                    if let Some(old_armor) = inventory.armor.take() {
                        commands
                            .entity(old_armor)
                            .insert((GridPos(player_pos.0), FloorEntity));
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
                game_log.push(format!("Picked up {}", item_name(&kind)));
                inventory.consumables.push(entity);
                commands.entity(entity).remove::<GridPos>();
                commands.entity(entity).remove::<FloorEntity>();
            }
            continue;
        }
    }
}

// ---------------------------------------------------------------------------
// resolve_enemy_turn — enemy AI + combat
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn resolve_enemy_turn(
    mut commands: Commands,
    player_query: Query<(&GridPos, &Inventory), (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<
        (
            Entity,
            &mut GridPos,
            &mut Health,
            Option<&Boss>,
            Option<&Tags>,
        ),
        (With<Enemy>, Without<Player>),
    >,
    mut player_health: Query<&mut Health, (With<Player>, Without<Enemy>)>,
    blocking_query: Query<
        (Entity, &GridPos, Option<&Tags>),
        (With<Blocking>, Without<Enemy>, Without<Player>),
    >,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    armor_query: Query<&ArmorDefense, (With<Item>, Without<Player>, Without<Enemy>)>,
    mut game_log: ResMut<GameLog>,
    mut death_cause: ResMut<DeathCause>,
) {
    let Ok((player_pos, inventory)) = player_query.single() else {
        next_phase.set(TurnPhase::ResolveEnvironment);
        return;
    };
    let player_tile = player_pos.0;

    let armor_def = inventory
        .armor
        .and_then(|e| armor_query.get(e).ok())
        .map(|a| a.0)
        .unwrap_or(0);

    let static_blocking: Vec<IVec2> = blocking_query.iter().map(|(_, pos, _)| pos.0).collect();

    let wall_positions: Vec<IVec2> = blocking_query
        .iter()
        .filter(|(_, _, tags)| tags.as_ref().is_some_and(|t| t.0.contains(&Tag::Stone)))
        .map(|(_, pos, _)| pos.0)
        .collect();

    let enemy_positions: Vec<(Entity, IVec2)> = enemy_query
        .iter()
        .map(|(e, pos, _, _, _)| (e, pos.0))
        .collect();

    let mut moves: Vec<(Entity, IVec2)> = Vec::new();
    let mut fire_spawns: Vec<IVec2> = Vec::new();

    for (entity, pos, _, boss, etags) in enemy_query.iter() {
        let epos = pos.0;
        let diff = player_tile - epos;
        let name = enemy_name(boss, etags);

        // Boss AI
        if boss.is_some() {
            if diff.x.abs() + diff.y.abs() == 1 {
                if let Ok(mut ph) = player_health.single_mut() {
                    let dmg = (2 - armor_def).max(0);
                    ph.0 -= dmg;
                    game_log.push(format!("{} hits you for {} damage", name, dmg));
                    if ph.0 <= 0 {
                        death_cause.0 = Some(format!("Slain by {}", name));
                    }
                }
            } else {
                game_log.push("Dragon breathes fire!");
                let dir = best_cardinal_direction(diff);
                for i in 1..=3 {
                    let fire_pos = epos + dir * i;
                    if wall_positions.contains(&fire_pos) {
                        break;
                    }
                    fire_spawns.push(fire_pos);
                }
            }
            continue;
        }

        // Regular enemy: if adjacent, attack
        if diff.x.abs() + diff.y.abs() == 1 {
            if let Ok(mut ph) = player_health.single_mut() {
                let dmg = (1 - armor_def).max(0);
                ph.0 -= dmg;
                game_log.push(format!("{} hits you for {} damage", name, dmg));
                if ph.0 <= 0 {
                    death_cause.0 = Some(format!("Slain by {}", name));
                }
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
            let new_dist = (player_tile - new_pos).x.abs() + (player_tile - new_pos).y.abs();
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
    for (mut_entity, mut pos, _, _, _) in enemy_query.iter_mut() {
        if let Some((_, to)) = moves.iter().find(|(e, _)| *e == mut_entity) {
            pos.0 = *to;
        }
    }

    // Spawn fire breath entities
    for fire_pos in fire_spawns {
        commands.spawn((
            GridPos(fire_pos),
            Tags(BTreeSet::from([Tag::OnFire])),
            DerivedTags::default(),
            FloorEntity,
            DespawnOnExit(GameState::Playing),
        ));
    }

    next_phase.set(TurnPhase::ResolveEnvironment);
}
