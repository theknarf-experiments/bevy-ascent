mod components;
mod datalog;
mod level;
mod render;
mod systems;
#[cfg(test)]
mod tests;

use bevy::prelude::*;

use components::*;
use datalog::resolve_environment;
use level::spawn_level;
use render::*;
use systems::*;

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
        .init_state::<GameState>()
        .add_sub_state::<TurnPhase>()
        // Startup
        .add_systems(Startup, (spawn_level, setup_camera))
        // Sprite spawning (runs every frame to catch new entities)
        .add_systems(Update, (spawn_sprites, sync_transforms, sync_colors))
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
        // End screens
        .add_systems(OnEnter(GameState::Victory), show_victory)
        .add_systems(OnEnter(GameState::GameOver), show_game_over)
        .run();
}
