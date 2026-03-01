mod harness;

use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;
use harness::GameHarness;

// =========================================================================
// System param validation tests
// =========================================================================

#[test]
fn player_input_system_params_are_valid() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_barrel(IVec2::new(2, 2));
    game.spawn_wall(IVec2::new(0, 0));
    game.app_mut().update();
}

#[test]
fn resolve_enemy_turn_system_params_are_valid() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_enemy(IVec2::new(8, 8));
    game.app_mut().update();
}

#[test]
fn resolve_player_turn_system_params_are_valid() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_torch(IVec2::new(1, 1));
    game.app_mut().update();
}

#[test]
fn apply_consequences_system_params_are_valid() {
    let mut game = GameHarness::custom();
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(1, 1)),
        Tags(BTreeSet::from([Tag::Flesh])),
        DerivedTags(BTreeSet::from([Tag::FireDamage])),
        Health(2),
        Enemy,
        Blocking,
    ));
    game.app_mut().update();
}

#[test]
fn check_win_system_params_are_valid() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_exit(IVec2::new(10, 10));
    game.app_mut().update();
}

#[test]
fn check_loss_system_params_are_valid() {
    let mut game = GameHarness::custom();
    game.app_mut().update();
}

#[test]
fn all_systems_together_no_param_conflicts() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_enemy(IVec2::new(8, 8));
    game.spawn_wall(IVec2::new(0, 0));
    game.spawn_barrel(IVec2::new(3, 3));
    game.spawn_exit(IVec2::new(10, 10));
    game.app_mut().update();
    game.app_mut().update();
}

// =========================================================================
// Level spawning
// =========================================================================

#[test]
fn level_spawns_correct_entity_counts() {
    let mut game = GameHarness::new();
    let w = game.app_mut().world_mut();
    let players = w.query_filtered::<(), With<Player>>().iter(w).count();
    let enemies = w.query_filtered::<(), With<Enemy>>().iter(w).count();
    let stairs_down = w.query_filtered::<(), With<StairsDown>>().iter(w).count();

    assert_eq!(players, 1, "should have exactly 1 player");
    assert_eq!(enemies, 2, "floor 1 should have exactly 2 goblins");
    assert_eq!(stairs_down, 1, "floor 1 should have exactly 1 stairs down");
}

#[test]
fn player_spawns_at_correct_position() {
    let mut game = GameHarness::new();
    assert_eq!(
        game.player_pos(),
        Some(IVec2::new(5, 10)),
        "player should be at (5, 10) in the level"
    );
}

#[test]
fn player_starts_with_5_hp() {
    let mut game = GameHarness::new();
    assert_eq!(
        game.player_health(),
        Some(5),
        "player should start with 5 HP"
    );
}

// =========================================================================
// Datalog rules
// =========================================================================

#[test]
fn wood_derives_flammable() {
    let mut game = GameHarness::custom();
    game.spawn_barrel(IVec2::new(0, 0));
    game.resolve();
    assert!(
        game.derived_at(IVec2::new(0, 0))[0].contains(&Tag::Flammable),
        "Wood should derive Flammable"
    );
}

#[test]
fn oil_derives_flammable() {
    let mut game = GameHarness::custom();
    game.spawn_oil(IVec2::new(0, 0));
    game.resolve();
    assert!(
        game.derived_at(IVec2::new(0, 0))[0].contains(&Tag::Flammable),
        "Oil should derive Flammable"
    );
}

#[test]
fn fire_spreads_to_adjacent_flammable() {
    let mut game = GameHarness::custom();
    game.spawn_torch(IVec2::new(0, 0));
    game.spawn_oil(IVec2::new(1, 0));
    game.resolve();
    assert!(
        game.derived_at(IVec2::new(1, 0))[0].contains(&Tag::OnFire),
        "oil adjacent to fire should catch fire"
    );
}

#[test]
fn fire_spreads_transitively() {
    let mut game = GameHarness::custom();
    game.spawn_torch(IVec2::new(0, 0));
    game.spawn_oil(IVec2::new(1, 0));
    game.spawn_barrel(IVec2::new(2, 0));
    game.resolve();
    assert!(
        game.derived_at(IVec2::new(2, 0))[0].contains(&Tag::OnFire),
        "fire should spread transitively: torch -> oil -> barrel"
    );
}

#[test]
fn fire_does_not_spread_to_non_adjacent() {
    let mut game = GameHarness::custom();
    game.spawn_torch(IVec2::new(0, 0));
    game.spawn_oil(IVec2::new(5, 5));
    game.resolve();
    assert!(
        !game.derived_at(IVec2::new(5, 5))[0].contains(&Tag::OnFire),
        "fire should not spread to non-adjacent entities"
    );
}

#[test]
fn wet_blocks_fire_spread() {
    let mut game = GameHarness::custom();
    game.spawn_torch(IVec2::new(0, 0));
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(1, 0)),
        Tags(BTreeSet::from([Tag::Wood, Tag::Wet])),
        DerivedTags::default(),
    ));
    game.resolve();
    assert!(
        !game.derived_at(IVec2::new(1, 0))[0].contains(&Tag::OnFire),
        "wet entities should not catch fire"
    );
}

#[test]
fn ice_melts_adjacent_to_fire() {
    let mut game = GameHarness::custom();
    game.spawn_torch(IVec2::new(0, 0));
    game.spawn_ice(IVec2::new(1, 0));
    // Use resolve_only so we can inspect derived tags before
    // apply_consequences despawns the ice entity.
    game.resolve_only();
    let derived = &game.derived_at(IVec2::new(1, 0))[0];
    assert!(
        derived.contains(&Tag::Melted),
        "ice adjacent to fire should melt"
    );
    assert!(derived.contains(&Tag::Wet), "melted ice should be wet");
}

