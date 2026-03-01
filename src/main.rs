mod components;
mod datalog;
mod level;
mod render;
mod systems;
mod ui;
#[cfg(test)]
mod tests;

use bevy::prelude::*;
use bevy::feathers::FeathersPlugins;

use components::*;
use datalog::resolve_environment;
use level::spawn_level;
use render::*;
use systems::*;
use ui::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Datalog Roguelike".to_string(),
                resolution: (600u32, 600u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FeathersPlugins)
        .init_state::<GameState>()
        .add_sub_state::<TurnPhase>()
        .init_resource::<HoveredCell>()
        // Global: camera persists across all states
        .add_systems(Startup, setup_camera)
        // Main Menu
        .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
        // Playing
        .add_systems(OnEnter(GameState::Playing), (spawn_level, spawn_tooltip))
        .add_systems(
            Update,
            (spawn_sprites, sync_transforms, sync_colors, tick_flash_timers)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            update_tooltip.run_if(in_state(GameState::Playing)),
        )
        // Turn phases
        .add_systems(
            Update,
            player_input.run_if(in_state(TurnPhase::WaitingForInput)),
        )
        .add_systems(
            Update,
            enemy_turn.run_if(in_state(TurnPhase::EnemyTurn)),
        )
        .add_systems(
            Update,
            resolve_environment.run_if(in_state(TurnPhase::ResolveEnvironment)),
        )
        .add_systems(
            Update,
            apply_consequences.run_if(in_state(TurnPhase::ApplyConsequences)),
        )
        // Win/loss checks (every frame while playing)
        .add_systems(
            Update,
            (check_win, check_loss).run_if(in_state(GameState::Playing)),
        )
        // Victory / GameOver screens
        .add_systems(OnEnter(GameState::Victory), spawn_victory_screen)
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
        // Global observers for hover tooltip
        .add_observer(on_hover_over)
        .add_observer(on_hover_out)
        .run();
}
