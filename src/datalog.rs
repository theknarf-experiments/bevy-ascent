use ascent::ascent_run;
use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::tags::{DerivedTags, Tag, Tags};

pub fn run_tag_inference(mut commands: Commands, query: Query<(Entity, &Tags)>) {
    // Flatten all (Entity, Tag) pairs from the query
    let has_tag: Vec<(Entity, Tag)> = query
        .iter()
        .flat_map(|(entity, tags)| tags.0.iter().map(move |&tag| (entity, tag)))
        .collect();

    let result = ascent_run! {
        relation has_tag(Entity, Tag) = has_tag;

        // Decompose into boolean relations
        relation is_wooden(Entity);
        relation is_flammable(Entity);
        relation is_hot(Entity);
        relation is_wet(Entity);

        is_wooden(e) <-- has_tag(e, t) if *t == Tag::Wooden;
        is_flammable(e) <-- has_tag(e, t) if *t == Tag::Flammable;
        is_hot(e) <-- has_tag(e, t) if *t == Tag::Hot;
        is_wet(e) <-- has_tag(e, t) if *t == Tag::Wet;

        // Derivation rules
        // Wooden → Flammable
        relation derived(Entity, Tag);
        derived(e, Tag::Flammable) <-- is_wooden(e);

        // Feed derived Flammable back into is_flammable
        is_flammable(e) <-- derived(e, t) if *t == Tag::Flammable;

        // Flammable + Hot + NOT Wet → OnFire
        derived(e, Tag::OnFire) <-- is_flammable(e), is_hot(e), !is_wet(e);

        // OnFire → Dangerous
        relation is_on_fire(Entity);
        is_on_fire(e) <-- derived(e, t) if *t == Tag::OnFire;
        derived(e, Tag::Dangerous) <-- is_on_fire(e);

        // Wet + Hot + Flammable → Steaming
        derived(e, Tag::Steaming) <-- is_wet(e), is_hot(e), is_flammable(e);
    };

    // Collect derived tags per entity
    let mut derived_map: std::collections::HashMap<Entity, BTreeSet<Tag>> =
        std::collections::HashMap::new();

    for (entity, tag) in &result.derived {
        derived_map
            .entry(*entity)
            .or_default()
            .insert(*tag);
    }

    // Write back DerivedTags for all queried entities
    for (entity, _) in query.iter() {
        let tags = derived_map
            .remove(&entity)
            .unwrap_or_default();
        commands.entity(entity).insert(DerivedTags(tags));
    }
}
