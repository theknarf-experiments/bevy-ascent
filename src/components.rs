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
    Victory,
    GameOver,
}

#[derive(Component)]
pub struct FlashTimer(pub Timer);

#[derive(Component)]
pub struct TooltipRoot;

#[derive(SubStates, Clone, Copy, Default, Debug, Hash, PartialEq, Eq)]
#[source(GameState = GameState::Playing)]
pub enum TurnPhase {
    #[default]
    WaitingForInput,
    EnemyTurn,
    ResolveEnvironment,
    ApplyConsequences,
}