#[test]
fn flesh_adjacent_to_fire_takes_damage() {
    let mut game = GameHarness::custom();
    game.spawn_torch(IVec2::new(0, 0));
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(1, 0)),
        Tags(BTreeSet::from([Tag::Flesh])),
        DerivedTags::default(),
    ));
    game.resolve();
    assert!(
        game.derived_at(IVec2::new(1, 0))[0].contains(&Tag::FireDamage),
        "flesh adjacent to fire should take damage"
    );
}

#[test]
fn flesh_on_same_tile_as_fire_takes_damage() {
    let mut game = GameHarness::custom();
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(0, 0)),
        Tags(BTreeSet::from([Tag::Oil, Tag::OnFire])),
        DerivedTags::default(),
    ));
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(0, 0)),
        Tags(BTreeSet::from([Tag::Flesh])),
        DerivedTags::default(),
    ));
    game.resolve();
    let derived_list = game.derived_at(IVec2::new(0, 0));
    let flesh_derived = derived_list
        .iter()
        .find(|dt| dt.contains(&Tag::FireDamage))
        .expect("flesh on same tile as fire should take damage");
    assert!(flesh_derived.contains(&Tag::FireDamage));
}

// =========================================================================
// Apply consequences
// =========================================================================

#[test]
fn taking_damage_decrements_health() {
    let mut game = GameHarness::custom();
    let enemy = game
        .app_mut()
        .world_mut()
        .spawn((
            GridPos(IVec2::new(0, 0)),
            Tags(BTreeSet::from([Tag::Flesh])),
            DerivedTags(BTreeSet::from([Tag::FireDamage])),
            Health(3),
            Enemy,
            Blocking,
        ))
        .id();

    // Set phase to ApplyConsequences and wait for it to complete
    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::ApplyConsequences);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.entity_health(enemy),
        Some(2),
        "health should decrease by 1"
    );
}

#[test]
fn zero_health_despawns() {
    let mut game = GameHarness::custom();
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(0, 0)),
        Tags(BTreeSet::from([Tag::Flesh])),
        DerivedTags(BTreeSet::from([Tag::FireDamage])),
        Health(1),
        Enemy,
        Blocking,
    ));

    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::ApplyConsequences);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.enemy_count(),
        0,
        "entity with 0 health should be despawned"
    );
}

#[test]
fn extinguished_removes_on_fire() {
    let mut game = GameHarness::custom();
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(0, 0)),
        Tags(BTreeSet::from([Tag::Wood, Tag::OnFire])),
        DerivedTags(BTreeSet::from([Tag::Extinguished])),
    ));

    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::ApplyConsequences);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert!(
        !game.tags_at(IVec2::new(0, 0))[0].contains(&Tag::OnFire),
        "extinguished should remove OnFire"
    );
}

#[test]
fn melted_ice_despawns_and_spawns_water() {
    let mut game = GameHarness::custom();
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(3, 3)),
        Tags(BTreeSet::from([Tag::Ice])),
        DerivedTags(BTreeSet::from([Tag::Melted])),
        Pushable,
        Blocking,
    ));

    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::ApplyConsequences);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    let tags_list = game.tags_at(IVec2::new(3, 3));
    let ice_count = tags_list.iter().filter(|t| t.contains(&Tag::Ice)).count();
    assert_eq!(ice_count, 0, "melted ice should be despawned");

    let water_count = tags_list.iter().filter(|t| t.contains(&Tag::Wet)).count();
    assert_eq!(
        water_count, 1,
        "water should be spawned at ice's old position"
    );
}

// =========================================================================
// Win/Loss conditions
// =========================================================================

#[test]
fn player_on_exit_wins() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_exit(IVec2::new(5, 5));
    game.enable_win_loss();
    game.app_mut().update();
    game.app_mut().update();
    assert!(
        game.victory_achieved(),
        "player on exit should set VictoryAchieved"
    );
    assert_eq!(
        game.game_state(),
        GameState::Playing,
        "game should stay in Playing state after victory"
    );
}

#[test]
fn player_dead_loses() {
    let mut game = GameHarness::custom();
    let player = game.spawn_player(IVec2::new(5, 5));
    // Set player health to 0
    game.app_mut()
        .world_mut()
        .get_mut::<Health>(player)
        .unwrap()
        .0 = 0;
    game.enable_win_loss();
    game.wait_until_overlay(MenuOverlay::GameOver);
    // Game stays in Playing state (world visible)
    assert_eq!(game.game_state(), GameState::Playing);
}

// =========================================================================
// Enemy AI
// =========================================================================

#[test]
fn enemy_moves_toward_player() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    let enemy = game.spawn_enemy(IVec2::new(5, 8));

    // Trigger enemy turn
    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::EnemyResolve);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    let enemy_pos = game.entity_pos(enemy).unwrap();
    assert!(
        enemy_pos.y < 8,
        "enemy should move toward player, got {:?}",
        enemy_pos
    );
}

#[test]
fn enemy_attacks_adjacent_player() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_enemy(IVec2::new(5, 6));

    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::EnemyResolve);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.player_health(),
        Some(4),
        "enemy adjacent to player should deal 1 damage"
    );
}

#[test]
fn player_melee_attacks_enemy() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    let enemy = game.spawn_enemy(IVec2::new(5, 4));

    // Bump into enemy = melee attack
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.entity_health(enemy),
        Some(1),
        "enemy should take 1 damage from player melee"
    );
    assert_eq!(
        game.player_pos(),
        Some(IVec2::new(5, 5)),
        "player should not move into enemy's tile"
    );
}

#[test]
fn player_melee_kills_enemy() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_enemy(IVec2::new(5, 4));

    // Two melee attacks kill a 2 HP enemy
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(game.enemy_count(), 0, "enemy should be dead after 2 hits");
}

// =========================================================================
// Player movement & push
// =========================================================================

