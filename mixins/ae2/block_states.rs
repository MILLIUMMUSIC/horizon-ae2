// SPDX-License-Identifier: LGPL-3.0-or-later
// Complete the owned effective resource root with the original AE2 cube states.
// The content API initially contributes one iron-block row. No synthetic blocks
// or template states enter the resulting registry, and vanilla ids never move.
use super::{ContentMod, Result, ae2_controller};
use horizon_world::nbt::{self, Compound, JavaString, Limits, Tag};
use serde_json::{Value, json};
use std::{cell::Cell, fs, path::Path};

thread_local! {
    static BUILDING: Cell<bool> = const { Cell::new(false) };
}
struct BuildGuard;
impl Drop for BuildGuard {
    fn drop(&mut self) {
        BUILDING.with(|flag| flag.set(false));
    }
}

pub(super) fn build(
    base: &Path,
    mods: &[ContentMod],
    out: &Path,
) -> core::ops::ControlFlow<Result<Vec<String>>> {
    let declarations: Vec<_> = mods
        .iter()
        .flat_map(|m| &m.content.blocks)
        .filter_map(|block| Definition::from_block(&block.id).map(|kind| (kind, block)))
        .collect();
    if declarations.is_empty() {
        return core::ops::ControlFlow::Continue(());
    }
    if BUILDING.with(|flag| flag.replace(true)) {
        return core::ops::ControlFlow::Continue(());
    }
    let _guard = BuildGuard;
    let result = (|| {
        for (kind, block) in &declarations {
            if block.template != kind.template()
                || (block.shapes.is_some() && *kind != Definition::Crank)
                || !block.default_state.is_empty()
            {
                return Err(format!(
                    "AE2 state adapter requires its pinned template: {}",
                    block.id
                )
                .into());
            }
        }
        let warnings = super::build(base, mods, out)?;
        expand(
            out,
            &declarations
                .iter()
                .map(|(kind, _)| *kind)
                .collect::<Vec<_>>(),
        )?;
        Ok(warnings)
    })();
    core::ops::ControlFlow::Break(result)
}

/// Fully orientable source blocks: `facing` (6) and `spin` (0..3), with the
/// source default facing north and spin 0. Native rows are ordered facing
/// major and spin minor so the engine's cartesian state table accepts them.
const ORIENTED_FACINGS: [&str; 6] = ["north", "east", "south", "west", "up", "down"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Definition {
    Controller,
    CableBus,
    EnergyCell,
    DenseEnergyCell,
    CreativeEnergyCell,
    Charger,
    Inscriber,
    Drive,
    Crank,
}
impl Definition {
    fn from_block(id: &str) -> Option<Self> {
        match id {
            "ae2:controller" => Some(Self::Controller),
            "ae2:cable_bus" => Some(Self::CableBus),
            "ae2:energy_cell" => Some(Self::EnergyCell),
            "ae2:dense_energy_cell" => Some(Self::DenseEnergyCell),
            "ae2:creative_energy_cell" => Some(Self::CreativeEnergyCell),
            "ae2:charger" => Some(Self::Charger),
            "ae2:inscriber" => Some(Self::Inscriber),
            "ae2:drive" => Some(Self::Drive),
            "ae2:crank" => Some(Self::Crank),
            _ => None,
        }
    }
    fn block(self) -> &'static str {
        match self {
            Self::Controller => ae2_controller::BLOCK,
            Self::CableBus => "ae2:cable_bus",
            Self::EnergyCell => "ae2:energy_cell",
            Self::DenseEnergyCell => "ae2:dense_energy_cell",
            Self::CreativeEnergyCell => "ae2:creative_energy_cell",
            Self::Charger => "ae2:charger",
            Self::Inscriber => "ae2:inscriber",
            Self::Drive => "ae2:drive",
            Self::Crank => "ae2:crank",
        }
    }
    fn template(self) -> &'static str {
        match self {
            Self::CableBus => "minecraft:glass",
            Self::Charger | Self::Inscriber | Self::Drive | Self::Crank => "minecraft:dispenser",
            _ => "minecraft:iron_block",
        }
    }
    fn variants(self) -> usize {
        match self {
            Self::Controller => ae2_controller::VARIANTS,
            Self::CableBus => 32,
            Self::CreativeEnergyCell => 1,
            Self::Inscriber => 48,
            Self::Charger | Self::Drive => 24,
            Self::Crank => 6,
            _ => 5,
        }
    }
    /// Rows the pinned template initially contributes for this block.
    fn source_rows(self) -> usize {
        match self {
            Self::Charger | Self::Inscriber | Self::Drive | Self::Crank => 12,
            _ => 1,
        }
    }
    /// The template row copied for native row `index`. Oriented blocks keep
    /// the facing-dependent template values and use its untriggered row.
    fn source_index(self, index: usize) -> usize {
        match self {
            // The dispenser template orders (north, east, south, west, up,
            // down) with its untriggered row second. Full orientation streams
            // spin fastest; the Crank has only the facing property.
            Self::Inscriber => (index / 8) * 2 + 1,
            Self::Charger | Self::Drive => (index / 4) * 2 + 1,
            Self::Crank => index * 2 + 1,
            _ => 0,
        }
    }
    /// The native row replacing a template row. Both triggered template rows
    /// collapse onto the same spin-zero orientation.
    fn old_to_new(self, old: usize) -> usize {
        match self {
            Self::Inscriber => old / 2 * 8,
            Self::Charger | Self::Drive => old / 2 * 4,
            Self::Crank => old / 2,
            _ => 0,
        }
    }
    /// Source hosts whose concrete default is facing north (and spin zero
    /// where present). Structure rotate/mirror are separate inherited hooks.
    fn oriented(self) -> bool {
        matches!(
            self,
            Self::Charger | Self::Inscriber | Self::Drive | Self::Crank
        )
    }
    /// Full orientation adds the source spin property.
    fn full(self) -> bool {
        matches!(self, Self::Charger | Self::Inscriber | Self::Drive)
    }
    fn properties(self, index: usize) -> Value {
        match self {
            Self::Controller => properties(index),
            Self::CableBus => {
                json!({"light_level":(index/2).to_string(),"waterlogged":(index%2==1).to_string()})
            }
            Self::CreativeEnergyCell => json!({}),
            Self::Inscriber => json!({
                "facing": ORIENTED_FACINGS[index / 8],
                "spin": ((index / 2) % 4).to_string(),
                "waterlogged": (index % 2 == 1).to_string(),
            }),
            Self::Charger | Self::Drive => json!({
                "facing": ORIENTED_FACINGS[index / 4],
                "spin": (index % 4).to_string(),
            }),
            Self::Crank => json!({"facing": ORIENTED_FACINGS[index]}),
            _ => json!({"fullness": index.to_string()}),
        }
    }
    fn schema(self) -> Value {
        match self {
            Self::Controller => {
                json!({"state":{"named":ae2_controller::STATES}, "type":{"named":ae2_controller::TYPES}})
            }
            Self::CreativeEnergyCell => json!({}),
            Self::CableBus => {
                json!({"light_level":{"integer":[0,15]},"waterlogged":{"named":["false","true"]}})
            }
            Self::Inscriber => json!({
                "facing":{"named":ORIENTED_FACINGS},
                "spin":{"integer":[0,3]},
                "waterlogged":{"named":["false","true"]},
            }),
            Self::Charger | Self::Drive => json!({
                "facing":{"named":ORIENTED_FACINGS},
                "spin":{"integer":[0,3]},
            }),
            Self::Crank => json!({"facing":{"named":ORIENTED_FACINGS}}),
            _ => json!({"fullness":{"integer":[0,4]}}),
        }
    }
    fn commands(self) -> Value {
        match self {
            Self::Controller => json!([
                ["state", ae2_controller::STATES],
                ["type", ae2_controller::TYPES]
            ]),
            Self::CreativeEnergyCell => json!([]),
            Self::CableBus => json!([
                [
                    "light_level",
                    (0..16).map(|i| i.to_string()).collect::<Vec<_>>()
                ],
                ["waterlogged", ["false", "true"]]
            ]),
            Self::Inscriber => json!([
                ["facing", ORIENTED_FACINGS],
                ["spin", ["0", "1", "2", "3"]],
                ["waterlogged", ["false", "true"]]
            ]),
            Self::Charger | Self::Drive => {
                json!([["facing", ORIENTED_FACINGS], ["spin", ["0", "1", "2", "3"]]])
            }
            Self::Crank => json!([["facing", ORIENTED_FACINGS]]),
            _ => json!([["fullness", ["0", "1", "2", "3", "4"]]]),
        }
    }
}

