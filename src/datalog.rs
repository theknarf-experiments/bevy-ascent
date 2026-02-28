use ascent::ascent_run;
use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;

pub fn resolve_environment(
    mut commands: Commands,
    query: Query<(Entity, &Tags, &GridPos)>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
) {
    // Gather all (Entity, Tag) pairs
    let has_tag: Vec<(Entity, Tag)> = query
        .iter()
        .flat_map(|(entity, tags, _)| tags.0.iter().map(move |&tag| (entity, tag)))
        .collect();

    // Gather all (Entity, x, y) positions
    let position: Vec<(Entity, i32, i32)> = query
        .iter()
        .map(|(entity, _, pos)| (entity, pos.0.x, pos.0.y))
        .collect();

    let result = ascent_run! {
        relation has_tag(Entity, Tag) = has_tag;
        relation position(Entity, i32, i32) = position;

        // Spatial relations: adjacent (4-directional, Manhattan distance 1)
        relation adjacent(Entity, Entity);
        adjacent(a, b) <--
            position(a, ax, ay),
            position(b, bx, by),
            if a != b && ((*ax - *bx).abs() + (*ay - *by).abs() == 1);

        // Same tile: entities at the same position
        relation same_tile(Entity, Entity);
        same_tile(a, b) <--
            position(a, ax, ay),
            position(b, bx, by),
            if a != b && ax == bx && ay == by;

        // Boolean decomposition
        relation is_flammable(Entity);
        relation is_on_fire(Entity);
        relation is_wet(Entity);

        // Base wet: only from input tags, used for fire-blocking (stratifiable)
        relation is_base_wet(Entity);
        is_base_wet(e) <-- has_tag(e, t) if *t == Tag::Wet;

        // Full wet: includes derived wetness (from melting)
        is_wet(e) <-- is_base_wet(e);

        // Material → Flammable
        is_flammable(e) <-- has_tag(e, t) if *t == Tag::Wood;
        is_flammable(e) <-- has_tag(e, t) if *t == Tag::Oil;
        is_flammable(e) <-- has_tag(e, t) if *t == Tag::Flesh;

        // Base OnFire
        is_on_fire(e) <-- has_tag(e, t) if *t == Tag::OnFire;

        // Derived results
        relation derived(Entity, Tag);
        derived(e, Tag::Flammable) <-- is_flammable(e);

        // Fire spreads: Flammable + adjacent(OnFire) + !BaseWet → OnFire
        // Uses is_base_wet (input-only) so the negation is stratifiable
        // Transitive closure: derived OnFire feeds back into is_on_fire
        derived(e, Tag::OnFire) <-- is_flammable(e), adjacent(e, other), is_on_fire(other), !is_base_wet(e);
        is_on_fire(e) <-- derived(e, t) if *t == Tag::OnFire;

        // Ice + adjacent(OnFire) → Melted
        relation is_ice(Entity);
        is_ice(e) <-- has_tag(e, t) if *t == Tag::Ice;
        derived(e, Tag::Melted) <-- is_ice(e), adjacent(e, other), is_on_fire(other);

        // Melted → Wet (derived wetness)
        derived(e, Tag::Wet) <-- derived(e, t) if *t == Tag::Melted;
        is_wet(e) <-- derived(e, t) if *t == Tag::Wet;

        // Wet + OnFire → Extinguished (uses full is_wet including derived)
        derived(e, Tag::Extinguished) <-- is_wet(e), is_on_fire(e);

        // Flesh + same_tile(OnFire) → TakingDamage
        relation is_flesh(Entity);
        is_flesh(e) <-- has_tag(e, t) if *t == Tag::Flesh;
        derived(e, Tag::TakingDamage) <-- is_flesh(e), same_tile(e, other), is_on_fire(other);

        // Flesh + adjacent(OnFire) → TakingDamage
        derived(e, Tag::TakingDamage) <-- is_flesh(e), adjacent(e, other), is_on_fire(other);
    };

    // Collect derived tags per entity
    let mut derived_map: std::collections::HashMap<Entity, BTreeSet<Tag>> =
        std::collections::HashMap::new();

    for (entity, tag) in &result.derived {
        derived_map.entry(*entity).or_default().insert(*tag);
    }

    // Write back DerivedTags for all queried entities
    for (entity, _, _) in query.iter() {
        let tags = derived_map.remove(&entity).unwrap_or_default();
        commands.entity(entity).insert(DerivedTags(tags));
    }

    next_phase.set(TurnPhase::ApplyConsequences);
}