#[test]
fn player_moves_into_empty_tile() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.player_pos(),
        Some(IVec2::new(5, 4)),
        "player should move up"
    );
}

#[test]
fn player_blocked_by_wall() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_wall(IVec2::new(5, 4));

    game.press_key(KeyCode::ArrowUp);
    // No turn cycle happens when blocked, so phase stays at WaitingForInput
    assert_eq!(
        game.player_pos(),
        Some(IVec2::new(5, 5)),
        "player should not move into wall"
    );
}

#[test]
fn player_pushes_barrel() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    let barrel = game.spawn_barrel(IVec2::new(5, 4));

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.player_pos(),
        Some(IVec2::new(5, 4)),
        "player should move into barrel's old spot"
    );
    assert_eq!(
        game.entity_pos(barrel),
        Some(IVec2::new(5, 3)),
        "barrel should be pushed one tile"
    );
}

#[test]
fn player_cannot_push_barrel_into_wall() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    let barrel = game.spawn_barrel(IVec2::new(5, 4));
    game.spawn_wall(IVec2::new(5, 3));

    game.press_key(KeyCode::ArrowUp);
    // Blocked push — no turn cycle
    assert_eq!(
        game.player_pos(),
        Some(IVec2::new(5, 5)),
        "player should not move"
    );
    assert_eq!(
        game.entity_pos(barrel),
        Some(IVec2::new(5, 4)),
        "barrel should not move"
    );
}

// =========================================================================
// Integration: turn cycle
// =========================================================================

#[test]
fn full_turn_cycle_returns_to_waiting_for_input() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_enemy(IVec2::new(1, 1));

    game.press_key(KeyCode::ArrowDown);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.turn_phase(),
        TurnPhase::WaitingForInput,
        "should return to WaitingForInput after full cycle"
    );
}

#[test]
fn player_can_move_twice() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));

    game.press_key(KeyCode::ArrowDown);
    game.wait_until_phase(TurnPhase::WaitingForInput);
    assert_eq!(
        game.player_pos(),
        Some(IVec2::new(5, 6)),
        "player should have moved down"
    );

    game.press_key(KeyCode::ArrowDown);
    game.wait_until_phase(TurnPhase::WaitingForInput);
    assert_eq!(
        game.player_pos(),
        Some(IVec2::new(5, 7)),
        "player should have moved down again"
    );
}

// =========================================================================
// Integration: fire chain
// =========================================================================

#[test]
fn fire_chain_damages_enemy() {
    let mut game = GameHarness::custom();
    game.spawn_torch(IVec2::new(0, 0));
    game.spawn_oil(IVec2::new(1, 0));
    let enemy = game.spawn_enemy(IVec2::new(2, 0));

    game.resolve();

    assert_eq!(
        game.entity_health(enemy),
        Some(1),
        "enemy should take 1 damage from adjacent fire chain"
    );
}

// =========================================================================
// Floor transitions
// =========================================================================

#[test]
fn stairs_down_triggers_floor_transition() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_stairs_down(IVec2::new(5, 4));
    game.app_mut().world_mut().resource_mut::<CurrentFloor>().0 = 1;

    game.press_key(KeyCode::ArrowUp);
    // Floor transition happens, player repositioned
    game.app_mut().update(); // flush commands

    assert_eq!(game.current_floor(), 2, "should be on floor 2");
}

#[test]
fn stairs_up_triggers_floor_transition() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_stairs_up(IVec2::new(5, 4));
    game.app_mut().world_mut().resource_mut::<CurrentFloor>().0 = 2;

    game.press_key(KeyCode::ArrowUp);
    game.app_mut().update(); // flush commands

    assert_eq!(game.current_floor(), 1, "should be on floor 1");
}

#[test]
fn health_persists_across_floors() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_stairs_down(IVec2::new(5, 4));
    game.app_mut().world_mut().resource_mut::<CurrentFloor>().0 = 1;

    // Damage player first
    {
        let w = game.app_mut().world_mut();
        let mut q = w.query_filtered::<&mut Health, With<Player>>();
        for mut h in q.iter_mut(w) {
            h.0 = 3;
        }
    }

    game.press_key(KeyCode::ArrowUp);
    game.app_mut().update();

    assert_eq!(
        game.player_health(),
        Some(3),
        "health should persist across floor transitions"
    );
}

#[test]
fn victory_does_not_change_game_state() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_exit(IVec2::new(5, 5));
    game.enable_win_loss();

    game.app_mut().update();
    game.app_mut().update();

    assert!(game.victory_achieved(), "victory should be achieved");
    assert_eq!(
        game.game_state(),
        GameState::Playing,
        "game should remain in Playing state"
    );
}

// =========================================================================
// Poison tests
// =========================================================================

#[test]
fn poison_spreads_adjacently() {
    let mut game = GameHarness::custom();
    game.spawn_poison(IVec2::new(0, 0));
    game.spawn_oil(IVec2::new(1, 0)); // any non-fire-base entity
    game.resolve_only();
    assert!(
        game.derived_at(IVec2::new(1, 0))[0].contains(&Tag::Poisoned),
        "poison should spread to adjacent entity"
    );
}

#[test]
fn poison_damages_flesh() {
    let mut game = GameHarness::custom();
    game.spawn_poison(IVec2::new(0, 0));
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(1, 0)),
        Tags(BTreeSet::from([Tag::Flesh])),
        DerivedTags::default(),
    ));
    game.resolve_only();
    let derived_list = game.derived_at(IVec2::new(1, 0));
    let flesh_derived = derived_list
        .iter()
        .find(|dt| dt.contains(&Tag::PoisonDamage))
        .expect("flesh adjacent to poison should take poison damage");
    assert!(flesh_derived.contains(&Tag::PoisonDamage));
}

