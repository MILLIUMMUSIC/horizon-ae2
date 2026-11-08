// SPDX-License-Identifier: LGPL-3.0-or-later
// Fluid transforms at each surviving ItemEntity's RETURN phase. Explosion
// recipes, client lightning and arbitrary fluid-tag/component codecs are pending.
use super::{Game, ae2_transform_throw as rules};
use horizon_client::{
    crafting::{Ingredient, recipe_result},
    entities::{ItemCells, ItemEntity},
    items::{ItemStack, Items},
    world::World,
};
use horizon_core::{geometry::Vec3, random::RandomSource};
use horizon_world::{
    fluid::{FluidKind, FluidState},
    registry::BlockRegistry,
};
use serde_json::Value;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[derive(Clone, Default)]
pub(super) struct Clock {
    count: i32,
    ticks: u32,
}
type Clocks = BTreeMap<u32, Clock>;
// The native API has no per-ItemEntity extension field usable across crates.
// Scope this transient sidecar by actual Game, world and dimension, clear it on
// lifecycle, prune dead IDs, and refuse discontinuous entity tick clocks.
type Scope = (usize, PathBuf, String);
thread_local! {static CLOCKS: RefCell<BTreeMap<Scope, Clocks>> = const {RefCell::new(BTreeMap::new())};}
fn scope(game: &Game<'_>) -> Scope {
    (
        std::ptr::from_ref(game) as usize,
        game.world_root.clone(),
        game.dim.id().to_owned(),
    )
}
pub(super) fn reset(game: &Game<'_>) {
    let id = std::ptr::from_ref(game) as usize;
    CLOCKS.with(|all| all.borrow_mut().retain(|key, _| key.0 != id));
}
pub(super) fn begin(game: &Game<'_>) -> Clocks {
    let mut clocks = CLOCKS.with(|all| all.borrow_mut().remove(&scope(game)).unwrap_or_default());
    if !clocks.is_empty() {
        let live = game
            .dropped
            .iter()
            .map(|item| (item.id, item.ticks))
            .collect::<BTreeMap<_, _>>();
        clocks.retain(|id, clock| live.get(id) == Some(&clock.ticks));
    }
    clocks
}
pub(super) fn finish(game: &Game<'_>, mut clocks: Clocks, items: &[ItemEntity], removed: &[bool]) {
    if !clocks.is_empty() {
        let live = items
            .iter()
            .enumerate()
            .filter(|(i, _)| !removed[*i])
            .map(|(_, item)| item.id)
            .collect::<BTreeSet<_>>();
        clocks.retain(|id, _| live.contains(id));
    }
    CLOCKS.with(|all| {
        all.borrow_mut().insert(scope(game), clocks);
    });
}

pub(super) struct Recipe {
    ingredients: Vec<Ingredient>,
    result: Option<ItemStack>,
    water: bool,
}
fn decode_recipe(json: &Value, items: &Items) -> Option<Recipe> {
    let water = match json.get("circumstance") {
        None => true,
        Some(c) if c["type"] == "fluid" && c["tag"] == "minecraft:water" => true,
        Some(c) if c["type"] == "fluid" && c["tag"] == "minecraft:lava" => false,
        _ => return None,
    };
    let ingredients = json
        .get("ingredients")?
        .as_array()?
        .iter()
        .map(|value| {
            super::ae2_recipe_ingredient::normalize(value)
                .and_then(|v| Ingredient::parse(&v, items))
        })
        .collect::<Option<Vec<_>>>()?;
    if ingredients.is_empty() {
        return None;
    }
    let result = json.get("result").and_then(|v| {
        if v.get("components").is_some() || v["id"] == "ae2:quantum_entangled_singularity" {
            return None;
        }
        recipe_result(v).filter(|s| s.count > 0)
    });
    Some(Recipe {
        ingredients,
        result,
        water,
    })
}
pub(super) fn recipes(game: &Game<'_>) -> Vec<Recipe> {
    game.recipes()
        .custom
        .get("ae2:transform")
        .into_iter()
        .flatten()
        .filter_map(|recipe| {
            let mut recipe = decode_recipe(&recipe.json, game.items())?;
            recipe.result = recipe
                .result
                .filter(|stack| game.items().get(&stack.item).is_some());
            Some(recipe)
        })
        .collect()
}

