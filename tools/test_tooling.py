#!/usr/bin/env python3
"""Regression tests for resource conversion, publication boundaries and packaging."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib
import unittest
from unittest.mock import patch
import zipfile

import package
import block_orientation
import prepare_content as content


class ContentTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def fixture(self):
        self.platform = self.root / "platform"
        base = self.platform / content.LOCK["platform_tags"]["resources"]
        content.write_json(base / "data/c/tags/item/ingots/iron.json", {"values": [
            "minecraft:iron_ingot", {"id": "#forge:ingots/iron", "required": False}
        ]})
        content.write_json(base / "data/c/tags/item/nuggets/iron.json", {"values": ["minecraft:iron_nugget"]})
        (self.platform / content.LOCK["platform_tags"]["license"]).write_text("platform licence fixture")
        carried = self.root / "carry"
        for name in content.UNSTACKABLE | {"certus_quartz_crystal", "blank_pattern"}:
            content.write_json(carried / f"data/ae2/horizon/item/{name}.json", {"template": "minecraft:stick"})
        def orientation_states(model):
            variants = {}
            for ordinal, face in enumerate(block_orientation.FACING_ORDER):
                for spin in range(4):
                    ax, ay, az = block_orientation.ANGLES[ordinal * 4 + spin]
                    variant = {"model": model}
                    if ax:
                        variant["x"] = ax
                    if ay:
                        variant["y"] = ay
                    if az:
                        variant["ae2:z"] = az
                    variants[f"facing={face},spin={spin}"] = variant
            return {"variants": variants}
        content.write_json(carried / "assets/ae2/blockstates/charger.json", orientation_states("ae2:block/charger"))
        content.write_json(carried / "assets/ae2/blockstates/drive.json", orientation_states("ae2:block/drive"))
        content.write_json(carried / "assets/ae2/blockstates/inscriber.json", orientation_states("ae2:block/inscriber"))
        for name in ("energy_cell", "dense_energy_cell"):
            content.write_json(carried / f"assets/ae2/blockstates/{name}.json", {
                "variants": {f"fullness={level}": {"model": f"ae2:block/{name}_{level}"} for level in range(5)}
            })
        content.write_json(carried / "data/ae2/recipe/charger/a.json", {
            "type": "ae2:charger", "ingredient": {"item": "ae2:certus_quartz_crystal"},
            "result": {"id": "ae2:charged_certus_quartz_crystal"}
        })
        content.write_json(carried / "data/c/tags/item/ingots/iron.json", {
            "replace": False, "values": [{"id": "other:iron", "required": False}]
        })
        jar = self.root / "test.jar"
        with zipfile.ZipFile(jar, "w") as z:
            z.writestr("assets/ae2/textures/guis/drive.png", b"original-gui-fixture")
            for name in content.CRAFTING:
                z.writestr("data/ae2/recipe/" + name, json.dumps({
                    "type": "minecraft:crafting_shaped", "key": {"a": {"tag": "c:ingots/iron"}},
                    "pattern": ["a"], "result": {"id": "ae2:charger", "count": 1}
                }))
        return carried, jar

    def test_flatten_refuses_collision_before_mutating_files(self):
        first = self.root / "data/ae2/recipe/a/b.json"
        second = self.root / "data/ae2/recipe/a.b.json"
        content.write_json(first, {"first": True})
        content.write_json(second, {"second": True})
        with self.assertRaisesRegex(ValueError, "collision"):
            content.flatten_recipes(self.root)
        self.assertTrue(first.exists())
        self.assertTrue(second.exists())

    def test_custom_recipes_kept_and_crafting_normalized_with_id_map(self):
        carried, jar = self.fixture()
        remap = content.fit(carried, jar, self.platform)
        self.assertEqual(remap["ae2:charger/a"], "ae2:charger.a")
        data = json.loads((carried / "data/ae2/recipe/charger.a.json").read_text())
        self.assertEqual(data["ingredient"], {"item": "ae2:certus_quartz_crystal"})
        restored = json.loads((carried / "data/ae2/recipe/network.blocks.crank.json").read_text())
        self.assertEqual(restored["key"]["a"], "#c:ingots/iron")
        tag = json.loads((carried / "data/c/tags/item/ingots/iron.json").read_text())
        self.assertEqual(tag["values"], ["minecraft:iron_ingot", {"id": "#forge:ingots/iron", "required": False}, {"id": "other:iron", "required": False}])
        self.assertFalse(tag["replace"])

    def test_item_stack_limits_keep_existing_properties_and_default_stackables(self):
        carried, jar = self.fixture()
        path = carried / "data/ae2/horizon/item/portable_item_cell_1k.json"
        original = {"template": "minecraft:paper", "rarity": "rare", "max_stack_size": 64,
                    "components": {"minecraft:custom_data": {"kept": True}}}
        content.write_json(path, original)
        content.fit(carried, jar, self.platform)
        data = json.loads(path.read_text())
        self.assertEqual(data, {**original, "max_stack_size": 1})
        for name in ("crafting_pattern", "wrapped_generic_stack", "certus_quartz_axe", "spatial_storage_cell_128"):
            self.assertEqual(json.loads((path.parent / f"{name}.json").read_text())["max_stack_size"], 1)
        for name in ("blank_pattern", "certus_quartz_crystal"):
            self.assertEqual(json.loads((path.parent / f"{name}.json").read_text())["max_stack_size"], 64)

    def test_native_hosts_have_block_entities_and_full_orientation_models(self):
        carried, jar = self.fixture()
        content.fit(carried, jar, self.platform)
        controller = json.loads((carried / "data/ae2/horizon/block/controller.json").read_text())
        self.assertEqual(controller["template"], "minecraft:iron_block")
        self.assertEqual(controller["properties"], {"hardness": 6, "explosion_resistance": 6, "requires_tool": False})
        for name in ["charger", "crank", "drive"]:
            block = json.loads((carried / f"data/ae2/horizon/block/{name}.json").read_text())
            self.assertEqual(block["template"], "minecraft:dispenser")
        inscriber = json.loads((carried / "data/ae2/horizon/block/inscriber.json").read_text())
        self.assertEqual(inscriber["properties"]["explosion_resistance"], 11)
        self.assertFalse(inscriber["properties"]["requires_tool"])
        variants = json.loads((carried / "assets/ae2/blockstates/inscriber.json").read_text())["variants"]
        self.assertEqual(len(variants), 48)
        for key in [f"facing={face},spin={spin}" for face in block_orientation.FACING_ORDER for spin in range(4)]:
            self.assertEqual(variants[key + ",waterlogged=false"], variants[key + ",waterlogged=true"])
        crank = json.loads((carried / "data/ae2/horizon/block/crank.json").read_text())
        self.assertEqual(crank["shapes"]["property"], "facing")
        self.assertEqual(crank["shapes"]["values"], {
            "north": [[0.15, 0.15, 0.3, 0.85, 0.85, 1.0]],
            "east": [[0.0, 0.15, 0.15, 0.7, 0.85, 0.85]],
            "south": [[0.15, 0.15, 0.0, 0.85, 0.85, 0.7]],
            "west": [[0.3, 0.15, 0.15, 1.0, 0.85, 0.85]],
            "up": [[0.15, 0.0, 0.15, 0.85, 0.7, 0.85]],
            "down": [[0.15, 0.3, 0.15, 0.85, 1.0, 0.85]],
        })
        for name in ["charger", "drive"]:
            block = json.loads((carried / f"data/ae2/horizon/block/{name}.json").read_text())
            self.assertNotIn("shapes", block)
        for name in ["energy_cell", "dense_energy_cell", "creative_energy_cell"]:
            block = json.loads((carried / f"data/ae2/horizon/block/{name}.json").read_text())
            self.assertEqual(block["template"], "minecraft:iron_block")
            self.assertEqual(block["properties"], {"light": 0, "hardness": 2.2, "explosion_resistance": 11,
                                                 "requires_tool": False, "map_color": 0, "sounds": "minecraft:glass"})
        for name in ("energy_cell", "dense_energy_cell"):
            original = json.loads((carried / f"assets/ae2/blockstates/{name}.json").read_text())
            self.assertEqual(original, {"variants": {f"fullness={level}": {"model": f"ae2:block/{name}_{level}"} for level in range(5)}})
        for name in ("energy_cell", "dense_energy_cell"):
            item = json.loads((carried / f"assets/ae2/items/{name}.json").read_text())["model"]
            self.assertEqual(item["property"], "minecraft:block_state")
            self.assertEqual(item["block_state_property"], "fullness")
            self.assertEqual([(entry["when"], entry["model"]["model"]) for entry in item["cases"]],
                             [(str(i), f"ae2:block/{name}_{i}") for i in range(5)])
        states = json.loads((carried / "assets/ae2/blockstates/charger.json").read_text())
        keys = {f"facing={face},spin={spin}" for face in block_orientation.FACING_ORDER for spin in range(4)}
        self.assertEqual(set(states["variants"]), keys)
        # The source x/y/ae2:z keys become the equivalent Horizon triple:
        # UP_NORTH is x=90, z=180 in Horizon's composition order.
        self.assertEqual(states["variants"]["facing=up,spin=0"],
                         {"model": "ae2:block/charger", "x": 90, "z": 180})
        self.assertNotIn("ae2:z", states["variants"]["facing=up,spin=0"])
        drive = json.loads((carried / "assets/ae2/blockstates/drive.json").read_text())
        self.assertEqual(set(drive["variants"]), keys)
        self.assertTrue(all(v["model"] == "ae2:block/drive/drive_base" for v in drive["variants"].values()))
        self.assertEqual((carried / "assets/ae2/textures/gui/drive.png").read_bytes(), b"original-gui-fixture")

    def test_wrong_jar_is_rejected_without_touching_output(self):
        carried, jar = self.fixture()
        out = self.root / "out"
        out.mkdir()
        (out / "sentinel").write_text("keep")
        with self.assertRaisesRegex(ValueError, "jar differs"):
            content.prepare(jar, carried, None, None, out)
        self.assertEqual((out / "sentinel").read_text(), "keep")

    def test_unowned_output_is_never_replaced(self):
        carried, jar = self.fixture()
        out = self.root / "out"
        out.mkdir()
        lock = {**content.LOCK, "jar": {"sha512": hashlib.sha512(jar.read_bytes()).hexdigest()}}
        with patch.object(content, "LOCK", lock), self.assertRaisesRegex(ValueError, "not generated"):
            content.prepare(jar, carried, None, None, out)

    def test_failed_conversion_does_not_replace_previous_generation(self):
        carried, jar = self.fixture()
        out = self.root / "out"
        out.mkdir()
        (out / content.MARKER).write_text("{}")
        (out / "sentinel").write_text("keep")
        (carried / "assets/ae2/blockstates/charger.json").write_text('{"variants":{}}')
        lock = {**content.LOCK, "jar": {"sha512": hashlib.sha512(jar.read_bytes()).hexdigest()}}
        with patch.object(content, "LOCK", lock), patch.object(content.upstream, "verify_platform"), self.assertRaisesRegex(ValueError, "orientation variants"):
            content.prepare(jar, carried, None, None, out, self.platform)
        self.assertEqual((out / "sentinel").read_text(), "keep")

    def test_property_facts_are_written_only_to_generated_mod_and_failure_keeps_previous(self):
        carried, jar = self.fixture()
        out = self.root / "out"
        lock = {**content.LOCK, "jar": {"sha512": hashlib.sha512(jar.read_bytes()).hexdigest()}}
        facts = {"schema": 1, "blocks": {"minecraft:stone": {}}}
        with patch.object(content, "LOCK", lock), patch.object(content.upstream, "verify_platform"), patch.object(content.source_properties, "build", return_value=facts):
            content.prepare(jar, carried, None, None, out, self.platform)
        self.assertEqual(json.loads((out / "data/ae2/source-property-domains.json").read_text()), facts)
        self.assertFalse((carried / "data/ae2/source-property-domains.json").exists())
        (out / "sentinel").write_text("keep")
        with patch.object(content, "LOCK", lock), patch.object(content.upstream, "verify_platform"), patch.object(content.source_properties, "build", side_effect=ValueError("bad reference")), self.assertRaisesRegex(ValueError, "bad reference"):
            content.prepare(jar, carried, None, None, out, self.platform)
        self.assertEqual((out / "sentinel").read_text(), "keep")

    def test_platform_tags_layer_before_mod_tags_and_preserve_replace_and_provenance(self):
        carried, _ = self.fixture()
        report = content.merge_platform_tags(carried, self.platform)
        nugget = carried / "data/c/tags/item/nuggets/iron.json"
        self.assertEqual(json.loads(nugget.read_text())["values"], ["minecraft:iron_nugget"])
        self.assertEqual(report["tag_count"], 2)
        self.assertEqual(report["by_registry"], {"item": 2})
        self.assertEqual(report["merged_with_ae2"], ["data/c/tags/item/ingots/iron.json"])
        self.assertEqual((carried / "data/ae2/licenses/neoforge-lgpl-2.1.txt").read_text(), "platform licence fixture")
        self.assertEqual(len(report["source_sha256"]), 2)
        # AE2's explicit replacement drops the lower NeoForge layer.
        content.write_json(carried / "data/c/tags/item/ingots/iron.json", {"replace": True, "values": ["other:replacement"]})
        content.merge_platform_tags(carried, self.platform)
        self.assertEqual(json.loads((carried / "data/c/tags/item/ingots/iron.json").read_text()), {"replace": True, "values": ["other:replacement"]})

    def test_changed_platform_checkout_does_not_replace_generated_output(self):
        carried, jar = self.fixture()
        git = lambda *args: subprocess.run(["git", "-C", str(self.platform), *args], check=True, capture_output=True)
        git("init", "-q"); git("add", ".")
        git("-c", "user.name=AE2 tooling test", "-c", "user.email=test@example.invalid", "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "commit", "-qm", "fixture")
        revision = git("rev-parse", "HEAD").stdout.decode().strip()
        lock = {**content.LOCK, "jar": {"sha512": hashlib.sha512(jar.read_bytes()).hexdigest()},
                "platform_tags": {**content.LOCK["platform_tags"], "revision": revision}}
        out = self.root / "out"; out.mkdir()
        (out / content.MARKER).write_text("{}"); (out / "sentinel").write_text("keep")
        path = self.platform / lock["platform_tags"]["resources"] / "data/c/tags/item/nuggets/iron.json"
        path.write_text('{"values":[]}')
        with patch.object(content, "LOCK", lock), patch.object(content.upstream, "LOCK", lock), self.assertRaisesRegex(ValueError, "local changes"):
            content.prepare(jar, carried, None, None, out, self.platform)
        self.assertEqual((out / "sentinel").read_text(), "keep")


class PackageTests(unittest.TestCase):
    def test_source_bundle_is_deterministic_licensed_and_within_horizon_path_contract(self):
        with contextlib.redirect_stdout(io.StringIO()):
            package.package()
            out = package.ROOT / "target/ae2-0.1.0.hnmod"
            first = out.read_bytes()
            package.package()
        self.assertEqual(first, out.read_bytes())
        with zipfile.ZipFile(out) as z:
            manifest = json.loads(z.read("horizon-native.json"))
            self.assertEqual(manifest["id"], "ae2")
            prefix = "mixins/ae2/"
            for name in z.namelist():
                if name == "horizon-native.json":
                    continue
                # Exact admissible path contract in Horizon index/native.rs.
                self.assertTrue(name.startswith(prefix))
                self.assertTrue(name.endswith((".rs", ".toml")) or name.startswith(prefix + "data/"))
                self.assertNotIn("..", name.split("/"))
                self.assertFalse(name.endswith((".png", ".jar", ".class")))
            self.assertIn(b"LESSER GENERAL PUBLIC LICENSE", z.read(prefix + "data/source/LICENSE"))
            self.assertIn(b"MIT License", z.read(prefix + "data/source/tools/LICENSE-MIT"))
            self.assertIn(prefix + "data/source/src/lib.rs", z.namelist())
            # Corresponding source must be runnable as its own Cargo tree.
            self.assertEqual(z.read(prefix + "data/source/mixins/ae2/charger.rs"),
                             z.read(prefix + "charger.rs"))

    def test_corresponding_source_includes_content_preparation_imports(self):
        with contextlib.redirect_stdout(io.StringIO()):
            package.package()
        with tempfile.TemporaryDirectory() as temporary:
            with zipfile.ZipFile(package.ROOT / "target/ae2-0.1.0.hnmod") as archive:
                archive.extractall(temporary)
            source = Path(temporary) / "mixins/ae2/data/source"
            run = subprocess.run([sys.executable, "-I", "-c",
                "import sys; sys.path.insert(0, sys.argv[1]); import prepare_content; import block_orientation; assert len(block_orientation.ANGLES)==24",
                str(source / "tools")], cwd=source, text=True, capture_output=True)
            self.assertEqual(run.returncode, 0, run.stderr)

    def test_bundle_pins_match_and_all_injections_have_helpers(self):
        root = package.ROOT
        mixin = tomllib.loads((root / "mixins/ae2/horizon-mixin.toml").read_text())
        compat = json.loads((root / "compatibility.json").read_text())
        self.assertEqual(mixin["compatibility"]["horizon_revision"], compat["horizon_revision"])
        for declaration in mixin["module"]:
            self.assertTrue((root / "mixins/ae2" / declaration["source"]).is_file())
        for injection in mixin["injection"]:
            self.assertTrue((root / "mixins/ae2" / injection["helper"]).is_file())


class BlockOrientationTests(unittest.TestCase):
    def test_every_source_orientation_has_the_same_horizon_transform(self):
        seen = set()
        for ordinal, face in enumerate(block_orientation.FACING_ORDER):
            for spin in range(4):
                angle = block_orientation.ANGLES[ordinal * 4 + spin]
                turns = block_orientation.horizon_turns(face, spin)
                self.assertEqual(block_orientation.horizon_matrix(turns),
                                 block_orientation.source_matrix(*angle),
                                 f"{face} spin {spin}")
                seen.add(turns)
        self.assertEqual(len(seen), 24)

    def test_source_angle_mismatch_is_refused(self):
        with self.assertRaisesRegex(ValueError, "differ"):
            block_orientation.oriented_variants({"variants": {
                "facing=north,spin=0": {"model": "ae2:block/charger", "x": 90}}})

class ItemTickSourceTests(unittest.TestCase):
    def test_carried_native_tick_rejects_unmarked_behavior_changes(self):
        import item_tick_source
        root = Path(__file__).resolve().parents[1]
        source = root / 'target/upstream/horizon/crates/horizon-client/src/game.rs'
        if not source.is_file():
            self.skipTest('fetch the pinned Horizon source to run the carried-body audit')
        payload = (root / 'mixins/ae2/item_tick.rs').read_text()
        self.assertTrue(item_tick_source.audit(source.read_text(), payload)['passed'])
        changed = payload.replace('item.pickup_delay == 0', 'item.pickup_delay <= 10')
        self.assertNotEqual(changed, payload)
        with self.assertRaisesRegex(ValueError, 'differs outside'):
            item_tick_source.audit(source.read_text(), changed)
        changed_literal = payload.replace('"minecraft:entity.item.pickup"', '"minecraft:entity.item.pickup "')
        self.assertNotEqual(changed_literal, payload)
        with self.assertRaisesRegex(ValueError, 'differs outside'):
            item_tick_source.audit(source.read_text(), changed_literal)


if __name__ == "__main__":
    unittest.main(verbosity=2)
