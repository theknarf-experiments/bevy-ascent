use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;
use crate::datalog::resolve_environment;
use crate::level::spawn_initial_floor;
use crate::level_gen::fallback_floors;
use crate::systems::*;

const MAX_FRAMES: usize = 100;

pub struct GameHarness {
    app: App,
}

impl GameHarness {
    /// Create a harness with the full level loaded (like the real game).
    pub fn new() -> Self {
        let mut app = Self::base_app();
        // Full game includes win/loss checks (level always has player + enemies)
        app.add_systems(
            Update,
            (check_win, check_loss).run_if(in_state(GameState::Playing)),
        );
        app.add_systems(OnEnter(GameState::Playing), spawn_initial_floor);

        // Initialize (MainMenu state)
        app.update();
        // Transition to Playing
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update(); // transition to Playing, OnEnter runs spawn_initial_floor
        app.update(); // flush spawn commands

        Self { app }
    }

    /// Create a harness with an empty world for isolated tests.
    /// Does NOT include check_win/check_loss — call `enable_win_loss()` after
    /// spawning entities if needed.
    pub fn custom() -> Self {
        let mut app = Self::base_app();
        // Initialize (MainMenu state)
        app.update();
        // Transition to Playing so TurnPhase sub-state exists
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update(); // transition to Playing
        Self { app }
    }

    /// Add win/loss systems. Call after spawning entities to avoid
    /// false triggers on an empty world.
    pub fn enable_win_loss(&mut self) {
        self.app.add_systems(
            Update,
            (check_win, check_loss).run_if(in_state(GameState::Playing)),
        );
    }

    fn base_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_state::<GameState>();
        app.add_sub_state::<TurnPhase>();
        app.init_resource::<CurrentFloor>();
        app.init_resource::<VictoryAchieved>();
        app.init_resource::<FloorTransition>();
        app.init_state::<MenuOverlay>();
        app.init_resource::<SettingsOrigin>();
        app.insert_resource(fallback_floors());

        // Turn-phase systems (same as main.rs, minus rendering and win/loss)
        app.add_systems(
            Update,
            player_input.run_if(in_state(TurnPhase::WaitingForInput)),
        );
        app.add_systems(
            Update,
            handle_floor_transition
                .after(player_input)
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            enemy_turn.run_if(in_state(TurnPhase::EnemyTurn)),
        );
        app.add_systems(
            Update,
            resolve_environment.run_if(in_state(TurnPhase::ResolveEnvironment)),
        );
        app.add_systems(
            Update,
            apply_consequences.run_if(in_state(TurnPhase::ApplyConsequences)),
        );

