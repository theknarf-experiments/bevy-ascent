use ascent::ascent_run;
use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::components::*;

pub fn resolve_environment(
    mut commands: Commands,
    query: Query<(Entity, &Tags, &GridPos)>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    player_inventory: Query<(Entity, &Inventory), With<Player>>,
    item_tags: Query<&Tags, With<Item>>,
    fire_resist_query: Query<(), With<FireResistBuff>>,
) {
    // Gather all (Entity, Tag) pairs
    let mut has_tag: Vec<(Entity, Tag)> = query
        .iter()
        .flat_map(|(entity, tags, _)| tags.0.iter().map(move |&tag| (entity, tag)))
        .collect();

    // Inject equipped item tags onto the player entity
    if let Ok((player_entity, inventory)) = player_inventory.single() {
        for slot_entity in [inventory.weapon, inventory.armor].iter().flatten() {
            if let Ok(slot_tags) = item_tags.get(*slot_entity) {
                for &tag in &slot_tags.0 {
                    has_tag.push((player_entity, tag));
                }
            }
        }
        if fire_resist_query.get(player_entity).is_ok() {
            has_tag.push((player_entity, Tag::FireResist));
        }
    }

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
        relation is_flesh(Entity);
        relation is_ice(Entity);
        relation is_poisoned(Entity);
        relation is_electrified(Entity);
        relation is_conductive(Entity);
        relation is_metal(Entity);
        relation is_explosive(Entity);
        relation is_fire_resist(Entity);

        // Base relations (input-only, used for negation / self-immunity)
        relation is_base_wet(Entity);
        relation is_base_on_fire(Entity);
        relation is_base_poisoned(Entity);
        relation is_base_electrified(Entity);

        is_base_wet(e) <-- has_tag(e, t) if *t == Tag::Wet;
        is_base_on_fire(e) <-- has_tag(e, t) if *t == Tag::OnFire;
        is_base_poisoned(e) <-- has_tag(e, t) if *t == Tag::Poisoned;
        is_base_electrified(e) <-- has_tag(e, t) if *t == Tag::Electrified;
        is_fire_resist(e) <-- has_tag(e, t) if *t == Tag::FireResist;

        // Full wet: includes derived wetness (from melting)
        is_wet(e) <-- is_base_wet(e);

        // Material decomposition
        is_flammable(e) <-- has_tag(e, t) if *t == Tag::Wood;
        is_flammable(e) <-- has_tag(e, t) if *t == Tag::Oil;
        is_flammable(e) <-- has_tag(e, t) if *t == Tag::Flesh;
        is_flesh(e) <-- has_tag(e, t) if *t == Tag::Flesh;
        is_ice(e) <-- has_tag(e, t) if *t == Tag::Ice;
        is_metal(e) <-- has_tag(e, t) if *t == Tag::Metal;
        is_explosive(e) <-- has_tag(e, t) if *t == Tag::Explosive;

        // Base states
        is_on_fire(e) <-- has_tag(e, t) if *t == Tag::OnFire;
        is_poisoned(e) <-- has_tag(e, t) if *t == Tag::Poisoned;
        is_electrified(e) <-- has_tag(e, t) if *t == Tag::Electrified;

        // Conductivity: metal or wet
        is_conductive(e) <-- is_metal(e);
        is_conductive(e) <-- is_wet(e);

        // Derived results
        relation derived(Entity, Tag);
        derived(e, Tag::Flammable) <-- is_flammable(e);
        derived(e, Tag::Conductive) <-- is_conductive(e);

        // === FIRE RULES ===
        // Fire spreads: Flammable + adjacent(OnFire) + !BaseWet → OnFire
        derived(e, Tag::OnFire) <-- is_flammable(e), adjacent(e, other), is_on_fire(other), !is_base_wet(e);
        is_on_fire(e) <-- derived(e, t) if *t == Tag::OnFire;

        // Ice + adjacent(OnFire) → Melted
        derived(e, Tag::Melted) <-- is_ice(e), adjacent(e, other), is_on_fire(other);

        // Melted → Wet (derived wetness)
        derived(e, Tag::Wet) <-- derived(e, t) if *t == Tag::Melted;
        is_wet(e) <-- derived(e, t) if *t == Tag::Wet;

        // Wet + OnFire → Extinguished
        derived(e, Tag::Extinguished) <-- is_wet(e), is_on_fire(e);

        // Fire damage: Flesh near fire, immune if self is base OnFire or has FireResist
        derived(e, Tag::FireDamage) <-- is_flesh(e), same_tile(e, other), is_on_fire(other), !is_base_on_fire(e), !is_fire_resist(e);
        derived(e, Tag::FireDamage) <-- is_flesh(e), adjacent(e, other), is_on_fire(other), !is_base_on_fire(e), !is_fire_resist(e);

        // Melt damage: flesh that melted takes fire damage
        derived(e, Tag::FireDamage) <-- is_flesh(e), derived(e, t) if *t == Tag::Melted;

        // === POISON RULES ===
        // Poison spreads adjacently (transitive), blocked by base fire
        derived(e, Tag::Poisoned) <-- adjacent(e, other), is_poisoned(other), !is_base_on_fire(e);
        is_poisoned(e) <-- derived(e, t) if *t == Tag::Poisoned;

        // Poison + Flesh = PoisonDamage (immune if self is base Poisoned)
        derived(e, Tag::PoisonDamage) <-- is_flesh(e), adjacent(e, other), is_poisoned(other), !is_base_poisoned(e);
        derived(e, Tag::PoisonDamage) <-- is_flesh(e), same_tile(e, other), is_poisoned(other), !is_base_poisoned(e);

        // Fire cleanses poison
        derived(e, Tag::PoisonBurned) <-- is_poisoned(e), is_on_fire(e);
        derived(e, Tag::PoisonBurned) <-- is_poisoned(e), adjacent(e, other), is_on_fire(other);

        // === ELECTRICITY RULES ===
        // Electricity conducts through conductive entities (transitive)
        derived(e, Tag::Electrified) <-- is_conductive(e), adjacent(e, other), is_electrified(other);
        derived(e, Tag::Electrified) <-- is_conductive(e), same_tile(e, other), is_electrified(other);
        is_electrified(e) <-- derived(e, t) if *t == Tag::Electrified;

        // Electric + Flesh = ElectricDamage (immune if self is base Electrified)
        derived(e, Tag::ElectricDamage) <-- is_flesh(e), adjacent(e, other), is_electrified(other), !is_base_electrified(e);
        derived(e, Tag::ElectricDamage) <-- is_flesh(e), same_tile(e, other), is_electrified(other), !is_base_electrified(e);

        // === EXPLOSIVE RULES ===
        derived(e, Tag::Exploding) <-- is_explosive(e), is_on_fire(e);
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
