mod components;
mod datalog;
mod fov;
mod items;
mod level;
mod level_gen;
mod render;
mod systems;
mod ui;
#[cfg(test)]
mod tests;

use bevy::prelude::*;
use bevy::feathers::FeathersPlugins;

use components::*;
use datalog::resolve_environment;
use fov::update_fog_of_war;
use level::spawn_initial_floor;
use level_gen::generate_levels;
use render::*;
use systems::*;
use ui::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Datalog Roguelike".to_string(),
                resolution: (800u32, 600u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FeathersPlugins)
        .init_state::<GameState>()
        .init_state::<MenuOverlay>()
        .add_sub_state::<TurnPhase>()
        .init_resource::<HoveredCell>()
        .init_resource::<CurrentFloor>()
        .init_resource::<VictoryAchieved>()
        .init_resource::<FloorTransition>()
        .init_resource::<SettingsOrigin>()
        .init_resource::<GoldCount>()
        .init_resource::<PlayerMoved>()
        .init_resource::<FogMap>()
        .init_resource::<GameLog>()
        .init_resource::<DeathCause>()
        // Global: camera persists across all states
        .add_systems(Startup, setup_camera)
        // Main Menu
        .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
        // Menu overlays
        .add_systems(OnEnter(MenuOverlay::Paused), spawn_pause_menu)
        .add_systems(OnEnter(MenuOverlay::Settings), spawn_settings_menu)
        .add_systems(Update, handle_esc_key)
        // Playing
        .add_systems(
            OnEnter(GameState::Playing),
            (
                generate_levels,
                (spawn_initial_floor, spawn_tooltip, spawn_floor_indicator, spawn_stats_panel, reset_game_resources)
                    .after(generate_levels),
            ),
        )
        .add_systems(
            Update,
            update_fog_of_war
                .after(player_input)
                .after(handle_floor_transition)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (spawn_sprites, sync_transforms, tick_flash_timers)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            sync_colors
                .after(update_fog_of_war)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (update_tooltip, show_victory_banner, update_floor_indicator, update_stats_panel)
                .run_if(in_state(GameState::Playing)),
        )
        // Turn phases (frozen while menu overlay is active)
        .add_systems(
            Update,
            (
                player_input,
                pickup_items.after(player_input),
                use_consumable,
            )
                .run_if(in_state(TurnPhase::WaitingForInput))
                .run_if(in_state(MenuOverlay::None)),
        )
        .add_systems(
            Update,
            handle_floor_transition
                .after(player_input)
                .run_if(in_state(GameState::Playing))
                .run_if(in_state(MenuOverlay::None)),
        )
        .add_systems(
            Update,
            enemy_turn
                .run_if(in_state(TurnPhase::EnemyTurn))
                .run_if(in_state(MenuOverlay::None)),
        )
        .add_systems(
            Update,
            resolve_environment
                .run_if(in_state(TurnPhase::ResolveEnvironment))
                .run_if(in_state(MenuOverlay::None)),
        )
        .add_systems(
            Update,
            apply_consequences
                .run_if(in_state(TurnPhase::ApplyConsequences))
                .run_if(in_state(MenuOverlay::None)),
        )
        // Win/loss checks (every frame while playing, frozen while paused)
        .add_systems(
            Update,
            (check_win, check_loss)
                .run_if(in_state(GameState::Playing))
                .run_if(in_state(MenuOverlay::None)),
        )
        // GameOver screen
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
        // Global observers for hover tooltip
        .add_observer(on_hover_over)
        .add_observer(on_hover_out)
        .run();
}
