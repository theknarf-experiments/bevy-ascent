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
    Metal,

    // States (base or derived)
    OnFire,
    Wet,
    Poisoned,
    Electrified,
    Explosive,

    // Derived only
    Flammable,
    Melted,
    FireDamage,
    PoisonDamage,
    ElectricDamage,
    Extinguished,
    Conductive,
    PoisonBurned,
    Exploding,

    // Buff
    FireResist,
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

// ---- Item system ----

#[derive(Component)]
pub struct Item;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemKind {
    IronSword,
    FireBlade,
    PoisonDagger,
    LeatherArmor,
    IronArmor,
    HealthPotion,
    Antidote,
    FireResistPotion,
    Gold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipSlot {
    Weapon,
    Armor,
}

#[derive(Component)]
pub struct Equippable(pub EquipSlot);

#[derive(Component)]
pub struct Consumable;

#[derive(Component)]
pub struct WeaponDamage(pub i32);

#[derive(Component)]
pub struct ArmorDefense(pub i32);

#[derive(Component, Default)]
pub struct Inventory {
    pub weapon: Option<Entity>,
    pub armor: Option<Entity>,
    pub consumables: Vec<Entity>,
}

#[derive(Component)]
pub struct Chest;

#[derive(Component, Clone)]
pub struct DropTable(pub Vec<(ItemKind, u32)>); // (kind, chance_percent)

#[derive(Component)]
pub struct FireResistBuff;

#[derive(Resource, Default)]
pub struct GoldCount(pub u32);

// ---- Game Log & Death Cause ----

#[derive(Resource, Default)]
pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub fn push(&mut self, msg: impl Into<String>) {
        self.entries.push(msg.into());
    }

    pub fn recent(&self, n: usize) -> &[String] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[derive(Resource, Default)]
pub struct DeathCause(pub Option<String>);

// ---- UI markers ----

#[derive(Component)]
pub struct StatsWeaponText;

#[derive(Component)]
pub struct StatsArmorText;

#[derive(Component)]
pub struct StatsConsumablesText;

#[derive(Component)]
pub struct StatsGoldText;

#[derive(Component)]
pub struct StatsLogText;

// ---- Fog of War ----

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TileVisibility {
    #[default]
    Unexplored,
    Explored,
    Visible,
}

#[derive(Resource)]
pub struct FogMap {
    pub tiles: [[TileVisibility; 12]; 12],
}

impl Default for FogMap {
    fn default() -> Self {
        Self {
            tiles: [[TileVisibility::Unexplored; 12]; 12],
        }
    }
}

impl FogMap {
    /// Demote all Visible tiles to Explored (called at start of each FOV update).
    pub fn begin_update(&mut self) {
        for row in &mut self.tiles {
            for tile in row.iter_mut() {
                if *tile == TileVisibility::Visible {
                    *tile = TileVisibility::Explored;
                }
            }
        }
    }

    pub fn mark_visible(&mut self, x: i32, y: i32) {
        if x >= 0 && x < 12 && y >= 0 && y < 12 {
            self.tiles[y as usize][x as usize] = TileVisibility::Visible;
        }
    }

    pub fn get(&self, x: i32, y: i32) -> TileVisibility {
        if x >= 0 && x < 12 && y >= 0 && y < 12 {
            self.tiles[y as usize][x as usize]
        } else {
            TileVisibility::Unexplored
        }
    }

    pub fn reset(&mut self) {
        self.tiles = [[TileVisibility::Unexplored; 12]; 12];
    }
}

// ---- Boss ----

#[derive(Component)]
pub struct Boss;

#[derive(States, Clone, Copy, Default, Debug, Hash, PartialEq, Eq)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
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

#[derive(Component)]
pub struct StatsPanel;

#[derive(Component)]
pub struct StatsHpText;

#[derive(Component)]
pub struct StatsFloorText;

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
    GameOver,
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
    PlayerResolve,
    EnemyResolve,
    ResolveEnvironment,
    ApplyConsequences,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerAction {
    Move(IVec2),
    UseConsumable(usize),
}

#[derive(Resource, Default)]
pub struct PendingAction(pub Option<PlayerAction>);