#[derive(Clone, Copy)]
struct Layout {
    first: usize,
    old_len: usize,
    kind: Definition,
}
impl Layout {
    /// Row-count change; negative when the source state set is smaller than
    /// the template's (the Crank drops the dispenser's triggered dimension).
    fn extra(self) -> i64 {
        self.kind.variants() as i64 - self.kind.source_rows() as i64
    }
    fn replaced(self) -> core::ops::Range<usize> {
        self.first..self.first + self.kind.source_rows()
    }
    fn id(self, old: u64) -> u64 {
        let old = old as usize;
        if self.replaced().contains(&old) {
            (self.first + self.kind.old_to_new(old - self.first)) as u64
        } else if old >= self.replaced().end && old < self.old_len {
            (old as i64 + self.extra()) as u64
        } else {
            old as u64
        }
    }
    fn remap(self, value: &mut Value) {
        match value {
            Value::Number(n) => {
                if let Some(id) = n.as_u64() {
                    *n = self.id(id).into();
                }
            }
            Value::Array(a) => {
                for v in a {
                    self.remap(v);
                }
            }
            Value::Object(o) => {
                for v in o.values_mut() {
                    self.remap(v);
                }
            }
            _ => {}
        }
    }
    fn replacement(self, rows: &[Value]) -> Vec<Value> {
        (0..self.kind.variants())
            .map(|i| rows[self.first + self.kind.source_index(i)].clone())
            .collect()
    }
    fn rows(self, value: &mut Value) -> Result<()> {
        let rows = value
            .as_array_mut()
            .ok_or("AE2 state table is not an array")?;
        if rows.len() != self.old_len {
            return Err("AE2 state table lengths disagree".into());
        }
        let replacement = self.replacement(rows);
        rows.splice(self.replaced(), replacement);
        Ok(())
    }
}

fn properties(index: usize) -> Value {
    json!({"state": ae2_controller::STATES[index / ae2_controller::TYPES.len()],
           "type": ae2_controller::TYPES[index % ae2_controller::TYPES.len()]})
}
fn read_json(root: &Path, name: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&owned(root, name)?)?)
}
fn owned(root: &Path, name: &str) -> Result<Vec<u8>> {
    let path = root.join(name);
    if fs::symlink_metadata(&path)?.file_type().is_symlink() {
        return Err(format!("refusing to rewrite linked base resource {name}").into());
    }
    Ok(fs::read(path)?)
}