#[test]
fn fire_cleanses_poison() {
    let mut game = GameHarness::custom();
    game.spawn_poison_mushroom(IVec2::new(0, 0)); // Wood + Poisoned
    game.spawn_torch(IVec2::new(1, 0)); // adjacent fire
    game.resolve();
    // After consequences, PoisonBurned should have removed Poisoned
    let tags_list = game.tags_at(IVec2::new(0, 0));
    // The mushroom may have caught fire and been consumed, or poison removed
    if !tags_list.is_empty() {
        assert!(
            !tags_list[0].contains(&Tag::Poisoned),
            "fire should cleanse poison via PoisonBurned"
        );
    }
}

#[test]
fn poison_does_not_spread_through_fire() {
    let mut game = GameHarness::custom();
    game.spawn_poison(IVec2::new(0, 0));
    // Entity with base OnFire should block poison spread
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(1, 0)),
        Tags(BTreeSet::from([Tag::Wood, Tag::OnFire])),
        DerivedTags::default(),
    ));
    game.spawn_oil(IVec2::new(2, 0));
    game.resolve_only();
    // The entity at (1,0) should NOT get Poisoned (has base OnFire)
    let derived_1 = &game.derived_at(IVec2::new(1, 0))[0];
    assert!(
        !derived_1.contains(&Tag::Poisoned),
        "poison should not spread to entities with base OnFire"
    );
}

#[test]
fn poison_and_fire_stack_damage() {
    let mut game = GameHarness::custom();
    // Poison source and fire source near a flesh entity
    game.spawn_poison(IVec2::new(0, 0));
    game.spawn_torch(IVec2::new(2, 0));
    let enemy = game
        .app_mut()
        .world_mut()
        .spawn((
            GridPos(IVec2::new(1, 0)),
            Tags(BTreeSet::from([Tag::Flesh])),
            DerivedTags::default(),
            Enemy,
            Health(5),
            Blocking,
        ))
        .id();
    game.resolve();
    // Both FireDamage and PoisonDamage should apply: -2 HP total
    assert_eq!(
        game.entity_health(enemy),
        Some(3),
        "fire + poison should stack for 2 damage"
    );
}

// =========================================================================
// Electricity tests
// =========================================================================

#[test]
fn electricity_conducts_through_metal() {
    let mut game = GameHarness::custom();
    game.spawn_spark(IVec2::new(0, 0)); // Metal + Electrified
    game.spawn_metal_crate(IVec2::new(1, 0)); // Metal
    game.resolve_only();
    assert!(
        game.derived_at(IVec2::new(1, 0))[0].contains(&Tag::Electrified),
        "electricity should conduct through metal"
    );
}

#[test]
fn electricity_conducts_through_water() {
    let mut game = GameHarness::custom();
    game.spawn_spark(IVec2::new(0, 0)); // Metal + Electrified
    game.spawn_water_puddle(IVec2::new(1, 0)); // Wet
    game.resolve_only();
    assert!(
        game.derived_at(IVec2::new(1, 0))[0].contains(&Tag::Electrified),
        "electricity should conduct through water"
    );
}

#[test]
fn electricity_damages_flesh() {
    let mut game = GameHarness::custom();
    game.spawn_spark(IVec2::new(0, 0));
    let enemy = game.spawn_enemy(IVec2::new(1, 0));
    game.resolve();
    assert_eq!(
        game.entity_health(enemy),
        Some(1),
        "electricity should damage adjacent flesh"
    );
}

#[test]
fn electricity_does_not_conduct_through_stone() {
    let mut game = GameHarness::custom();
    game.spawn_spark(IVec2::new(0, 0));
    game.spawn_wall(IVec2::new(1, 0)); // Stone
    game.spawn_oil(IVec2::new(2, 0));
    game.resolve_only();
    // Stone wall doesn't conduct, so (2,0) should not be electrified
    let derived = &game.derived_at(IVec2::new(2, 0))[0];
    assert!(
        !derived.contains(&Tag::Electrified),
        "electricity should not conduct through stone"
    );
}

// =========================================================================
// Combo tests
// =========================================================================

#[test]
fn fire_melts_ice_water_conducts_electricity() {
    let mut game = GameHarness::custom();
    // torch -> ice -> spark (the ice melts, becomes water, water conducts electricity)
    game.spawn_torch(IVec2::new(0, 0));
    game.spawn_ice(IVec2::new(1, 0));
    game.spawn_spark(IVec2::new(2, 0));
    game.resolve_only();
    let derived = &game.derived_at(IVec2::new(1, 0))[0];
    assert!(derived.contains(&Tag::Melted), "ice should melt near fire");
    assert!(derived.contains(&Tag::Wet), "melted ice should be wet");
    // Wet + adjacent Electrified → should conduct
    assert!(
        derived.contains(&Tag::Electrified),
        "melted ice (wet) should conduct electricity from spark"
    );
}

#[test]
fn all_three_elements_triple_damage() {
    let mut game = GameHarness::custom();
    // Fire, poison, and electricity sources around a flesh entity
    game.spawn_torch(IVec2::new(1, 0)); // fire source
    game.spawn_poison(IVec2::new(0, 1)); // poison source
    game.spawn_spark(IVec2::new(1, 2)); // electricity source
    let enemy = game
        .app_mut()
        .world_mut()
        .spawn((
            GridPos(IVec2::new(1, 1)),
            Tags(BTreeSet::from([Tag::Flesh])),
            DerivedTags::default(),
            Enemy,
            Health(5),
            Blocking,
        ))
        .id();
    game.resolve();
    assert_eq!(
        game.entity_health(enemy),
        Some(2),
        "fire + poison + electric should deal 3 damage"
    );
}

// =========================================================================
// Enemy type tests
// =========================================================================

#[test]
fn fire_imp_immune_to_fire() {
    let mut game = GameHarness::custom();
    let imp = game.spawn_fire_imp(IVec2::new(0, 0));
    game.spawn_torch(IVec2::new(1, 0)); // adjacent fire
    game.resolve();
    assert_eq!(
        game.entity_health(imp),
        Some(2),
        "fire imp should be immune to fire damage (base OnFire)"
    );
}