        app
    }

    // ---- Input ----

    pub fn press_key(&mut self, key: KeyCode) {
        self.app
            .world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        self.app.update();
        // reset_all clears pressed + just_pressed + just_released,
        // so the next press() will register as just_pressed again.
        self.app
            .world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
    }

    // ---- Waiting ----

    pub fn wait_until(&mut self, condition: impl Fn(&mut App) -> bool, max_frames: usize) {
        for _ in 0..max_frames {
            self.app.update();
            if condition(&mut self.app) {
                return;
            }
        }
        panic!(
            "wait_until: condition not met after {} frames",
            max_frames
        );
    }

    pub fn wait_until_phase(&mut self, phase: TurnPhase) {
        self.wait_until(
            move |app| {
                *app.world_mut()
                    .resource::<State<TurnPhase>>()
                    .get()
                    == phase
            },
            MAX_FRAMES,
        );
    }

    pub fn wait_until_state(&mut self, state: GameState) {
        self.wait_until(
            move |app| {
                *app.world_mut()
                    .resource::<State<GameState>>()
                    .get()
                    == state
            },
            MAX_FRAMES,
        );
    }

    /// Run one resolve+consequences cycle (sets phase to ResolveEnvironment and waits for return).
    pub fn resolve(&mut self) {
        self.app
            .world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::ResolveEnvironment);
        self.wait_until_phase(TurnPhase::WaitingForInput);
    }

    /// Run only the resolve_environment phase (one update). DerivedTags are
    /// written but apply_consequences has NOT run yet. Use this when you need
    /// to inspect derived tags before consequences modify or despawn entities.
    pub fn resolve_only(&mut self) {
        self.app
            .world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::ResolveEnvironment);
        self.app.update();
    }

    // ---- Query helpers ----

    pub fn player_pos(&mut self) -> Option<IVec2> {
        let w = self.app.world_mut();
        let mut q = w.query_filtered::<&GridPos, With<Player>>();
        q.iter(w).next().map(|gp| gp.0)
    }

    pub fn entity_pos(&mut self, entity: Entity) -> Option<IVec2> {
        self.app
            .world_mut()
            .get::<GridPos>(entity)
            .map(|gp| gp.0)
    }

    pub fn player_health(&mut self) -> Option<i32> {
        let w = self.app.world_mut();
        let mut q = w.query_filtered::<&Health, With<Player>>();
        q.iter(w).next().map(|h| h.0)
    }

    pub fn entity_health(&mut self, entity: Entity) -> Option<i32> {
        self.app
            .world_mut()
            .get::<Health>(entity)
            .map(|h| h.0)
    }

    pub fn enemy_count(&mut self) -> usize {
        let w = self.app.world_mut();
        w.query_filtered::<(), With<Enemy>>().iter(w).count()
    }

    pub fn turn_phase(&mut self) -> TurnPhase {
        *self
            .app
            .world_mut()
            .resource::<State<TurnPhase>>()
            .get()
    }

    pub fn current_floor(&mut self) -> u32 {
        self.app.world_mut().resource::<CurrentFloor>().0
    }

    pub fn victory_achieved(&mut self) -> bool {
        self.app.world_mut().resource::<VictoryAchieved>().0
    }

    pub fn game_state(&mut self) -> GameState {
        *self.app.world_mut().resource::<State<GameState>>().get()
    }

    pub fn tags_at(&mut self, pos: IVec2) -> Vec<BTreeSet<Tag>> {
        let w = self.app.world_mut();
        let mut q = w.query::<(&GridPos, &Tags)>();
        q.iter(w)
            .filter(|(gp, _)| gp.0 == pos)
            .map(|(_, t)| t.0.clone())
            .collect()
    }

    pub fn derived_at(&mut self, pos: IVec2) -> Vec<BTreeSet<Tag>> {
        let w = self.app.world_mut();
        let mut q = w.query::<(&GridPos, &DerivedTags)>();
        q.iter(w)
            .filter(|(gp, _)| gp.0 == pos)
            .map(|(_, dt)| dt.0.clone())
            .collect()
    }

    // ---- Spawn helpers (for custom harness) ----

    pub fn spawn_player(&mut self, pos: IVec2) -> Entity {
        self.app
            .world_mut()
            .spawn((
                GridPos(pos),
                Tags(BTreeSet::from([Tag::Flesh])),
                DerivedTags::default(),
                Player,
                Health(5),
            ))
            .id()
    }

    pub fn spawn_enemy(&mut self, pos: IVec2) -> Entity {
        self.app
            .world_mut()
            .spawn((
                GridPos(pos),
                Tags(BTreeSet::from([Tag::Flesh])),
                DerivedTags::default(),
                Enemy,
                Health(2),
                Blocking,
            ))
            .id()
    }

    pub fn spawn_wall(&mut self, pos: IVec2) -> Entity {
        self.app
            .world_mut()
            .spawn((
                GridPos(pos),
                Tags(BTreeSet::from([Tag::Stone])),
                Blocking,
            ))
            .id()
    }

    pub fn spawn_torch(&mut self, pos: IVec2) -> Entity {
        self.app
            .world_mut()
            .spawn((
                GridPos(pos),
                Tags(BTreeSet::from([Tag::Wood, Tag::OnFire])),
                DerivedTags::default(),
                Pushable,
            ))
            .id()
    }

    pub fn spawn_barrel(&mut self, pos: IVec2) -> Entity {
        self.app
            .world_mut()
            .spawn((
                GridPos(pos),
                Tags(BTreeSet::from([Tag::Wood])),
                DerivedTags::default(),
                Pushable,
                Blocking,
            ))
            .id()
    }

    pub fn spawn_oil(&mut self, pos: IVec2) -> Entity {
        self.app
            .world_mut()
            .spawn((
                GridPos(pos),
                Tags(BTreeSet::from([Tag::Oil])),
                DerivedTags::default(),
            ))
            .id()
    }

    pub fn spawn_ice(&mut self, pos: IVec2) -> Entity {
        self.app
            .world_mut()
            .spawn((
                GridPos(pos),
                Tags(BTreeSet::from([Tag::Ice])),
                DerivedTags::default(),
                Pushable,
                Blocking,
            ))
            .id()
    }

    pub fn spawn_exit(&mut self, pos: IVec2) -> Entity {
        self.app
            .world_mut()
            .spawn((GridPos(pos), Exit))
            .id()
    }

    pub fn spawn_stairs_down(&mut self, pos: IVec2) -> Entity {
        self.app
            .world_mut()
            .spawn((GridPos(pos), StairsDown, FloorEntity))
            .id()
    }

    pub fn spawn_stairs_up(&mut self, pos: IVec2) -> Entity {
        self.app
            .world_mut()
            .spawn((GridPos(pos), StairsUp, FloorEntity))
            .id()
    }

    /// Expose app for param-validation tests that need direct access.
    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }
}