fn expand(root: &Path, definitions: &[Definition]) -> Result<()> {
    let facts = read_json(root, "facts.json")?;
    let rows = facts["states"].as_array().ok_or("no facts states")?;
    let mut initial = Vec::new();
    for &kind in definitions {
        let matches: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r[0].as_str() == Some(kind.block()))
            .map(|(i, _)| i)
            .collect();
        if matches.len() != kind.source_rows() {
            return Err(format!(
                "expected {} initial states: {}",
                kind.source_rows(),
                kind.block()
            )
            .into());
        }
        initial.push((matches[0], kind));
    }
    initial.sort_by_key(|(id, _)| *id);
    let mut offset: i64 = 0;
    let mut layouts = Vec::new();
    for (first, kind) in initial {
        let first = first as i64 + offset;
        let old_len = rows.len() as i64 + offset;
        let layout = Layout {
            first: first as usize,
            old_len: old_len as usize,
            kind,
        };
        offset += layout.extra();
        layouts.push(layout);
    }
    let total = rows.len() as i64 + offset;
    if !(0..=super::MAX_STATES as i64).contains(&total) {
        return Err("AE2 states exceed the engine palette limit".into());
    }
    let mut outputs = Vec::new();
    for name in [
        "facts.json",
        "gameplay-facts.json",
        "block-behaviours.json",
        "model-states.json",
        "render-definitions.json",
        "command-registries.json",
        "mob-data.json",
        "worldgen/block_flags.json",
        "worldgen/blocks/structure-blocks.json",
    ] {
        let mut doc = read_json(root, name)?;
        for &layout in &layouts {
            rewrite_json(layout, name, &mut doc)?;
        }
        outputs.push((name, serde_json::to_vec(&doc)?));
    }
    for name in [
        "occlusion.nbt",
        "cached-collision.nbt",
        "movement-properties.nbt",
        "worldgen/blocks/block-properties.nbt",
        "worldgen/blocks/cached-support.nbt",
        "worldgen/blocks/cached-collision.nbt",
    ] {
        let bytes = owned(root, name)?;
        let (mut named, _) = nbt::read_named(&bytes, Limits::tables())?;
        for &layout in &layouts {
            if rewrite_nbt(layout, &mut named.tag)? == 0 {
                return Err(format!("no state arrays expanded in {name}").into());
            }
        }
        outputs.push((name, nbt::write_named(&named, Limits::tables())?));
    }
    if definitions.contains(&Definition::CableBus) {
        let mut report = read_json(root, "registries.json")?;
        let entries = report["minecraft:block_entity_type"]["entries"]
            .as_object_mut()
            .ok_or("no block entity type registry")?;
        if !entries.contains_key("ae2:cable_bus") {
            let next = entries
                .values()
                .filter_map(|v| v["protocol_id"].as_u64())
                .max()
                .map_or(0, |n| n + 1);
            entries.insert("ae2:cable_bus".into(), json!({"protocol_id":next}));
        }
        outputs.push(("registries.json", serde_json::to_vec(&report)?));
    }
    // Validate/encode every table before changing any of the owned files. The
    // engine writes its resource stamp only after this whole build succeeds.
    for (name, bytes) in outputs {
        fs::write(root.join(name), bytes)?;
    }
    Ok(())
}

fn rewrite_json(layout: Layout, name: &str, doc: &mut Value) -> Result<()> {
    match name {
        "facts.json" => {
            layout.rows(&mut doc["states"])?;
            for i in 0..layout.kind.variants() {
                doc["states"][layout.first + i][4]["properties"] = layout.kind.properties(i);
                if matches!(layout.kind, Definition::CableBus | Definition::Inscriber) {
                    doc["states"][layout.first + i][2] = json!(i % 2 == 1);
                }
            }
            layout.remap(&mut doc["defaults"]);
            if layout.kind.oriented() {
                // Source default: facing north (spin 0 for full orientation).
                doc["defaults"][layout.kind.block()] = json!(layout.first);
            }
        }
        "gameplay-facts.json" => {
            layout.rows(&mut doc["states"])?;
            let rows = doc["states"].as_array_mut().unwrap();
            for (id, row) in rows.iter_mut().enumerate().skip(layout.first) {
                row["id"] = json!(id);
            }
            for (i, row) in rows[layout.first..layout.first + layout.kind.variants()]
                .iter_mut()
                .enumerate()
            {
                row["block_entity"] = json!(true);
                if layout.kind == Definition::Inscriber {
                    row["dampening"] = json!(2);
                    row["can_occlude"] = json!(false);
                    row["solid_render"] = json!(false);
                }
                if layout.kind == Definition::CableBus {
                    row["emission"] = json!(i / 2);
                    row["full_collision"] = json!(false);
                    row["render_shape"] = json!("INVISIBLE");
                    row["outline"] = json!([[0.375, 0.375, 0.375, 0.625, 0.625, 0.625]]);
                    row["outline_edges"] = json!([]);
                    row["push"] = json!("BLOCK");
                }
            }
        }
        "block-behaviours.json" => {
            layout.rows(&mut doc["states"])?;
            let water = doc["fluids"]
                .as_array()
                .ok_or("missing fluid registry")?
                .iter()
                .position(|v| v.as_str() == Some("minecraft:water"))
                .ok_or("missing water fluid")?;
            if matches!(layout.kind, Definition::CableBus | Definition::Inscriber) {
                let blocks = doc["blocks"]
                    .as_array_mut()
                    .ok_or("missing behaviour blocks")?;
                let block = blocks
                    .iter_mut()
                    .find(|b| b["name"] == layout.kind.block())
                    .ok_or("missing waterlogged host behaviour")?;
                if layout.kind == Definition::Inscriber {
                    // AEBaseEntityBlock extends Block, not DispenserBlock.
                    block["class"] = json!(["Block"]);
                }
                let interfaces = block["interfaces"]
                    .as_array_mut()
                    .ok_or("missing waterlogged host interfaces")?;
                for name in [
                    "SimpleWaterloggedBlock",
                    "LiquidBlockContainer",
                    "BucketPickup",
                ] {
                    if !interfaces.iter().any(|v| v == name) {
                        interfaces.push(json!(name));
                    }
                }
            }
            for i in 0..layout.kind.variants() {
                let row = &mut doc["states"][layout.first + i];
                let flags = row[1].as_u64().ok_or("no behaviour flags")?;
                if matches!(layout.kind, Definition::CableBus | Definition::Inscriber) {
                    row[2] = json!(if i % 2 == 1 { water } else { 0 });
                    row[3] = json!(if i % 2 == 1 { 8 } else { 0 });
                    row[4] = json!(false);
                }
                row[1] = json!(
                    flags
                        | if !matches!(
                            layout.kind,
                            Definition::CreativeEnergyCell | Definition::CableBus
                        ) {
                            u64::from(horizon_world::level::data::state_flags::ANALOG_OUTPUT)
                        } else {
                            0
                        }
                );
            }
        }
        "mob-data.json" => layout.rows(&mut doc["states"])?,
        "worldgen/block_flags.json" => layout.rows(doc)?,
        "model-states.json" => {
            doc[layout.kind.block()]["schema"] = layout.kind.schema();
            doc[layout.kind.block()]["states"] = Value::Array(
                (0..layout.kind.variants())
                    .map(|i| layout.kind.properties(i))
                    .collect(),
            );
        }
        "render-definitions.json" => {
            layout.remap(doc);
            doc[layout.kind.block()]["states"] =
                json!((layout.first..layout.first + layout.kind.variants()).collect::<Vec<_>>());
            if layout.kind.oriented() {
                doc[layout.kind.block()]["default"] = json!(layout.first);
            }
        }
        "command-registries.json" => {
            doc["blocks"][layout.kind.block()] = layout.kind.commands();
        }
        "worldgen/blocks/structure-blocks.json" => {
            for key in ["rotate", "mirror"] {
                layout.remap(&mut doc[key]);
                layout.rows(&mut doc[key])?;
                // These AE2 hosts inherit BlockBehaviour.rotate/mirror,
                // which return the original state. An orientation strategy
                // enables wrench turns; it does not override structure hooks.
                for i in 0..layout.kind.variants() {
                    let row = doc[key][layout.first + i]
                        .as_array_mut()
                        .ok_or("no structure rotation row")?;
                    for id in row {
                        *id = json!(layout.first + i);
                    }
                }
            }
            layout.rows(&mut doc["full_block"])?;
            doc["states"] = json!(layout.old_len as i64 + layout.extra());
        }
        _ => return Err("unhandled AE2 state table".into()),
    }
    Ok(())
}

