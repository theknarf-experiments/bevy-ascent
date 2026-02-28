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
fn enemy_turn_system_params_are_valid() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_enemy(IVec2::new(8, 8));
    game.app_mut().update();
}

#[test]
fn resolve_environment_system_params_are_valid() {
    let mut game = GameHarness::custom();
    game.spawn_torch(IVec2::new(1, 1));
    game.app_mut().update();
}

#[test]
fn apply_consequences_system_params_are_valid() {
    let mut game = GameHarness::custom();
    game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(1, 1)),
        Tags(BTreeSet::from([Tag::Flesh])),
        DerivedTags(BTreeSet::from([Tag::TakingDamage])),
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
    let exits = w.query_filtered::<(), With<Exit>>().iter(w).count();

    assert_eq!(players, 1, "should have exactly 1 player");
    assert_eq!(enemies, 2, "should have exactly 2 goblins");
    assert_eq!(exits, 1, "should have exactly 1 exit");
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
    assert!(derived.contains(&Tag::Melted), "ice adjacent to fire should melt");
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
        game.derived_at(IVec2::new(1, 0))[0].contains(&Tag::TakingDamage),
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
        .find(|dt| dt.contains(&Tag::TakingDamage))
        .expect("flesh on same tile as fire should take damage");
    assert!(flesh_derived.contains(&Tag::TakingDamage));
}

// =========================================================================
// Apply consequences
// =========================================================================

#[test]
fn taking_damage_decrements_health() {
    let mut game = GameHarness::custom();
    let enemy = game.app_mut().world_mut().spawn((
        GridPos(IVec2::new(0, 0)),
        Tags(BTreeSet::from([Tag::Flesh])),
        DerivedTags(BTreeSet::from([Tag::TakingDamage])),
        Health(3),
        Enemy,
        Blocking,
    )).id();

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
        DerivedTags(BTreeSet::from([Tag::TakingDamage])),
        Health(1),
        Enemy,
        Blocking,
    ));

    game.app_mut()
        .world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::ApplyConsequences);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(game.enemy_count(), 0, "entity with 0 health should be despawned");
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
    assert_eq!(water_count, 1, "water should be spawned at ice's old position");
}

// =========================================================================
// Win/Loss conditions
// =========================================================================

#[test]
fn player_on_exit_wins() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_exit(IVec2::new(5, 5));
    game.spawn_enemy(IVec2::new(8, 8));
    game.enable_win_loss();
    game.wait_until_state(GameState::Victory);
}

#[test]
fn all_enemies_dead_wins() {
    let mut game = GameHarness::custom();
    game.spawn_player(IVec2::new(5, 5));
    game.spawn_exit(IVec2::new(10, 10));
    // No enemies spawned
    game.enable_win_loss();
    game.wait_until_state(GameState::Victory);
}

#[test]
fn player_dead_loses() {
    let mut game = GameHarness::custom();
    // No player spawned
    game.enable_win_loss();
    game.wait_until_state(GameState::GameOver);
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
        .set(TurnPhase::EnemyTurn);
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
        .set(TurnPhase::EnemyTurn);
    game.wait_until_phase(TurnPhase::WaitingForInput);

    assert_eq!(
        game.player_health(),
        Some(2),
        "enemy adjacent to player should deal 1 damage"
    );
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

    assert_eq!(game.player_pos(), Some(IVec2::new(5, 4)), "player should move up");
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
    assert_eq!(game.player_pos(), Some(IVec2::new(5, 6)), "player should have moved down");

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
