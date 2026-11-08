#!/usr/bin/env python3
"""Prepare local AE2 content from the pinned jar; never install into player resources."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import zipfile
import upstream
import source_properties
import block_orientation
from item_stack_limits import UNSTACKABLE

ROOT = Path(__file__).resolve().parents[1]
LOCK = json.loads((ROOT / "upstream.lock.json").read_text())
MARKER = ".horizon-ae2-generated.json"
ORIENTED_FACINGS = ["north", "east", "south", "west", "up", "down"]
CRAFTING = ["network/blocks/crystal_processing_charger.json", "network/blocks/crank.json",
            "network/blocks/energy_energy_cell.json", "network/blocks/energy_dense_energy_cell.json",
            "network/blocks/storage_drive.json"]


def write_json(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n")


def ingredient(value):
    if isinstance(value, list):
        return [ingredient(v) for v in value]
    if isinstance(value, dict):
        if "item" in value:
            return value["item"]
        if "tag" in value:
            return "#" + value["tag"]
        raise ValueError(f"unsupported crafting ingredient: {value}")
    return value


def flatten_recipes(content):
    remap = {}
    for folder in sorted((content / "data").glob("*/recipe")):
        plans = []
        destinations = set()
        for path in sorted(folder.rglob("*.json")):
            rel = path.relative_to(folder)
            flat = ".".join(rel.parts)
            if flat in destinations:
                raise ValueError(f"recipe flattening collision: {folder / flat}")
            destinations.add(flat)
            plans.append((path, folder / flat))
            before = f"{folder.parent.name}:{rel.with_suffix('').as_posix()}"
            after = f"{folder.parent.name}:{Path(flat).stem}"
            remap[before] = after
        # Check all destinations before moving anything (a.b/x vs a/b.x).
        for original, target in plans:
            if original != target:
                original.rename(target)
    return remap


def merge_platform_tags(content, source):
    spec = LOCK["platform_tags"]
    hashes = {}; merged = []; counts = {}
    for namespace in spec["namespaces"]:
        base = source / spec["resources"] / "data" / namespace / "tags"
        for path in sorted(base.rglob("*.json")):
            relative = path.relative_to(source / spec["resources"])
            raw = path.read_bytes()
            data = json.loads(raw)
            if not isinstance(data.get("values"), list) or set(data) - {"values", "replace"}:
                raise ValueError(f"unsupported platform tag format: {path}")
            target = content / relative
            if target.exists():
                higher = json.loads(target.read_text())
                if not higher.get("replace", False):
                    higher["values"] = data["values"] + higher.get("values", [])
                data = higher
                merged.append(relative.as_posix())
            write_json(target, data)
            hashes[relative.as_posix()] = hashlib.sha256(raw).hexdigest()
            kind = relative.parts[3]
            counts[kind] = counts.get(kind, 0) + 1
    if not hashes:
        raise ValueError("pinned platform checkout contains no common tags")
    license_path = source / spec["license"]
    license_target = content / "data/ae2/licenses/neoforge-lgpl-2.1.txt"
    license_target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(license_path, license_target)
    report = {"schema": 1, "repository": spec["repository"], "revision": spec["revision"],
              "neoforge_version": spec["neoforge_version"], "tag_count": len(hashes),
              "by_registry": counts, "merged_with_ae2": merged, "source_sha256": hashes,
              "license_path": license_target.relative_to(content).as_posix(),
              "license_sha256": hashlib.sha256(license_path.read_bytes()).hexdigest(),
              "scope": "generated common c tags only; no NeoForge Java runtime or Minecraft tag overrides"}
    write_json(content / "data/ae2/platform-tags.json", report)
    return report


def fit(content, jar, platform):
    # Keep the original atlas extension in the mod's allowed namespace. The
    # native atlas adapter appends its sources to minecraft:blocks at load time.
    with zipfile.ZipFile(jar) as archive:
        atlas_name = "assets/minecraft/atlases/blocks.json"
        if atlas_name in archive.namelist():
            path = content / "assets/ae2/atlases/cable.json"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(archive.read(atlas_name))
    merge_platform_tags(content, platform)
    items = content / "data/ae2/horizon/item"
    missing = sorted(name for name in UNSTACKABLE if not (items / f"{name}.json").is_file())
    if missing:
        raise ValueError(f"carried AE2 item registrations missing: {missing}")
    # Template items are a resource-carry implementation detail. Their stack
    # sizes must never override AE2's registration properties (default 64).
    for path in sorted(items.glob("*.json")):
        data = json.loads(path.read_text())
        data["max_stack_size"] = 1 if path.stem in UNSTACKABLE else 64
        write_json(path, data)
    blocks = content / "data/ae2/horizon/block"
    # AE2 uses the same (height, inset), facing/waterlogged schema, default,
    # support and placement rules as the matching vanilla amethyst stage.
    # Keep the original AE2 models/loot tables, not the template's outputs.
    for name, template, light in [
        ("small_quartz_bud", "small_amethyst_bud", 1),
        ("medium_quartz_bud", "medium_amethyst_bud", 2),
        ("large_quartz_bud", "large_amethyst_bud", 4),
        ("quartz_cluster", "amethyst_cluster", 5),
    ]:
        write_json(blocks / f"{name}.json", {
            "template": f"minecraft:{template}",
            "properties": {"hardness": 1.5, "explosion_resistance": 1.5,
                           "requires_tool": True, "light": light},
        })
    # The native state adapter registers Controller's original state/type
    # product over this cube's physical rows. Keep its original assets intact.
    write_json(blocks / "controller.json", {
        "template": "minecraft:iron_block",
        "properties": {"hardness": 6, "explosion_resistance": 6, "requires_tool": False},
    })
    write_json(blocks / "cable_bus.json", {
        "template": "minecraft:glass", "item": False,
        "properties": {"light": 0, "hardness": 2.2, "explosion_resistance": 11,
                       "requires_tool": False, "map_color": 0, "sounds": "minecraft:glass"},
    })
    # CableBus quads are generated from its parts, never the glass template model.
    write_json(content / "assets/ae2/blockstates/cable_bus.json", {"variants":{"":{"model":"ae2:block/cable_bus_empty"}}})
    write_json(content / "assets/ae2/models/block/cable_bus_empty.json", {"elements":[]})
    states = content / "assets/ae2/blockstates"
    # A dispenser supplies six-facing placement and a block entity. Its menu
    # and inventory are intercepted; template redstone remains a parity gap.
    for name in ["charger", "crank", "drive"]:
        data = {
            "template": "minecraft:dispenser",
            "properties": {"light": 0, "hardness": 2.2},
        }
        if name == "crank":
            # CrankBlock.createShape: a 0.7-wide box shifted 0.15 away from
            # the attached side along the facing vector.
            def box(face):
                step = {"down": (0, -1, 0), "up": (0, 1, 0), "north": (0, 0, -1),
                        "south": (0, 0, 1), "west": (-1, 0, 0), "east": (1, 0, 0)}[face]
                off = [-0.15 * v for v in step]
                return [[off[0] + 0.15, off[1] + 0.15, off[2] + 0.15,
                         off[0] + 0.85, off[1] + 0.85, off[2] + 0.85]]
            data["shapes"] = {"property": "facing",
                              "values": {face: box(face) for face in ORIENTED_FACINGS}}
        write_json(blocks / f"{name}.json", data)
    for name in ["energy_cell", "dense_energy_cell", "creative_energy_cell"]:
        write_json(blocks / f"{name}.json", {
            "template": "minecraft:iron_block",
            "properties": {"light": 0, "hardness": 2.2, "explosion_resistance": 11,
                           "requires_tool": False, "map_color": 0, "sounds": "minecraft:glass"},
        })

    for name in ["energy_cell", "dense_energy_cell"]:
        # Source item ratios are fifths, not the placed block's quarters.
        write_json(content / f"assets/ae2/items/{name}.json", {"model": {
            "type": "minecraft:select", "property": "minecraft:block_state", "block_state_property": "fullness",
            "cases": [{"when": str(i), "model": {"type": "minecraft:model", "model": f"ae2:block/{name}_{i}"}} for i in range(5)],
            "fallback": {"type": "minecraft:model", "model": f"ae2:block/{name}_0"},
        }})

    for name in ["charger", "drive"]:
        original = json.loads((states / f"{name}.json").read_text())
        # Upstream's generated drive.json is empty: DriveBakedModel composes
        # the base, cell chassis and LEDs at runtime. Carry the actual base
        # housing now; dynamic cells/LEDs remain open.
        model = "ae2:block/drive/drive_base" if name == "drive" else None
        variants = block_orientation.oriented_variants(original, model)
        write_json(states / f"{name}.json", {"variants": variants})
    # CrankBlockEntityRenderer uses a separate handle model. Static preview only
    # for this milestone; the exact animation needs render/sync work.

    with zipfile.ZipFile(jar) as archive:
        # Native GUI atlases scan textures/gui, whereas AE2 styles use guis.
        # Copy the original bytes; the adapter only changes the resource route.
        gui = content / "assets/ae2/textures/gui/drive.png"
        gui.parent.mkdir(parents=True, exist_ok=True)
        gui.write_bytes(archive.read("assets/ae2/textures/guis/drive.png"))
        for name in CRAFTING:
            path = f"data/ae2/recipe/{name}"
            data = json.loads(archive.read(path))
            data["key"] = {k: ingredient(v) for k, v in data["key"].items()}
            write_json(content / path, data)

    remap = flatten_recipes(content)
    write_json(content / "ae2-recipe-map.json", remap)
    write_json(content / MARKER, {"schema": 2, "upstream": LOCK["revision"], "jar_sha512": LOCK["jar"]["sha512"],
                                 "platform_tags": LOCK["platform_tags"]["revision"]})
    return remap


def carry(jar, resources, port_pack, work):
    jars = work / "pack"
    jars.mkdir()
    shutil.copy2(jar, jars / jar.name)
    run = lambda *args: subprocess.run([str(port_pack), *map(str, args)], check=True)
    run("import", jars, work / "port")
    (work / "port/jars").mkdir(exist_ok=True)
    shutil.copy2(jar, work / "port/jars" / jar.name)
    run("inventory", work / "port", "--offline")
    run("carry", work / "port", "--resources", resources, "ae2")
    return work / "port/mods/ae2/carry"


def prepare(jar, carried, resources, port_pack, out, platform=None, java_home=None):
    if hashlib.sha512(jar.read_bytes()).hexdigest() != LOCK["jar"]["sha512"]:
        raise ValueError("jar differs from pinned AE2 19.2.18; do not mix source and content versions")
    if out.exists() and not (out / MARKER).is_file():
        raise ValueError(f"refusing to replace a directory not generated by this tool: {out}")
    platform = platform or ROOT / "target/upstream/neoforge"
    upstream.verify_platform(platform)
    out.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".ae2-content-", dir=out.parent) as temporary:
        work = Path(temporary)
        source = carried or carry(jar, resources, port_pack, work)
        stage = work / "ae2"
        shutil.copytree(source, stage)
        fit(stage, jar, platform)
        write_json(stage / "data/ae2/source-property-domains.json", source_properties.build(java_home))
        backup = work / "previous"
        if out.exists():
            out.rename(backup)
        try:
            stage.rename(out)
        except BaseException:
            if backup.exists():
                backup.rename(out)
            raise
    return out


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--carried", type=Path, help="reuse horizon-port-pack carry output")
    parser.add_argument("--resources", type=Path, help="prepared Horizon 26.3 resources")
    parser.add_argument("--port-pack", type=Path, default=Path(os.environ.get("HORIZON_PORT_PACK", "horizon-port-pack")))
    parser.add_argument("--out", type=Path, default=ROOT / "target/content/ae2")
    parser.add_argument("--platform", type=Path, default=ROOT / "target/upstream/neoforge", help="pinned NeoForge checkout for generated common tags")
    parser.add_argument("--java-home", type=Path, default=Path(os.environ["JAVA_HOME"]) if "JAVA_HOME" in os.environ else None, help="JDK 21 for original source property metadata")
    args = parser.parse_args()
    if not args.carried and not args.resources:
        parser.error("give --resources or --carried")
    print(prepare(args.jar.resolve(), args.carried.resolve() if args.carried else None,
                  args.resources, args.port_pack, args.out.resolve(), args.platform.resolve(), args.java_home))


if __name__ == "__main__":
    main()
