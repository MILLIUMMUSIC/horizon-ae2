// SPDX-License-Identifier: LGPL-3.0-or-later
// Native world view of center-only CableBus parts. Full attachments/facades,
// source client packet and smooth block-model lighting are separate port work.
use super::ae2_cable::{Cable, CableType};
use super::ae2_cable_render::{self as cable_render, Core, RenderState};
use super::ae2_cube_builder::Texture;
use super::ae2_grid::Direction;
use super::ae2_part_geometry::{reporting_host_boxes, select_primary};
use crate::{containers::BlockEntity, hud::Rect, mesher::Vertex, world::World};
use horizon_core::geometry::{Aabb, ShapeOp, Vec3, VoxelShape};
use horizon_world::{
    nbt::{JavaString, Tag},
    registry::BlockRegistry,
};
use std::collections::HashMap;
pub const MAGIC: i32 = 0x414532;
const BLOCK: &str = "ae2:cable_bus";
fn profile(world: &World, registry: &BlockRegistry, p: [i32; 3]) -> Option<Cable> {
    let state = registry.state(world.block(p[0], p[1], p[2]))?;
    if state.name != BLOCK {
        return None;
    }
    let BlockEntity::Mod(be) = world.block_entities.get(&(p[0], p[1], p[2]))? else {
        return None;
    };
    if be.id != BLOCK {
        return None;
    }
    let Tag::Compound(part) = be.nbt.get(&JavaString::from("cable"))? else {
        return None;
    };
    let Tag::String(id) = part.get(&JavaString::from("id"))? else {
        return None;
    };
    Cable::from_item(&id.to_string().ok()?)
}
fn view(world: &World, registry: &BlockRegistry, p: [i32; 3]) -> Option<(Cable, u8, [i32; 6])> {
    let cable = profile(world, registry, p)?;
    let BlockEntity::Mod(be) = world.block_entities.get(&(p[0], p[1], p[2]))? else {
        return None;
    };
    if be.data.len() == 8 && be.data[0] == MAGIC {
        Some((
            cable,
            be.data[1] as u8,
            std::array::from_fn(|i| be.data[i + 2]),
        ))
    } else {
        Some((cable, 0, [0; 6]))
    }
}
fn dense(world: &World, registry: &BlockRegistry, p: [i32; 3]) -> u8 {
    Direction::ALL
        .into_iter()
        .enumerate()
        .fold(0, |bits, (i, d)| {
            let delta = d.offset();
            let q = std::array::from_fn(|a| p[a].wrapping_add(delta[a]));
            let is_dense = profile(world, registry, q).is_some_and(|c| c.kind.is_dense())
                || registry
                    .state(world.block(q[0], q[1], q[2]))
                    .is_some_and(|s| s.name == "ae2:controller");
            bits | if is_dense { 1 << i } else { 0 }
        })
}
pub fn outline(world: &World, registry: &BlockRegistry, p: [i32; 3]) -> Option<Vec<[f64; 6]>> {
    if registry.state(world.block(p[0], p[1], p[2]))?.name != BLOCK {
        return None;
    }
    Some(
        view(world, registry, p).map_or_else(Vec::new, |(c, mask, _)| {
            reporting_host_boxes(c.kind, 0, mask, dense(world, registry, p), true)
                .into_iter()
                .map(|(_, local)| std::array::from_fn(|i| local.0[i] + f64::from(p[i % 3])))
                .collect()
        }),
    )
}
pub fn collision(world: &World, registry: &BlockRegistry, p: [i32; 3]) -> Option<VoxelShape> {
    if registry.state(world.block(p[0], p[1], p[2]))?.name != BLOCK {
        return None;
    }
    let Some((c, mask, _)) = view(world, registry, p) else {
        return Some(VoxelShape::empty());
    };
    let mut result = VoxelShape::empty();
    for (_, bounds) in reporting_host_boxes(c.kind, 0, mask, dense(world, registry, p), false) {
        let v = bounds.0;
        let shape = VoxelShape::from_box(Aabb::new(
            Vec3::new(v[0], v[1], v[2]),
            Vec3::new(v[3], v[4], v[5]),
        ))
        .ok()?;
        result = result.join(&shape, ShapeOp::OR).ok()?;
    }
    Some(result)
}
pub fn selects_center(world: &World, registry: &BlockRegistry, p: [i32; 3], hit: [f64; 3]) -> bool {
    let Some((c, mask, _)) = view(world, registry, p) else {
        return false;
    };
    let boxes = reporting_host_boxes(c.kind, 0, mask, dense(world, registry, p), true);
    let local = std::array::from_fn(|i| hit[i] - f64::from(p[i]));
    select_primary(&boxes, local) == Some(None)
}
fn external(registry: &BlockRegistry, world: &World, q: [i32; 3]) -> CableType {
    if let Some(c) = profile(world, registry, q) {
        return c.kind;
    }
    match registry
        .state(world.block(q[0], q[1], q[2]))
        .map(|s| s.name.as_str())
    {
        Some("ae2:controller") => CableType::DenseSmart,
        Some("ae2:drive") => CableType::Smart,
        Some(
            "ae2:charger"
            | "ae2:inscriber"
            | "ae2:energy_cell"
            | "ae2:dense_energy_cell"
            | "ae2:creative_energy_cell",
        ) => CableType::Covered,
        // The default AEBaseBlockEntity cable connection type.
        _ => CableType::Glass,
    }
}
pub fn geometry(
    world: &World,
    registry: &BlockRegistry,
    camera: glam::DVec3,
    sprites: &HashMap<String, Rect>,
    out: &mut Vec<Vertex>,
) {
    for (&(x, y, z), _) in &world.block_entities {
        let p = [x, y, z];
        let Some((cable, mask, channels)) = view(world, registry, p) else {
            continue;
        };
        let origin =
            (glam::DVec3::new(f64::from(x), f64::from(y), f64::from(z)) - camera).as_vec3();
        if origin.length_squared() > 64. * 64. {
            continue;
        }
        let mut adjacent = 0;
        let connections = std::array::from_fn(|i| {
            if mask & (1 << i) == 0 {
                return None;
            }
            let d = Direction::ALL[i].offset();
            let q = std::array::from_fn(|a| p[a].wrapping_add(d[a]));
            if profile(world, registry, q).is_some() {
                adjacent |= 1 << i;
            }
            Some(cable.kind.min(external(registry, world, q)))
        });
        let state = RenderState {
            cable,
            core: Core::of(cable.kind).unwrap(),
            connections,
            adjacent,
            channels,
            attachments: [None; 6],
            attachment_requires_connection: false,
        };
        let quads = cable_render::render(&state, &|id| Texture::unit(id));
        let light = crate::block_entity_render::vertex_light(world.light(x, y, z));
        for quad in quads {
            let Some(rect) = sprites.get(&quad.texture) else {
                continue;
            };
            let a = (quad.color >> 24) as u8;
            let shade = match quad.face {
                Direction::Down => 0.5,
                Direction::Up => 1.0,
                Direction::North | Direction::South => 0.8,
                Direction::East | Direction::West => 0.6,
            };
            let color = u32::from_le_bytes([
                (((quad.color >> 16) & 255) as f32 * shade) as u8,
                (((quad.color >> 8) & 255) as f32 * shade) as u8,
                ((quad.color & 255) as f32 * shade) as u8,
                a,
            ]);
            // EntityBatch consumes quads and creates its own triangle indices.
            for i in 0..4 {
                let pos = (origin + glam::Vec3::from_array(quad.positions[i])).to_array();
                let uv = quad.uv[i];
                out.push(Vertex {
                    pos,
                    uv: [
                        rect.u0 + (rect.u1 - rect.u0) * uv[0],
                        rect.v0 + (rect.v1 - rect.v0) * uv[1],
                    ],
                    layer: 0,
                    color,
                    light: if quad.emissive { 0xf0f0 } else { light },
                    normal: crate::mesher::CULL_BACK,
                    glint_uv: [0.; 2],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_world::registry::BlockStateId;
    use serde_json::json;

    #[test]
    fn actual_world_view_keeps_dense_outline_collision_and_local_selection_distinct() {
        let registry = BlockRegistry::from_facts_json(
            json!({"states":[
                ["",true,false,"","minecraft:air"],
                ["",false,false,"","ae2:cable_bus"]
            ],"defaults":{"minecraft:air":0,"ae2:cable_bus":1}})
            .to_string()
            .as_bytes(),
        )
        .unwrap();
        for p in [[0, 74, 0], [30_000_000, 74, -30_000_000]] {
            let mut world = World::new(BlockStateId(0), 42);
            world.chunks.insert(
                (p[0] >> 4, p[2] >> 4),
                std::sync::Arc::new(crate::chunk::Chunk::empty(p[0] >> 4, p[2] >> 4)),
            );
            world.set(p[0], p[1], p[2], BlockStateId(1)).unwrap();
            for kind in CableType::VALID {
                let cable = Cable {
                    kind,
                    color: super::super::ae2_cable::Color::Transparent,
                };
                world.block_entities.insert(
                    (p[0], p[1], p[2]),
                    BlockEntity::Mod(Box::new(crate::containers::ModBlockEntity {
                        id: BLOCK.into(),
                        nbt: [(
                            "cable".into(),
                            Tag::compound([(
                                "id",
                                Tag::String(cable.item().unwrap().as_str().into()),
                            )]),
                        )]
                        .into(),
                        ..Default::default()
                    })),
                );
                let (low, high) = if kind.is_dense() {
                    (3., 13.)
                } else if kind == CableType::Glass {
                    (6., 10.)
                } else {
                    (5., 11.)
                };
                assert_eq!(
                    outline(&world, &registry, p).unwrap(),
                    vec![std::array::from_fn(
                        |i| f64::from(p[i % 3]) + if i < 3 { low / 16. } else { high / 16. }
                    )]
                );
                let (low, high) = if kind.is_dense() {
                    (4.9, 11.1)
                } else {
                    (low, high)
                };
                assert_eq!(
                    collision(&world, &registry, p).unwrap().bounds(),
                    Some(Aabb::new(
                        Vec3::new(low / 16., low / 16., low / 16.),
                        Vec3::new(high / 16., high / 16., high / 16.)
                    ))
                );
                let edge = [
                    f64::from(p[0]) + 0.2,
                    f64::from(p[1]) + 0.5,
                    f64::from(p[2]) + 0.5,
                ];
                assert_eq!(selects_center(&world, &registry, p, edge), kind.is_dense());
                assert!(selects_center(
                    &world,
                    &registry,
                    p,
                    p.map(|v| f64::from(v) + 0.5)
                ));
            }
        }
    }
}