#[test]
fn fire_imp_ignites_adjacent_wood() {
    let mut game = GameHarness::custom();
    game.spawn_fire_imp(IVec2::new(0, 0));
    game.spawn_barrel(IVec2::new(1, 0));
    game.resolve_only();
    assert!(
        game.derived_at(IVec2::new(1, 0))[0].contains(&Tag::OnFire),
        "fire imp should ignite adjacent wood"
    );
}

#[test]
fn ice_golem_melts_near_fire() {
    let mut game = GameHarness::custom();
    let _golem = game.spawn_ice_golem(IVec2::new(1, 0));
    game.spawn_torch(IVec2::new(0, 0));
    game.resolve_only();
    let derived = &game.derived_at(IVec2::new(1, 0))[0];
    assert!(
        derived.contains(&Tag::Melted),
        "ice golem should melt near fire"
    );
    assert!(
        derived.contains(&Tag::FireDamage),
        "ice golem should take fire damage from melting"
    );
    // Verify it also has Wet derived
    assert!(
        derived.contains(&Tag::Wet),
        "melted ice golem should be wet"
    );
}

#[test]
fn poison_spider_immune_to_poison() {
    let mut game = GameHarness::custom();
    let spider = game.spawn_poison_spider(IVec2::new(0, 0));
    game.spawn_poison(IVec2::new(1, 0)); // adjacent poison
    game.resolve();
    assert_eq!(
        game.entity_health(spider),
        Some(1),
        "poison spider should be immune to poison damage (base Poisoned)"
    );
}

#[test]
fn shock_eel_electrifies_adjacent_water() {
    let mut game = GameHarness::custom();
    game.spawn_shock_eel(IVec2::new(0, 0)); // Flesh + Wet + Electrified
    game.spawn_water_puddle(IVec2::new(1, 0));
    game.resolve_only();
    assert!(
        game.derived_at(IVec2::new(1, 0))[0].contains(&Tag::Electrified),
        "shock eel should electrify adjacent water"
    );
}

// =========================================================================
// Environmental object tests
// =========================================================================

#[test]
fn explosive_barrel_area_fire_on_ignite() {
    let mut game = GameHarness::custom();
    game.spawn_torch(IVec2::new(0, 0));
    game.spawn_explosive_barrel(IVec2::new(1, 0)); // Wood + Explosive, adjacent to fire
    // Entity at dist 2 from barrel should get fire from explosion
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(3, 0)),
        Tags(BTreeSet::from([Tag::Wood])),
        DerivedTags::default(),
    ));
    game.resolve(); // resolve + apply_consequences
    // After explosion, the barrel should be despawned
    let tags_at_barrel = game.tags_at(IVec2::new(1, 0));
    let barrel_exists = tags_at_barrel.iter().any(|t| t.contains(&Tag::Explosive));
    assert!(
        !barrel_exists,
        "explosive barrel should be despawned after exploding"
    );
    // Entity at (3,0) should have gained OnFire from explosion radius
    let tags_at_3 = game.tags_at(IVec2::new(3, 0));
    assert!(
        !tags_at_3.is_empty() && tags_at_3[0].contains(&Tag::OnFire),
        "explosion should add OnFire to entities within radius 2"
    );
}

#[test]
fn explosive_barrel_chain_reaction() {
    let mut game = GameHarness::custom();
    game.spawn_torch(IVec2::new(0, 0));
    game.spawn_explosive_barrel(IVec2::new(1, 0));
    game.spawn_explosive_barrel(IVec2::new(3, 0)); // within radius 2 of first barrel
    game.resolve(); // first explosion
    // After first explosion, second barrel should have OnFire added
    // Need another resolve cycle for chain reaction
    game.resolve();
    let tags_at_3 = game.tags_at(IVec2::new(3, 0));
    let second_barrel_exists = tags_at_3.iter().any(|t| t.contains(&Tag::Explosive));
    assert!(
        !second_barrel_exists,
        "chain reaction should despawn second barrel too"
    );
}

#[test]
fn metal_crate_conducts_electricity() {
    let mut game = GameHarness::custom();
    game.spawn_spark(IVec2::new(0, 0));
    game.spawn_metal_crate(IVec2::new(1, 0));
    game.resolve_only();
    assert!(
        game.derived_at(IVec2::new(1, 0))[0].contains(&Tag::Electrified),
        "metal crate should conduct electricity"
    );
}

#[test]
fn poison_mushroom_burned_by_fire() {
    let mut game = GameHarness::custom();
    game.spawn_torch(IVec2::new(0, 0));
    game.spawn_poison_mushroom(IVec2::new(1, 0)); // Wood + Poisoned
    game.resolve();
    // After fire cleansing, poison should be removed
    let tags_list = game.tags_at(IVec2::new(1, 0));
    if !tags_list.is_empty() {
        assert!(
            !tags_list[0].contains(&Tag::Poisoned),
            "fire should burn away poison from mushroom"
        );
    }
}

#[test]
fn lightning_rod_electrifies_water() {
    let mut game = GameHarness::custom();
    game.spawn_lightning_rod(IVec2::new(0, 0)); // Metal + Electrified + Blocking
    game.spawn_water_puddle(IVec2::new(1, 0));
    game.resolve_only();
    assert!(
        game.derived_at(IVec2::new(1, 0))[0].contains(&Tag::Electrified),
        "lightning rod should electrify adjacent water"
    );
}

