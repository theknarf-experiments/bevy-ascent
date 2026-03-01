use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;

pub fn item_name(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::IronSword => "Iron Sword",
        ItemKind::FireBlade => "Fire Blade",
        ItemKind::PoisonDagger => "Poison Dagger",
        ItemKind::LeatherArmor => "Leather Armor",
        ItemKind::IronArmor => "Iron Armor",
        ItemKind::HealthPotion => "Health Potion",
        ItemKind::Antidote => "Antidote",
        ItemKind::FireResistPotion => "Fire Resist Potion",
        ItemKind::Gold => "Gold",
    }
}

pub fn item_glyph(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::IronSword => "/",
        ItemKind::FireBlade => "/",
        ItemKind::PoisonDagger => "\\",
        ItemKind::LeatherArmor => "[",
        ItemKind::IronArmor => "[",
        ItemKind::HealthPotion => "%",
        ItemKind::Antidote => "%",
        ItemKind::FireResistPotion => "%",
        ItemKind::Gold => "$",
    }
}

pub fn item_color(kind: &ItemKind) -> Color {
    match kind {
        ItemKind::IronSword => Color::srgb(0.8, 0.8, 0.9),
        ItemKind::FireBlade => Color::srgb(1.0, 0.6, 0.0),
        ItemKind::PoisonDagger => Color::srgb(0.2, 0.7, 0.1),
        ItemKind::LeatherArmor => Color::srgb(0.6, 0.4, 0.2),
        ItemKind::IronArmor => Color::srgb(0.7, 0.7, 0.8),
        ItemKind::HealthPotion => Color::srgb(1.0, 0.3, 0.3),
        ItemKind::Antidote => Color::srgb(0.3, 1.0, 0.3),
        ItemKind::FireResistPotion => Color::srgb(1.0, 0.5, 0.0),
        ItemKind::Gold => Color::srgb(1.0, 0.85, 0.0),
    }
}

fn item_tags(kind: &ItemKind) -> BTreeSet<Tag> {
    match kind {
        ItemKind::IronSword => BTreeSet::from([Tag::Metal]),
        ItemKind::FireBlade => BTreeSet::from([Tag::Metal, Tag::OnFire]),
        ItemKind::PoisonDagger => BTreeSet::from([Tag::Metal, Tag::Poisoned]),
        ItemKind::LeatherArmor => BTreeSet::from([Tag::Wood]),
        ItemKind::IronArmor => BTreeSet::from([Tag::Metal]),
        ItemKind::Gold => BTreeSet::from([Tag::Metal]),
        _ => BTreeSet::new(),
    }
}

pub fn spawn_item(commands: &mut Commands, kind: ItemKind, pos: IVec2) -> Entity {
    let tags = item_tags(&kind);
    let mut entity_commands = commands.spawn((
        GridPos(pos),
        Item,
        kind,
        DerivedTags::default(),
        FloorEntity,
        DespawnOnExit(GameState::Playing),
    ));

    if !tags.is_empty() {
        entity_commands.insert(Tags(tags));
    }

    match kind {
        ItemKind::IronSword => {
            entity_commands.insert((Equippable(EquipSlot::Weapon), WeaponDamage(1)));
        }
        ItemKind::FireBlade => {
            entity_commands.insert((Equippable(EquipSlot::Weapon), WeaponDamage(1)));
        }
        ItemKind::PoisonDagger => {
            entity_commands.insert((Equippable(EquipSlot::Weapon), WeaponDamage(1)));
        }
        ItemKind::LeatherArmor => {
            entity_commands.insert((Equippable(EquipSlot::Armor), ArmorDefense(1)));
        }
        ItemKind::IronArmor => {
            entity_commands.insert((Equippable(EquipSlot::Armor), ArmorDefense(1)));
        }
        ItemKind::HealthPotion | ItemKind::Antidote | ItemKind::FireResistPotion => {
            entity_commands.insert(Consumable);
        }
        ItemKind::Gold => {}
    }

    entity_commands.id()
}