fn rewrite_nbt(layout: Layout, tag: &mut Tag) -> Result<usize> {
    let Tag::Compound(compound) = tag else {
        return Ok(0);
    };
    let mut cable_shape = None;
    if layout.kind == Definition::CableBus
        && compound.contains_key(&JavaString::from("cached_shapes"))
    {
        if let Some(Tag::List(shapes)) = compound.get_mut(&JavaString::from("shapes")) {
            let shape =
                horizon_core::geometry::VoxelShape::from_box(horizon_core::geometry::Aabb::new(
                    horizon_core::geometry::Vec3::new(0.375, 0.375, 0.375),
                    horizon_core::geometry::Vec3::new(0.625, 0.625, 0.625),
                ))?;
            cable_shape = Some(shapes.len() as i32);
            shapes.push(super::encode_shape(&shape));
        }
    }
    let mut changed = 0;
    for (key, value) in compound.iter_mut() {
        match (key.to_string_lossy().as_str(), value) {
            ("states", Tag::List(rows)) if rows.len() == layout.old_len => {
                changed += 1;
                let replacement: Vec<Tag> = (0..layout.kind.variants())
                    .map(|i| rows[layout.first + layout.kind.source_index(i)].clone())
                    .collect();
                rows.splice(layout.replaced(), replacement);
                for (i, row) in rows[layout.first..layout.first + layout.kind.variants()]
                    .iter_mut()
                    .enumerate()
                {
                    if let Tag::Compound(row) = row
                        && row.get(&JavaString::from("id"))
                            == Some(&Tag::String(layout.kind.block().into()))
                    {
                        let props: Compound = layout
                            .kind
                            .properties(i)
                            .as_object()
                            .unwrap()
                            .iter()
                            .map(|(key, value)| {
                                (
                                    key.as_str().into(),
                                    Tag::String(value.as_str().unwrap().into()),
                                )
                            })
                            .collect();
                        row.insert("properties".into(), Tag::Compound(props));
                    }
                }
            }
            ("cached_shapes", Tag::IntArray(rows)) if rows.len() == layout.old_len => {
                changed += 1;
                let replacement: Vec<i32> = (0..layout.kind.variants())
                    .map(|i| {
                        cable_shape.unwrap_or(rows[layout.first + layout.kind.source_index(i)])
                    })
                    .collect();
                rows.splice(layout.replaced(), replacement);
            }
            (_, inner @ Tag::Compound(_)) => changed += rewrite_nbt(layout, inner)?,
            _ => {}
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mods::Content;
    use std::path::PathBuf;

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let parent = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .to_owned();
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = parent.join(format!(
                "ae2-controller-states-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn content(root: &Path, id: &str, declarations: &[(&str, Value)]) -> ContentMod {
        content_assets(root, id, declarations, &[])
    }

    fn content_assets(
        root: &Path,
        id: &str,
        declarations: &[(&str, Value)],
        assets: &[(&str, Value)],
    ) -> ContentMod {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join("horizon-mod.toml"), format!("id=\"{id}\"\nname=\"Test\"\nversion=\"0.1.0\"\napi=\"0.2\"\nside=\"both\"\ncapabilities=[\"registries\"]\n")).unwrap();
        for (block, data) in declarations {
            let path = root.join(format!("data/{id}/horizon/block/{block}.json"));
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, serde_json::to_vec(data).unwrap()).unwrap();
        }
        for (relative, data) in assets {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, serde_json::to_vec(data).unwrap()).unwrap();
        }
        let package = horizon_mods::Package::read(root).unwrap();
        ContentMod {
            id: id.to_owned(),
            content: Content::read(root, &package.manifest).unwrap(),
        }
    }

    #[test]
    fn owned_controller_states_keep_vanilla_ids_and_foreign_shape_mappings() {
        let scratch = Scratch::new();
        let base = PathBuf::from(std::env::var_os("HORIZON_RESOURCES").unwrap());
        let before = fs::read(base.join("facts.json")).unwrap();
        let baseline: Value = serde_json::from_slice(&before).unwrap();
        let ae2 = content(
            &scratch.0.join("ae2"),
            "ae2",
            &[
                ("aaa_before", json!({"template":"minecraft:glass"})),
                (
                    "cable_bus",
                    json!({"template":"minecraft:glass", "item":false}),
                ),
                (
                    "controller",
                    json!({"template":"minecraft:iron_block","properties":{"hardness":6,"requires_tool":false}}),
                ),
                (
                    "creative_energy_cell",
                    json!({"template":"minecraft:iron_block"}),
                ),
                (
                    "dense_energy_cell",
                    json!({"template":"minecraft:iron_block"}),
                ),
                ("energy_cell", json!({"template":"minecraft:iron_block"})),
                ("zzz_door", json!({"template":"minecraft:oak_door"})),
            ],
        );
        let foreign = content(
            &scratch.0.join("foreign"),
            "zztest",
            &[(
                "shaped",
                json!({"template":"minecraft:stone_button", "default_state":{"powered":"true"},
                "shapes":{"property":"powered","values":{"false":[[0,0,0,0.5,0.5,0.5]],"true":[[0,0,0,1,1,1]]}}}),
            )],
        );
        let out = scratch.0.join("root");
        super::super::build(&base, &[foreign, ae2], &out).unwrap();
        let facts = read_json(&out, "facts.json").unwrap();
        let rows = facts["states"].as_array().unwrap();
        let old = baseline["states"].as_array().unwrap();
        assert_eq!(
            &rows[..old.len()],
            old.as_slice(),
            "vanilla state ids/rows never move"
        );
        assert_eq!(fs::read(base.join("facts.json")).unwrap(), before);
        for (name, value) in baseline["defaults"].as_object().unwrap() {
            assert_eq!(&facts["defaults"][name], value);
        }
        let controller: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r[0] == ae2_controller::BLOCK)
            .collect();
        let registry_before = read_json(&base, "registries.json").unwrap();
        let registry_after = read_json(&out, "registries.json").unwrap();
        let prior = registry_before["minecraft:block_entity_type"]["entries"]
            .as_object()
            .unwrap();
        for (key, value) in prior {
            assert_eq!(
                &registry_after["minecraft:block_entity_type"]["entries"][key],
                value
            );
        }
        assert!(
            registry_after["minecraft:block_entity_type"]["entries"]["ae2:cable_bus"]["protocol_id"]
                .as_u64()
                .unwrap() >= prior.len() as u64
        );
        assert_eq!(rows.iter().filter(|r| r[0] == "ae2:cable_bus").count(), 32);
        let behaviours = read_json(&out, "block-behaviours.json").unwrap();
        let water = behaviours["fluids"]
            .as_array()
            .unwrap()
            .iter()
            .position(|v| v == "minecraft:water")
            .unwrap();
        for (i, (id, _)) in rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r[0] == "ae2:cable_bus")
            .enumerate()
        {
            assert_eq!(
                behaviours["states"][id][2],
                json!(if i % 2 == 1 { water } else { 0 })
            );
            assert_eq!(
                behaviours["states"][id][3],
                json!(if i % 2 == 1 { 8 } else { 0 })
            );
        }
        assert_eq!(controller.len(), 18);
        for (i, (_, row)) in controller.iter().enumerate() {
            assert_eq!(row[4]["properties"], properties(i));
        }
        assert_eq!(
            facts["defaults"][ae2_controller::BLOCK],
            json!(controller[0].0)
        );
        let model = read_json(&out, "model-states.json").unwrap();
        assert_eq!(
            model[ae2_controller::BLOCK]["states"]
                .as_array()
                .unwrap()
                .len(),
            18
        );
        let command = read_json(&out, "command-registries.json").unwrap();
        assert_eq!(
            command["blocks"][ae2_controller::BLOCK],
            json!([
                ["state", ae2_controller::STATES],
                ["type", ae2_controller::TYPES]
            ])
        );
        let gameplay = read_json(&out, "gameplay-facts.json").unwrap();
        for (id, _) in &controller {
            assert_eq!(gameplay["states"][*id]["block_entity"], true);
        }
        for kind in [
            Definition::EnergyCell,
            Definition::DenseEnergyCell,
            Definition::CreativeEnergyCell,
        ] {
            let cell: Vec<_> = rows
                .iter()
                .enumerate()
                .filter(|(_, r)| r[0] == kind.block())
                .collect();
            assert_eq!(cell.len(), kind.variants());
            assert_eq!(facts["defaults"][kind.block()], cell[0].0);
            assert_eq!(model[kind.block()]["schema"], kind.schema());
            assert_eq!(command["blocks"][kind.block()], kind.commands());
            let behaviour = read_json(&out, "block-behaviours.json").unwrap();
            for (i, (id, row)) in cell.iter().enumerate() {
                assert_eq!(row[4]["properties"], kind.properties(i));
                assert_eq!(gameplay["states"][*id]["block_entity"], true);
                let flags = behaviour["states"][*id][1].as_u64().unwrap();
                assert_eq!(
                    flags & u64::from(horizon_world::level::data::state_flags::ANALOG_OUTPUT) != 0,
                    kind != Definition::CreativeEnergyCell
                );
            }
        }
        let (named, _) = nbt::read_named(
            &fs::read(out.join("cached-collision.nbt")).unwrap(),
            Limits::tables(),
        )
        .unwrap();
        let Tag::Compound(collision) = named.tag else {
            panic!("no collision root");
        };
        let Some(Tag::IntArray(mapping)) = collision.get(&JavaString::from("cached_shapes")) else {
            panic!("no mapping");
        };
        assert_eq!(mapping.len(), rows.len());
        let Some(Tag::List(shapes)) = collision.get(&JavaString::from("shapes")) else {
            panic!("no shapes");
        };
        let shaped: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r[0] == "zztest:shaped")
            .collect();
        assert_eq!(shaped.len(), 24);
        for (i, (id, _)) in shaped.iter().enumerate() {
            assert_eq!(mapping[*id] as usize, shapes.len() - 25 + i);
        }
        // Run the actual loaders too: all of the tables must agree, not merely
        // their lengths in a hand-written fixture.
        let assets = crate::assets::Assets::load(&out).unwrap();
        assert_eq!(assets.registry.len(), rows.len());
        let state = assets
            .registry
            .default_state(ae2_controller::BLOCK)
            .unwrap();
        assert_eq!(state.0 as usize, controller[0].0);
        let mut sprites = horizon_client_render::sprites::Sprites::default();
        let models = horizon_client_render::models::catalog::StateModels::load(
            &out,
            assets.registry,
            &[
                ae2_controller::BLOCK,
                "ae2:energy_cell",
                "ae2:dense_energy_cell",
                "ae2:creative_energy_cell",
                "ae2:zzz_door",
                "zztest:shaped",
            ],
            &mut sprites,
        )
        .unwrap();
        for (id, row) in rows.iter().enumerate().filter(|(_, row)| {
            Definition::from_block(row[0].as_str().unwrap()).is_some() && row[0] != "ae2:cable_bus"
        }) {
            let _ = row;
            assert!(
                !models
                    .collect(
                        horizon_world::registry::BlockStateId(id as u32),
                        horizon_core::position::BlockPos::new(0, 0, 0)
                    )
                    .unwrap()
                    .is_empty()
            );
        }
    }
    #[test]
    fn energy_cell_only_content_builds_without_controller_or_template_states() {
        let scratch = Scratch::new();
        let base = PathBuf::from(std::env::var_os("HORIZON_RESOURCES").unwrap());
        let ae2 = content(
            &scratch.0.join("ae2"),
            "ae2",
            &[("energy_cell", json!({"template":"minecraft:iron_block"}))],
        );
        let out = scratch.0.join("root");
        super::super::build(&base, &[ae2], &out).unwrap();
        let facts = read_json(&out, "facts.json").unwrap();
        let states: Vec<_> = facts["states"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row[0] == "ae2:energy_cell")
            .collect();
        assert_eq!(states.len(), 5);
        assert!(
            states
                .iter()
                .enumerate()
                .all(|(i, row)| row[4]["properties"] == Definition::EnergyCell.properties(i))
        );
        crate::assets::Assets::load(&out).unwrap();
    }

    #[test]
    fn oriented_hosts_keep_source_states_and_inherited_structure_identity() {
        let scratch = Scratch::new();
        let base = PathBuf::from(std::env::var_os("HORIZON_RESOURCES").unwrap());
        let states: Value = (0..24)
            .map(|i| {
                (
                    format!("facing={},spin={}", ORIENTED_FACINGS[i / 4], i % 4),
                    json!({"model": "ae2:block/charger"}),
                )
            })
            .collect::<serde_json::Map<_, _>>()
            .into();
        let drive_states: Value = (0..24)
            .map(|i| {
                (
                    format!("facing={},spin={}", ORIENTED_FACINGS[i / 4], i % 4),
                    json!({"model": "ae2:block/drive/drive_base"}),
                )
            })
            .collect::<serde_json::Map<_, _>>()
            .into();
        let ae2 = content_assets(
            &scratch.0.join("ae2"),
            "ae2",
            &[
                ("charger", json!({"template":"minecraft:dispenser"})),
                ("drive", json!({"template":"minecraft:dispenser"})),
            ],
            &[
                (
                    "assets/ae2/blockstates/charger.json",
                    json!({"variants": states}),
                ),
                (
                    "assets/ae2/blockstates/drive.json",
                    json!({"variants": drive_states}),
                ),
                (
                    "assets/ae2/models/block/charger.json",
                    json!({"elements": []}),
                ),
                (
                    "assets/ae2/models/block/drive/drive_base.json",
                    json!({"elements": []}),
                ),
            ],
        );
        let out = scratch.0.join("root");
        super::super::build(&base, &[ae2], &out).unwrap();
        let facts = read_json(&out, "facts.json").unwrap();
        let rows = facts["states"].as_array().unwrap();
        let model = read_json(&out, "model-states.json").unwrap();
        let render = read_json(&out, "render-definitions.json").unwrap();
        let command = read_json(&out, "command-registries.json").unwrap();
        let gameplay = read_json(&out, "gameplay-facts.json").unwrap();
        let structure = read_json(&out, "worldgen/blocks/structure-blocks.json").unwrap();
        for kind in [Definition::Charger, Definition::Drive] {
            let states: Vec<_> = rows
                .iter()
                .enumerate()
                .filter(|(_, row)| row[0] == kind.block())
                .collect();
            assert_eq!(states.len(), 24);
            for (i, (_, row)) in states.iter().enumerate() {
                assert_eq!(row[4]["properties"], kind.properties(i));
                assert_eq!(gameplay["states"][states[i].0]["block_entity"], true);
            }
            assert_eq!(facts["defaults"][kind.block()], states[0].0);
            assert_eq!(model[kind.block()]["schema"], kind.schema());
            assert_eq!(model[kind.block()]["states"].as_array().unwrap().len(), 24);
            assert_eq!(render[kind.block()]["default"], states[0].0);
            assert_eq!(render[kind.block()]["states"].as_array().unwrap().len(), 24);
            assert_eq!(command["blocks"][kind.block()], kind.commands());
            assert_eq!(structure["states"].as_u64().unwrap() as usize, rows.len());
            // AEBaseBlock does not override the vanilla identity hooks.
            // A wrench orientation strategy does not change structure hooks.
            let first = states[0].0;
            assert_eq!(
                structure["rotate"][first + 2],
                json!([first + 2, first + 2, first + 2])
            );
            assert_eq!(
                structure["mirror"][first + 2],
                json!([first + 2, first + 2])
            );
        }
        let (named, _) = nbt::read_named(
            &fs::read(out.join("occlusion.nbt")).unwrap(),
            Limits::tables(),
        )
        .unwrap();
        let Tag::Compound(occlusion) = named.tag else {
            panic!("no occlusion root");
        };
        let Some(Tag::List(states)) = occlusion.get(&JavaString::from("states")) else {
            panic!("no occlusion states");
        };
        assert_eq!(states.len(), rows.len());
        // The engine derives the cartesian state table from the facts rows;
        // a wrong order or a property mismatch fails this load.
        let assets = crate::assets::Assets::load(&out).unwrap();
        assert_eq!(assets.registry.len(), rows.len());
        let mut table = horizon_world::level::StateTable::new(&assets.registry).unwrap();
        let facing = table.key_of("facing").unwrap();
        let spin = table.key_of("spin").unwrap();
        for kind in [Definition::Charger, Definition::Drive] {
            let block = table.block_by_name(kind.block()).unwrap();
            let default = table.default_state(block);
            assert_eq!(table.name(default), kind.block());
            for i in 0..24 {
                // States of a block are contiguous from its first row.
                let state = horizon_world::registry::BlockStateId(default.0 + i as u32);
                assert_eq!(table.text(state, facing), ORIENTED_FACINGS[i / 4]);
                assert_eq!(table.int(state, spin), i as i32 % 4);
            }
            let east = table.with_text(default, facing, "east").unwrap();
            assert_eq!(table.int(east, spin), 0);
            let sheet = table.with_int(east, spin, 2).unwrap();
            assert_eq!(table.text(sheet, facing), "east");
            assert_eq!(table.int(sheet, spin), 2);
        }
        let mut sprites = horizon_client_render::sprites::Sprites::default();
        let models = horizon_client_render::models::catalog::StateModels::load(
            &out,
            assets.registry,
            &["ae2:charger", "ae2:drive"],
            &mut sprites,
        )
        .unwrap();
        for kind in [Definition::Charger, Definition::Drive] {
            let first = facts["defaults"][kind.block()].as_u64().unwrap() as u32;
            for i in 0..24 {
                assert!(
                    !models
                        .collect(
                            horizon_world::registry::BlockStateId(first + i),
                            horizon_core::position::BlockPos::new(0, 0, 0)
                        )
                        .unwrap()
                        .is_empty()
                );
            }
        }
    }
    #[test]
    fn inscriber_has_all_48_orientations_fluids_and_source_block_behavior() {
        let scratch = Scratch::new();
        let base = PathBuf::from(std::env::var_os("HORIZON_RESOURCES").unwrap());
        let kind = Definition::Inscriber;
        let variants = (0..48)
            .map(|i| {
                let p = kind.properties(i);
                (
                    format!(
                        "facing={},spin={},waterlogged={}",
                        p["facing"].as_str().unwrap(),
                        p["spin"].as_str().unwrap(),
                        p["waterlogged"].as_str().unwrap()
                    ),
                    json!({"model":"ae2:block/inscriber"}),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        let ae2 = content_assets(
            &scratch.0.join("ae2"),
            "ae2",
            &[("inscriber", json!({"template":"minecraft:dispenser"}))],
            &[
                (
                    "assets/ae2/blockstates/inscriber.json",
                    json!({"variants":variants}),
                ),
                (
                    "assets/ae2/models/block/inscriber.json",
                    json!({"elements":[]}),
                ),
            ],
        );
        let out = scratch.0.join("root");
        super::super::build(&base, &[ae2], &out).unwrap();
        let facts = read_json(&out, "facts.json").unwrap();
        let rows = facts["states"].as_array().unwrap();
        let first = facts["defaults"][kind.block()].as_u64().unwrap() as usize;
        assert_eq!(rows.iter().filter(|r| r[0] == kind.block()).count(), 48);
        let behavior = read_json(&out, "block-behaviours.json").unwrap();
        let water = behavior["fluids"]
            .as_array()
            .unwrap()
            .iter()
            .position(|f| f == "minecraft:water")
            .unwrap();
        let block = behavior["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["name"] == kind.block())
            .unwrap();
        assert_eq!(block["class"], json!(["Block"]));
        assert!(
            block["interfaces"]
                .as_array()
                .unwrap()
                .iter()
                .any(|i| i == "SimpleWaterloggedBlock")
        );
        let gameplay = read_json(&out, "gameplay-facts.json").unwrap();
        for i in 0..48 {
            assert_eq!(rows[first + i][4]["properties"], kind.properties(i));
            assert_eq!(rows[first + i][2], i % 2 == 1);
            assert_eq!(
                behavior["states"][first + i][2],
                if i % 2 == 1 { water } else { 0 }
            );
            assert_eq!(
                behavior["states"][first + i][3],
                if i % 2 == 1 { 8 } else { 0 }
            );
            assert_eq!(gameplay["states"][first + i]["dampening"], 2);
            assert_eq!(gameplay["states"][first + i]["can_occlude"], false);
        }
        let assets = crate::assets::Assets::load(&out).unwrap();
        let table = horizon_world::level::StateTable::new(assets.registry).unwrap();
        let block = table.block_by_name(kind.block()).unwrap();
        assert_eq!(table.default_state(block).0 as usize, first);
        let mut sprites = horizon_client_render::sprites::Sprites::default();
        let models = horizon_client_render::models::catalog::StateModels::load(
            &out,
            assets.registry,
            &[kind.block()],
            &mut sprites,
        )
        .unwrap();
        for i in 0..48 {
            assert!(
                !models
                    .collect(
                        horizon_world::registry::BlockStateId((first + i) as u32),
                        horizon_core::position::BlockPos::new(0, 0, 0)
                    )
                    .unwrap()
                    .is_empty()
            );
        }
    }
    #[test]
    fn facing_only_crank_gets_source_states_and_shape_boxes() {
        let scratch = Scratch::new();
        let base = PathBuf::from(std::env::var_os("HORIZON_RESOURCES").unwrap());
        let boxes = [
            ("north", [0.15, 0.15, 0.30, 0.85, 0.85, 1.00]),
            ("east", [0.0, 0.15, 0.15, 0.7, 0.85, 0.85]),
            ("south", [0.15, 0.15, 0.00, 0.85, 0.85, 0.70]),
            ("west", [0.30, 0.15, 0.15, 1.00, 0.85, 0.85]),
            ("up", [0.15, 0.0, 0.15, 0.85, 0.7, 0.85]),
            ("down", [0.15, 0.30, 0.15, 0.85, 1.00, 0.85]),
        ];
        let shapes: Value = json!({"property": "facing",
            "values": boxes.iter().map(|(face, b)| ((*face).to_owned(), json!([b]))).collect::<serde_json::Map<_, _>>()});
        let ae2 = content_assets(
            &scratch.0.join("ae2"),
            "ae2",
            &[(
                "crank",
                json!({"template": "minecraft:dispenser", "shapes": shapes}),
            )],
            &[
                (
                    "assets/ae2/blockstates/crank.json",
                    json!({"variants": {"": {"model": "ae2:block/crank"}}}),
                ),
                (
                    "assets/ae2/models/block/crank.json",
                    json!({"elements": []}),
                ),
            ],
        );
        let out = scratch.0.join("root");
        super::super::build(&base, &[ae2], &out).unwrap();
        let facts = read_json(&out, "facts.json").unwrap();
        let rows = facts["states"].as_array().unwrap();
        let model = read_json(&out, "model-states.json").unwrap();
        let render = read_json(&out, "render-definitions.json").unwrap();
        let command = read_json(&out, "command-registries.json").unwrap();
        let gameplay = read_json(&out, "gameplay-facts.json").unwrap();
        let structure = read_json(&out, "worldgen/blocks/structure-blocks.json").unwrap();
        let kind = Definition::Crank;
        let states: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row[0] == kind.block())
            .collect();
        assert_eq!(states.len(), 6);
        for (i, (_, row)) in states.iter().enumerate() {
            assert_eq!(row[4]["properties"], kind.properties(i));
            assert_eq!(gameplay["states"][states[i].0]["block_entity"], true);
        }
        let first = states[0].0;
        assert_eq!(facts["defaults"][kind.block()], json!(first));
        assert_eq!(model[kind.block()]["schema"], kind.schema());
        assert_eq!(model[kind.block()]["states"].as_array().unwrap().len(), 6);
        assert_eq!(render[kind.block()]["default"], json!(first));
        assert_eq!(render[kind.block()]["states"].as_array().unwrap().len(), 6);
        assert_eq!(command["blocks"][kind.block()], kind.commands());
        assert_eq!(structure["states"].as_u64().unwrap() as usize, rows.len());
        // Crank also inherits the vanilla identity structure hooks.
        assert_eq!(structure["rotate"][first], json!([first, first, first]));
        assert_eq!(structure["mirror"][first], json!([first, first]));
        let (named, _) = nbt::read_named(
            &fs::read(out.join("cached-collision.nbt")).unwrap(),
            Limits::tables(),
        )
        .unwrap();
        let Tag::Compound(collision) = named.tag else {
            panic!("no collision root");
        };
        let Some(Tag::IntArray(mapping)) = collision.get(&JavaString::from("cached_shapes")) else {
            panic!("no collision mapping");
        };
        let Some(Tag::List(entries)) = collision.get(&JavaString::from("shapes")) else {
            panic!("no collision shapes");
        };
        assert_eq!(mapping.len(), rows.len());
        let encode = |b: [f64; 6]| {
            let shape =
                horizon_core::geometry::VoxelShape::from_box(horizon_core::geometry::Aabb::new(
                    horizon_core::geometry::Vec3::new(b[0], b[1], b[2]),
                    horizon_core::geometry::Vec3::new(b[3], b[4], b[5]),
                ))
                .unwrap();
            super::super::encode_shape(&shape)
        };
        for (i, (face, b)) in boxes.iter().enumerate() {
            let at = mapping[first + i] as usize;
            assert_eq!(entries[at], encode(*b), "crank shape for {face}");
        }
        let assets = crate::assets::Assets::load(&out).unwrap();
        let table = horizon_world::level::StateTable::new(&assets.registry).unwrap();
        let facing = table.key_of("facing").unwrap();
        let block = table.block_by_name(kind.block()).unwrap();
        let default = table.default_state(block);
        for i in 0..6 {
            let state = horizon_world::registry::BlockStateId(default.0 + i as u32);
            assert_eq!(table.text(state, facing), ORIENTED_FACINGS[i]);
            assert!(table.value(state, facing).is_some());
        }
        assert_eq!(
            table.with_text(default, facing, "east").unwrap(),
            horizon_world::registry::BlockStateId(default.0 + 1)
        );
        let mut sprites = horizon_client_render::sprites::Sprites::default();
        let models = horizon_client_render::models::catalog::StateModels::load(
            &out,
            assets.registry,
            &[kind.block()],
            &mut sprites,
        )
        .unwrap();
        for i in 0..6 {
            assert!(
                !models
                    .collect(
                        horizon_world::registry::BlockStateId(default.0 + i as u32),
                        horizon_core::position::BlockPos::new(0, 0, 0)
                    )
                    .unwrap()
                    .is_empty()
            );
        }
    }
}