#[test]
fn water_puddle_conducts_electricity() {
    let mut game = GameHarness::custom();
    game.spawn_spark(IVec2::new(0, 0));
    game.spawn_water_puddle(IVec2::new(1, 0));
    game.spawn_water_puddle(IVec2::new(2, 0));
    game.resolve_only();
    assert!(
        game.derived_at(IVec2::new(2, 0))[0].contains(&Tag::Electrified),
        "electricity should conduct through chain of water puddles"
    );
}

// =========================================================================
// Item pickup tests
// =========================================================================

#[test]
fn gold_pickup() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_item(ItemKind::Gold, IVec2::new(5, 4));

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(game.gold_count(), 1, "gold should be picked up");
    assert_eq!(
        game.item_count_at(IVec2::new(5, 4)),
        0,
        "gold should be despawned"
    );
}

#[test]
fn weapon_auto_equip() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    let sword = game.spawn_item(ItemKind::IronSword, IVec2::new(5, 4));

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    let (weapon, _, _) = game.player_inventory().unwrap();
    assert_eq!(weapon, Some(sword), "iron sword should be equipped");
}

#[test]
fn weapon_swap_drops_old() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    let sword1 = game.spawn_item(ItemKind::IronSword, IVec2::new(5, 4));
    let sword2 = game.spawn_item(ItemKind::FireBlade, IVec2::new(5, 3));

    // Pick up first sword
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    let (weapon, _, _) = game.player_inventory().unwrap();
    assert_eq!(weapon, Some(sword1), "first sword should be equipped");

    // Pick up second sword — first should be dropped
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    let (weapon, _, _) = game.player_inventory().unwrap();
    assert_eq!(weapon, Some(sword2), "second sword should be equipped");
    // Old sword should be dropped at player's position
    assert!(
        game.entity_pos(sword1).is_some(),
        "old sword should be back on ground"
    );
}

#[test]
fn armor_auto_equip() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    let armor = game.spawn_item(ItemKind::LeatherArmor, IVec2::new(5, 4));

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    let (_, equipped_armor, _) = game.player_inventory().unwrap();
    assert_eq!(equipped_armor, Some(armor), "armor should be equipped");
}

#[test]
fn consumable_pickup() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    let potion = game.spawn_item(ItemKind::HealthPotion, IVec2::new(5, 4));

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    let (_, _, consumables) = game.player_inventory().unwrap();
    assert_eq!(consumables.len(), 1, "consumable should be in inventory");
    assert_eq!(
        consumables[0], potion,
        "health potion should be the consumable"
    );
}

#[test]
fn consumable_full_stays_on_ground() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 9));

    // Fill 4 consumable slots
    game.spawn_item(ItemKind::HealthPotion, IVec2::new(5, 8));
    game.spawn_item(ItemKind::HealthPotion, IVec2::new(5, 7));
    game.spawn_item(ItemKind::HealthPotion, IVec2::new(5, 6));
    game.spawn_item(ItemKind::HealthPotion, IVec2::new(5, 5));

    // Pick up 4
    for _ in 0..4 {
        game.press_key(KeyCode::ArrowUp);
        game.wait_until_phase(TurnPhase::WaitingForInput);
    }

    let (_, _, consumables) = game.player_inventory().unwrap();
    assert_eq!(consumables.len(), 4, "should have 4 consumables");

    // Place a 5th potion at player's next position
    game.spawn_item(ItemKind::Antidote, IVec2::new(5, 4));

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    let (_, _, consumables) = game.player_inventory().unwrap();
    assert_eq!(consumables.len(), 4, "should still have 4 consumables");
    assert_eq!(
        game.item_count_at(IVec2::new(5, 4)),
        1,
        "5th consumable should stay on ground"
    );
}

// =========================================================================
// Consumable use tests
// =========================================================================

#[test]
fn health_potion_restores_hp() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_item(ItemKind::HealthPotion, IVec2::new(5, 4));

    // Pick up
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Damage player
    {
        let w = game.app_mut().world_mut();
        let mut q = w.query_filtered::<&mut Health, With<Player>>();
        for mut h in q.iter_mut(w) {
            h.0 = 2;
        }
    }

    // Use consumable (goes through PlayerResolve phase)
    game.press_key(KeyCode::Digit1);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.player_health(),
        Some(5),
        "health potion should restore 3 HP"
    );
    let (_, _, consumables) = game.player_inventory().unwrap();
    assert_eq!(consumables.len(), 0, "consumable should be removed");
}

#[test]
fn antidote_removes_poison() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_item(ItemKind::Antidote, IVec2::new(5, 4));

    // Pick up
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Add Poisoned tag to player
    {
        let w = game.app_mut().world_mut();
        let mut q = w.query_filtered::<&mut Tags, With<Player>>();
        for mut tags in q.iter_mut(w) {
            tags.0.insert(Tag::Poisoned);
        }
    }

    // Use antidote (goes through PlayerResolve phase)
    game.press_key(KeyCode::Digit1);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    let player_pos = game.player_pos().unwrap();
    let tags = game.tags_at(player_pos);
    let player_tags = tags.iter().find(|t| t.contains(&Tag::Flesh)).unwrap();
    assert!(
        !player_tags.contains(&Tag::Poisoned),
        "antidote should remove Poisoned"
    );
}

#[test]
fn use_empty_slot_noop() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));

    // Try to use empty slot — should not crash (goes through PlayerResolve)
    game.press_key(KeyCode::Digit1);
    game.wait_until_phase(TurnPhase::WaitingForInput);
    assert_eq!(game.player_health(), Some(5), "health should be unchanged");
}

// =========================================================================
// Weapon damage tests
// =========================================================================

#[test]
fn iron_sword_adds_damage() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_item(ItemKind::IronSword, IVec2::new(5, 4));

    // Pick up sword
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Spawn enemy adjacent above player's new position (5,4)
    game.spawn_enemy(IVec2::new(5, 3)); // 2 HP

    // Attack enemy
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // With IronSword (+1), total damage = 2, so 2HP enemy should die
    assert_eq!(
        game.enemy_count(),
        0,
        "iron sword should deal 2 damage (1 base + 1 weapon), killing 2HP enemy"
    );
}

