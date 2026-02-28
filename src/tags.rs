use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Tag {
    Flammable,
    Hot,
    Wet,
    Wooden,
    OnFire,
    Dangerous,
    Steaming,
}

#[derive(Component, Clone, Debug)]
pub struct Tags(pub BTreeSet<Tag>);

#[derive(Component, Clone, Debug)]
#[component(immutable, on_insert = on_derived_insert, on_replace = on_derived_replace)]
pub struct DerivedTags(pub BTreeSet<Tag>);

#[derive(Component, Clone, Debug)]
pub struct EntityName(pub &'static str);

fn on_derived_insert(world: DeferredWorld, ctx: HookContext) {
    let entity = ctx.entity;
    let name = world
        .get::<EntityName>(entity)
        .map(|n| n.0)
        .unwrap_or("unnamed");
    let tags = world.get::<DerivedTags>(entity).unwrap();
    info!("[hook:insert] {name}: derived tags = {:?}", tags.0);
}

fn on_derived_replace(world: DeferredWorld, ctx: HookContext) {
    let entity = ctx.entity;
    let name = world
        .get::<EntityName>(entity)
        .map(|n| n.0)
        .unwrap_or("unnamed");
    let old = world.get::<DerivedTags>(entity).unwrap();
    info!("[hook:replace] {name}: old derived tags = {:?}", old.0);
}
