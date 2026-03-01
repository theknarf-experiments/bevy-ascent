use bevy::prelude::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Tag {
    // Materials (base, assigned at spawn)
    Wood,
    Oil,
    Ice,
    Flesh,
    Stone,

    // States (base or derived)
    OnFire,
    Wet,

    // Derived only
    Flammable,
    Melted,
    TakingDamage,
    Extinguished,
}

#[derive(Component, Clone, Debug)]
pub struct Tags(pub BTreeSet<Tag>);

#[derive(Component, Clone, Debug, Default)]
pub struct DerivedTags(pub BTreeSet<Tag>);

#[derive(Component, Clone, Debug, PartialEq)]
pub struct GridPos(pub IVec2);

#[derive(Component, Clone, Debug)]
pub struct Health(pub i32);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Exit;

#[derive(Component)]
pub struct Pushable;

#[derive(Component)]
pub struct Blocking;

#[derive(States, Clone, Copy, Default, Debug, Hash, PartialEq, Eq)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    GameOver,
}

#[derive(Resource, Default)]
pub struct CurrentFloor(pub u32);

#[derive(Resource, Default)]
pub struct VictoryAchieved(pub bool);

#[derive(Component)]
pub struct FloorEntity;

#[derive(Component)]
pub struct StairsDown;

#[derive(Component)]
pub struct StairsUp;

#[derive(Component)]
pub struct VictoryBanner;

#[derive(Component)]
pub struct FloorIndicator;

#[derive(Resource, Default)]
pub struct FloorTransition(pub Option<bool>); // Some(true) = going down, Some(false) = going up

#[derive(Component)]
pub struct FlashTimer(pub Timer);

#[derive(Component)]
pub struct TooltipRoot;

#[derive(States, Clone, Copy, Default, Debug, Hash, PartialEq, Eq)]
pub enum MenuOverlay {
    #[default]
    None,
    Paused,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsFrom {
    MainMenu,
    Paused,
}

#[derive(Resource)]
pub struct SettingsOrigin(pub SettingsFrom);

impl Default for SettingsOrigin {
    fn default() -> Self {
        Self(SettingsFrom::MainMenu)
    }
}

#[derive(SubStates, Clone, Copy, Default, Debug, Hash, PartialEq, Eq)]
#[source(GameState = GameState::Playing)]
pub enum TurnPhase {
    #[default]
    WaitingForInput,
    EnemyTurn,
    ResolveEnvironment,
    ApplyConsequences,
}