pub(super) fn eligible(stack: &ItemStack, recipes: &[Recipe]) -> bool {
    recipes.iter().any(|r| r.ingredients[0].test(stack))
}
// The fresh entity is visible to later queries/merges immediately, but the
// caller's original range bound prevents ticking it during this same pass.
pub(super) fn fresh(
    stack: ItemStack,
    position: [f64; 3],
    velocity: [f64; 3],
    id: u32,
    bob_offset: f32,
) -> ItemEntity {
    let position = Vec3::new(position[0], position[1], position[2]);
    ItemEntity {
        health: 5,
        fire_ticks: 0,
        id,
        stack,
        thrower: None,
        target: None,
        position,
        previous: position,
        velocity: Vec3::new(velocity[0], velocity[1], velocity[2]),
        on_ground: false,
        age: 0,
        pickup_delay: 0,
        bob_offset,
        ticks: 0,
    }
}

fn fluid(world: &World, registry: &BlockRegistry, item: &ItemEntity) -> Option<bool> {
    let b = item.bounds();
    let pos = [
        item.position.x.floor() as i32,
        ((b.min_position().y + b.max_position().y) / 2.0).floor() as i32,
        item.position.z.floor() as i32,
    ];
    let state = registry.state(world.block(pos[0], pos[1], pos[2]))?;
    match FluidState::from_block(state).ok()?.kind() {
        FluidKind::Water | FluidKind::FlowingWater => Some(true),
        FluidKind::Lava | FluidKind::FlowingLava => Some(false),
        _ => None,
    }
}
fn intersects(origin: Vec3, item: &ItemEntity) -> bool {
    let bounds = item.bounds();
    let (min, max) = (bounds.min_position(), bounds.max_position());
    min.x < origin.x + 1.0
        && max.x > origin.x - 1.0
        && min.y < origin.y + 1.0
        && max.y > origin.y - 1.0
        && min.z < origin.z + 1.0
        && max.z > origin.z - 1.0
}
#[allow(clippy::too_many_arguments)]
pub(super) fn after_tick(
    world: &mut World,
    registry: &BlockRegistry,
    items: &mut Vec<ItemEntity>,
    removed: &mut Vec<bool>,
    cells: &mut ItemCells,
    index: usize,
    recipes: &[Recipe],
    clocks: &mut Clocks,
    mut spawn: impl FnMut(ItemStack, [f64; 3], [f64; 3]) -> ItemEntity,
) {
    if removed[index] {
        return;
    }
    let item = &items[index];
    let eligible = eligible(&item.stack, recipes);
    let clock = if eligible {
        Some(clocks.entry(item.id).or_default())
    } else {
        clocks.get_mut(&item.id)
    };
    let Some(clock) = clock else {
        return;
    };
    if item.ticks != clock.ticks.wrapping_add(1) {
        clock.count = 0;
    }
    clock.ticks = item.ticks;
    if !eligible {
        return;
    }
    let wet = fluid(world, registry, item);
    let valid = wet.is_some_and(|water| {
        recipes
            .iter()
            .any(|r| r.water == water && r.ingredients[0].test(&item.stack))
    });
    rules::fluid_tick(&mut clock.count, false, eligible, valid, || {
        let origin = items[index].position;
        let counts = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if !removed[i] && intersects(origin, item) {
                    item.stack.count
                } else {
                    0
                }
            })
            .collect::<Vec<_>>();
        for recipe in recipes.iter().filter(|r| Some(r.water) == wet) {
            let matching = recipe
                .ingredients
                .iter()
                .map(|ingredient| {
                    items
                        .iter()
                        .map(|item| ingredient.test(&item.stack))
                        .collect()
                })
                .collect::<Vec<_>>();
            let Some(claimed) = rules::claims(&counts, &matching, index) else {
                continue;
            };
            let Some(result) = recipe.result.as_ref() else {
                return false;
            };
            for (i, count) in claimed.into_iter().enumerate() {
                if count == 0 {
                    continue;
                }
                items[i].stack.count -= count;
                if items[i].stack.count == 0 {
                    removed[i] = true;
                    cells.remove(i, items[i].position);
                }
            }
            let random = &mut world.random;
            let position = [origin.x, origin.y, origin.z]
                .map(|v| v.floor() + 0.25 + random.next_double() * 0.5);
            let velocity = [(); 3].map(|_| random.next_double() * 0.25 - 0.125);
            items.push(spawn(result.clone(), position, velocity));
            removed.push(false);
            *cells = ItemCells::new(items);
            return true;
        }
        false
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::random::LegacyRandom;
    use horizon_world::registry::BlockStateId;
    use serde_json::json;
    fn fixture() -> (World, BlockRegistry, Vec<ItemEntity>, Vec<Recipe>) {
        let registry = BlockRegistry::from_facts_json(
            json!({"states":[
            ["",true,false,"","minecraft:air"],
            ["",false,true,"",{"id":"minecraft:water","properties":{"level":"0"}}]
        ],"defaults":{"minecraft:air":0,"minecraft:water":1}})
            .to_string()
            .as_bytes(),
        )
        .unwrap();
        let mut world = World::new(BlockStateId(0), 42);
        world.chunks.insert(
            (0, 0),
            std::sync::Arc::new(horizon_client::chunk::Chunk::empty(0, 0)),
        );
        world.set(0, 0, 0, BlockStateId(1)).unwrap();
        let items = [
            "ae2:charged_certus_quartz_crystal",
            "minecraft:redstone",
            "minecraft:quartz",
        ]
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let p = Vec3::new(0.5, 0.5, 0.5);
            ItemEntity {
                health: 5,
                fire_ticks: 0,
                id: i as u32 + 1,
                stack: ItemStack::new(*id, 1),
                thrower: None,
                target: None,
                position: p,
                previous: p,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                on_ground: false,
                age: 61,
                pickup_delay: 0,
                bob_offset: 0.0,
                ticks: 61,
            }
        })
        .collect();
        let recipe = Recipe {
            ingredients: [
                "ae2:charged_certus_quartz_crystal",
                "minecraft:redstone",
                "minecraft:quartz",
            ]
            .map(|id| Ingredient::of(vec![id.to_owned()]))
            .to_vec(),
            result: Some(ItemStack::new("ae2:fluix_crystal", 2)),
            water: true,
        };
        (world, registry, items, vec![recipe])
    }
    fn spawn(stack: ItemStack, position: [f64; 3], velocity: [f64; 3]) -> ItemEntity {
        fresh(stack, position, velocity, 4, 0.0)
    }
    fn clocks() -> Clocks {
        BTreeMap::from([(
            1,
            Clock {
                count: 60,
                ticks: 60,
            },
        )])
    }
    #[test]
    fn source_json_objects_are_decoded_before_native_ingredient_matching() {
        let items = Items::for_tests(Default::default());
        let recipe=decode_recipe(&json!({"type":"ae2:transform","ingredients":[
            {"item":"ae2:charged_certus_quartz_crystal"},{"item":"minecraft:redstone"},{"item":"minecraft:quartz"}
        ],"result":{"id":"ae2:fluix_crystal","count":2}}),&items).unwrap();
        assert!(recipe.water);
        assert_eq!(recipe.ingredients.len(), 3);
        assert!(
            recipe.ingredients[0].test(&ItemStack::new("ae2:charged_certus_quartz_crystal", 1))
        );
        assert!(recipe.ingredients[1].test(&ItemStack::new("minecraft:redstone", 1)));
        assert!(recipe.ingredients[2].test(&ItemStack::new("minecraft:quartz", 1)));
        assert_eq!(
            recipe.result.unwrap(),
            ItemStack::new("ae2:fluix_crystal", 2)
        );
    }
    #[test]
    fn temporarily_ineligible_stack_retains_its_entity_timer_without_tracking_unrelated_items() {
        let (mut world, registry, mut items, recipes) = fixture();
        items[0].stack = ItemStack::new("minecraft:dirt", 1);
        let mut cells = ItemCells::new(&items);
        let mut removed = vec![false; 3];
        let mut empty = Clocks::new();
        after_tick(
            &mut world,
            &registry,
            &mut items,
            &mut removed,
            &mut cells,
            0,
            &recipes,
            &mut empty,
            spawn,
        );
        assert!(empty.is_empty());
        let mut clocks = clocks();
        after_tick(
            &mut world,
            &registry,
            &mut items,
            &mut removed,
            &mut cells,
            0,
            &recipes,
            &mut clocks,
            spawn,
        );
        assert_eq!((clocks[&1].count, clocks[&1].ticks), (60, 61));
        items[0].stack = ItemStack::new("ae2:charged_certus_quartz_crystal", 1);
        items[0].ticks = 62;
        after_tick(
            &mut world,
            &registry,
            &mut items,
            &mut removed,
            &mut cells,
            0,
            &recipes,
            &mut clocks,
            spawn,
        );
        assert_eq!(items.len(), 4);
        assert_eq!(&*items[3].stack.item, "ae2:fluix_crystal");
    }
    #[test]
    fn output_uses_six_level_draws_and_consumed_neighbors_cannot_transform() {
        let (mut world, registry, mut items, recipes) = fixture();
        let mut cells = ItemCells::new(&items);
        let mut removed = vec![false; items.len()];
        let mut clocks = clocks();
        after_tick(
            &mut world,
            &registry,
            &mut items,
            &mut removed,
            &mut cells,
            0,
            &recipes,
            &mut clocks,
            spawn,
        );
        assert_eq!(removed, vec![true, true, true, false]);
        assert_eq!(items.len(), 4);
        assert_eq!(
            (&*items[3].stack.item, items[3].stack.count),
            ("ae2:fluix_crystal", 2)
        );
        let mut expected = LegacyRandom::new(42);
        let position = [(); 3].map(|_| 0.25 + expected.next_double() * 0.5);
        let velocity = [(); 3].map(|_| expected.next_double() * 0.25 - 0.125);
        assert_eq!(
            items[3].position,
            Vec3::new(position[0], position[1], position[2])
        );
        assert_eq!(
            (items[3].age, items[3].ticks, items[3].pickup_delay),
            (0, 0, 0)
        );
        assert_eq!(
            items[3].velocity,
            Vec3::new(velocity[0], velocity[1], velocity[2])
        );
        assert_eq!(world.random.next_long(), expected.next_long());
        after_tick(
            &mut world,
            &registry,
            &mut items,
            &mut removed,
            &mut cells,
            1,
            &recipes,
            &mut clocks,
            spawn,
        );
        assert_eq!(items.len(), 4);
    }
    #[test]
    fn removed_entities_and_unrepresentable_results_do_not_consume_or_draw() {
        for dead in [false, true] {
            let (mut world, registry, mut items, mut recipes) = fixture();
            recipes[0].result = None;
            let original = items
                .iter()
                .map(|item| item.stack.clone())
                .collect::<Vec<_>>();
            let mut cells = ItemCells::new(&items);
            let mut removed = vec![dead, false, false];
            let mut clocks = clocks();
            after_tick(
                &mut world,
                &registry,
                &mut items,
                &mut removed,
                &mut cells,
                0,
                &recipes,
                &mut clocks,
                spawn,
            );
            assert_eq!(
                items
                    .iter()
                    .map(|item| item.stack.clone())
                    .collect::<Vec<_>>(),
                original
            );
            assert_eq!(items.len(), 3);
            assert_eq!(clocks[&1].count, if dead { 60 } else { 0 });
            assert_eq!(world.random.next_long(), LegacyRandom::new(42).next_long());
        }
    }
    #[test]
    fn samples_current_position_and_rejects_discontinuous_reused_entity_clock() {
        let (mut world, registry, mut items, recipes) = fixture();
        items[0].previous = Vec3::new(100.0, 100.0, 100.0);
        items[0].ticks = 1; // same numeric ID belongs to a new native entity
        let mut cells = ItemCells::new(&items);
        let mut removed = vec![false; 3];
        let mut clocks = clocks();
        after_tick(
            &mut world,
            &registry,
            &mut items,
            &mut removed,
            &mut cells,
            0,
            &recipes,
            &mut clocks,
            spawn,
        );
        assert_eq!(clocks[&1].count, 1);
        assert_eq!(items.len(), 3);
        items[0].ticks = 2;
        clocks.get_mut(&1).unwrap().count = 60;
        after_tick(
            &mut world,
            &registry,
            &mut items,
            &mut removed,
            &mut cells,
            0,
            &recipes,
            &mut clocks,
            spawn,
        );
        assert_eq!(items.len(), 4);
    }
}