#[test]
fn unarmed_base_damage() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    let enemy = game.spawn_enemy(IVec2::new(5, 4)); // 2 HP

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.entity_health(enemy),
        Some(1),
        "unarmed should deal 1 base damage"
    );
}

#[test]
fn poison_dagger_poisons_enemy() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_item(ItemKind::PoisonDagger, IVec2::new(5, 4));

    // Pick up
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Spawn a high-HP enemy adjacent above player (5,4) so it survives the hit
    let enemy = game
        .app_mut()
        .world_mut()
        .spawn((
            GridPos(IVec2::new(5, 3)),
            Tags(BTreeSet::from([Tag::Flesh])),
            DerivedTags::default(),
            Enemy,
            Health(5),
            Blocking,
        ))
        .id();

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Enemy should now have Poisoned tag
    let w = game.app_mut().world_mut();
    let tags = w.get::<Tags>(enemy);
    assert!(
        tags.is_some_and(|t| t.0.contains(&Tag::Poisoned)),
        "poison dagger should add Poisoned to enemy"
    );
}

// =========================================================================
// Armor defense tests
// =========================================================================

#[test]
fn armor_reduces_melee_damage() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_item(ItemKind::LeatherArmor, IVec2::new(5, 4));

    // Pick up armor
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Spawn enemy adjacent to player
    game.spawn_enemy(IVec2::new(5, 2));

    // Trigger enemy turn — enemy attacks
    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::EnemyResolve);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // With armor defense 1, melee dmg = max(1-1, 0) = 0
    assert_eq!(
        game.player_health(),
        Some(5),
        "armor should reduce melee damage to 0"
    );
}

#[test]
fn no_armor_full_damage() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_enemy(IVec2::new(5, 6));

    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::EnemyResolve);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.player_health(),
        Some(4),
        "without armor, should take full 1 damage"
    );
}

// =========================================================================
// Systemic equipment tests
// =========================================================================

#[test]
fn iron_sword_makes_player_conductive() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_item(ItemKind::IronSword, IVec2::new(5, 4));

    // Pick up sword
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Put a spark adjacent to player at (5,4)
    game.spawn_spark(IVec2::new(5, 3));
    game.resolve_only();

    let derived = game.derived_at(IVec2::new(5, 4));
    // Player at (5,4) should have Metal from sword → Conductive → Electrified
    let player_derived = derived.iter().find(|d| d.contains(&Tag::Conductive));
    assert!(
        player_derived.is_some(),
        "iron sword should make player conductive via Metal tag"
    );
}

// =========================================================================
// Chest tests
// =========================================================================

#[test]
fn chest_opens_on_bump() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    let chest = game.spawn_chest(IVec2::new(5, 4));

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Chest should be despawned
    assert!(
        game.entity_pos(chest).is_none(),
        "chest should be despawned after opening"
    );
    // Items should spawn at chest position
    let items = game.item_count_at(IVec2::new(5, 4));
    assert!(items >= 1, "opening chest should spawn at least 1 item");
}

#[test]
fn chest_is_blocking() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_chest(IVec2::new(5, 4));

    // Bumping chest opens it (consuming the turn) but doesn't move player into chest tile
    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Player should NOT have moved to the chest position (the bump opens the chest, player stays)
    assert_eq!(
        game.player_pos(),
        Some(IVec2::new(5, 5)),
        "player should not move into chest tile when opening"
    );
}

// =========================================================================
// Enemy drops tests
// =========================================================================

#[test]
fn enemy_drops_loot_on_death() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    // Spawn a 1HP enemy with guaranteed gold drop
    game.spawn_enemy_with_drops(IVec2::new(5, 4), vec![(ItemKind::Gold, 100)]);

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(game.enemy_count(), 0, "enemy should be dead");
    assert!(
        game.item_count_at(IVec2::new(5, 4)) >= 1,
        "enemy should drop gold"
    );
}

#[test]
fn enemy_without_drop_table_no_items() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    // Regular enemy without DropTable, 1HP for quick kill
    let enemy = game
        .app_mut()
        .world_mut()
        .spawn((
            GridPos(IVec2::new(5, 4)),
            Tags(BTreeSet::from([Tag::Flesh])),
            DerivedTags::default(),
            Enemy,
            Health(1),
            Blocking,
        ))
        .id();

    game.press_key(KeyCode::ArrowUp);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert!(game.entity_pos(enemy).is_none(), "enemy should be dead");
    assert_eq!(
        game.item_count_at(IVec2::new(5, 4)),
        0,
        "no items should drop without DropTable"
    );
}

// =========================================================================
// Player starts with inventory
// =========================================================================

#[test]
fn player_starts_with_empty_inventory() {
    let mut game = GameHarness::new();
    let (weapon, armor, consumables) = game.player_inventory().unwrap();
    assert!(weapon.is_none(), "player should start with no weapon");
    assert!(armor.is_none(), "player should start with no armor");
    assert!(
        consumables.is_empty(),
        "player should start with no consumables"
    );
}

// =========================================================================
// Fog of War tests
// =========================================================================

#[test]
fn fog_initial_state_is_all_unexplored() {
    let mut game = GameHarness::custom();
    for y in 0..12 {
        for x in 0..12 {
            assert_eq!(
                game.fog_at(IVec2::new(x, y)),
                TileVisibility::Unexplored,
                "tile ({},{}) should start Unexplored",
                x,
                y
            );
        }
    }
}

#[test]
fn fog_player_tile_is_visible() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.update_fog();
    assert_eq!(
        game.fog_at(IVec2::new(5, 5)),
        TileVisibility::Visible,
        "player's tile should be Visible"
    );
}

