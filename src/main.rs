mod datalog;
mod tags;

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use std::collections::BTreeSet;

use datalog::run_tag_inference;
use tags::{DerivedTags, EntityName, Tag, Tags};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::log::LogPlugin::default())
        .add_systems(Startup, spawn_entities)
        .add_systems(Update, (run_tag_inference, print_state).chain())
        .run();
}

fn spawn_entities(mut commands: Commands) {
    // Wooden Barrel: Wooden, Hot → expected: Flammable, OnFire, Dangerous
    commands.spawn((
        EntityName("Wooden Barrel"),
        Tags(BTreeSet::from([Tag::Wooden, Tag::Hot])),
    ));

    // Oil Pool: Flammable → expected: (none)
    commands.spawn((
        EntityName("Oil Pool"),
        Tags(BTreeSet::from([Tag::Flammable])),
    ));

    // Wet Wooden Door: Wooden, Hot, Wet → expected: Flammable, Steaming
    commands.spawn((
        EntityName("Wet Wooden Door"),
        Tags(BTreeSet::from([Tag::Wooden, Tag::Hot, Tag::Wet])),
    ));

    // Campfire: Hot → expected: (none)
    commands.spawn((
        EntityName("Campfire"),
        Tags(BTreeSet::from([Tag::Hot])),
    ));
}

fn print_state(
    query: Query<(&EntityName, &Tags, Option<&DerivedTags>)>,
    mut printed: Local<bool>,
    mut exit: MessageWriter<AppExit>,
) {
    if *printed {
        return;
    }
    *printed = true;

    info!("=== Entity State Summary ===");
    for (name, tags, derived) in query.iter() {
        let derived_set = derived.map(|d| &d.0);
        info!(
            "{}: input={:?}, derived={:?}",
            name.0, tags.0, derived_set
        );
    }
    info!("============================");

    exit.write(AppExit::Success);
}