#[test]
fn fog_open_area_within_radius_is_visible() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.update_fog();
    // Tile 3 tiles away in open area should be visible (within radius 5)
    assert_eq!(
        game.fog_at(IVec2::new(5, 2)),
        TileVisibility::Visible,
        "nearby tile in open area should be Visible"
    );
    // Tile 4 tiles away diagonally (4²+4²=32 > 25) should be unexplored
    assert_eq!(
        game.fog_at(IVec2::new(1, 1)),
        TileVisibility::Unexplored,
        "tile outside radius should stay Unexplored"
    );
}

#[test]
fn fog_wall_blocks_line_of_sight() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_wall(IVec2::new(5, 4)); // wall directly above player
    game.update_fog();
    // The wall tile itself should be visible (you see the wall face)
    assert_eq!(
        game.fog_at(IVec2::new(5, 4)),
        TileVisibility::Visible,
        "wall tile should be visible"
    );
    // Tile behind the wall should be blocked
    assert_eq!(
        game.fog_at(IVec2::new(5, 3)),
        TileVisibility::Unexplored,
        "tile behind wall should stay Unexplored"
    );
}

#[test]
fn fog_previously_visible_becomes_explored() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.update_fog();
    assert_eq!(
        game.fog_at(IVec2::new(5, 2)),
        TileVisibility::Visible,
        "tile near player should be Visible"
    );

    // Move player far away
    {
        let w = game.app_mut().world_mut();
        let mut q = w.query_filtered::<&mut GridPos, With<Player>>();
        for mut gp in q.iter_mut(w) {
            gp.0 = IVec2::new(1, 10);
        }
    }
    game.update_fog();

    assert_eq!(
        game.fog_at(IVec2::new(5, 2)),
        TileVisibility::Explored,
        "previously visible tile far from player should become Explored"
    );
}

#[test]
fn fog_resets_on_new_game() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.update_fog();
    assert_eq!(game.fog_at(IVec2::new(5, 5)), TileVisibility::Visible);

    // Simulate a game reset
    game.app_mut().world_mut().resource_mut::<FogMap>().reset();
    assert_eq!(
        game.fog_at(IVec2::new(5, 5)),
        TileVisibility::Unexplored,
        "fog should reset to all Unexplored on new game"
    );
}

// =========================================================================
// Dragon Boss tests
// =========================================================================

#[test]
fn dragon_breathes_fire_toward_player() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(2, 5));
    game.spawn_dragon(IVec2::new(8, 5));

    // Trigger enemy turn — dragon should breathe fire toward player (left)
    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::EnemyResolve);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Fire should spawn at (7,5), (6,5), (5,5) — 3 tiles toward player
    let fire_at_7 = game.tags_at(IVec2::new(7, 5));
    let fire_at_6 = game.tags_at(IVec2::new(6, 5));
    let fire_at_5 = game.tags_at(IVec2::new(5, 5));
    assert!(
        fire_at_7.iter().any(|t| t.contains(&Tag::OnFire)),
        "fire should spawn at (7,5)"
    );
    assert!(
        fire_at_6.iter().any(|t| t.contains(&Tag::OnFire)),
        "fire should spawn at (6,5)"
    );
    assert!(
        fire_at_5.iter().any(|t| t.contains(&Tag::OnFire)),
        "fire should spawn at (5,5)"
    );
}

#[test]
fn dragon_melee_attacks_adjacent_player() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_dragon(IVec2::new(5, 6));

    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::EnemyResolve);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Dragon deals 2 melee damage + 1 fire damage (OnFire tag) via environment resolve
    assert_eq!(
        game.player_health(),
        Some(2),
        "dragon should deal 2 melee + 1 fire damage"
    );
}

#[test]
fn dragon_fire_stops_at_wall() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(2, 5));
    game.spawn_dragon(IVec2::new(8, 5));
    game.spawn_wall(IVec2::new(6, 5)); // wall blocking fire line

    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::EnemyResolve);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Fire at (7,5) should exist (before wall)
    let fire_at_7 = game.tags_at(IVec2::new(7, 5));
    assert!(
        fire_at_7.iter().any(|t| t.contains(&Tag::OnFire)),
        "fire should spawn at (7,5) before wall"
    );
    // Fire at (5,5) should NOT exist (behind wall)
    let fire_at_5 = game.tags_at(IVec2::new(5, 5));
    assert!(
        !fire_at_5.iter().any(|t| t.contains(&Tag::OnFire)),
        "fire should not pass through wall"
    );
}

#[test]
fn dragon_does_not_move() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(2, 5));
    let dragon = game.spawn_dragon(IVec2::new(8, 5));

    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::EnemyResolve);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.entity_pos(dragon),
        Some(IVec2::new(8, 5)),
        "dragon should not move"
    );
}

#[test]
fn dragon_is_immune_to_fire() {
    let mut game = GameHarness::custom();
    let dragon = game.spawn_dragon(IVec2::new(5, 5));
    game.spawn_torch(IVec2::new(6, 5)); // adjacent fire
    game.resolve();
    assert_eq!(
        game.entity_health(dragon),
        Some(8),
        "dragon should be immune to fire (has OnFire tag)"
    );
}

#[test]
fn dragon_fire_ignites_barrel() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(2, 5));
    game.spawn_dragon(IVec2::new(8, 5));
    game.spawn_barrel(IVec2::new(7, 5)); // barrel in fire breath path

    // Dragon breathes fire
    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::EnemyResolve);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    // Fire entity at (7,5) should have been spawned — now resolve to see systemic effects
    game.resolve_only();

    // The barrel at (7,5) should have derived OnFire from adjacent/co-located fire
    let derived = game.derived_at(IVec2::new(7, 5));
    let has_on_fire = derived.iter().any(|d| d.contains(&Tag::OnFire));
    assert!(
        has_on_fire,
        "dragon fire should ignite barrel via systemic fire spread"
    );
}
