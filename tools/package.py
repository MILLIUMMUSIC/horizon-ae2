#!/usr/bin/env python3
"""Package the mod as a source-only `.hnmod`, the file Horizon's registry and launcher take.

The archive holds `horizon-native.json` and `mixins/<id>/` (only `.rs` and `.toml`), with fixed dates and
modes, so the same sources always give the same bytes and the same SHA-256. Horizon's
`horizon_mods::index::native` reads it without running anything; the launcher compiles it on the pinned
Horizon revision. Based on levzzz5154's packager for Veinminer.
"""
import hashlib
import io
import json
from pathlib import Path
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
GAME = '26.3'


def zip_bytes(files):
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    return buffer.getvalue()


def package():
    compat = json.loads((ROOT / 'compatibility.json').read_text())
    manifests = sorted((ROOT / 'mixins').glob('*/horizon-mixin.toml'))
    if len(manifests) != 1:
        raise SystemExit('a .hnmod holds one bundle: keep one mixins/<id>/horizon-mixin.toml')
    bundle = manifests[0].parent
    mixin = tomllib.loads(manifests[0].read_text())
    mod = mixin['mod']
    if bundle.name != mod['id']:
        raise SystemExit(f"rename mixins/{bundle.name}/ to mixins/{mod['id']}/ (the registry finds it by id)")
    if not mod['id'].replace('_', '').isalnum() or not mod['id'].islower():
        raise SystemExit('the registry takes ids of lowercase letters, digits and _ only')
    if mixin.get('compatibility', {}).get('horizon_revision') != compat['horizon_revision']:
        raise SystemExit('horizon-mixin.toml and compatibility.json pin different Horizon revisions')
    manifest = dict(schema=1, id=mod['id'], version=mod['version'], game=GAME,
                    horizon=compat['horizon_revision'], mixins=compat['horizon_mixins_revision'])
    files = {'horizon-native.json': (json.dumps(manifest, indent=2, sort_keys=True) + '\n').encode()}
    source_files = {}
    for path in sorted(bundle.iterdir()):
        if path.is_file() and path.suffix in ('.rs', '.toml'):
            files[path.relative_to(ROOT).as_posix()] = path.read_bytes()
            source_files[path.relative_to(ROOT).as_posix()] = path.read_bytes()
    # Preserve the licence and the complete corresponding port source/tooling.
    for name in ['LICENSE', 'NOTICE.md', 'README.md', 'Cargo.toml', 'Cargo.lock',
                 'rust-toolchain.toml', 'compatibility.json', 'upstream.lock.json',
                 'src/lib.rs', 'LICENSES/Fabric-Apache-2.0', 'tools/LICENSE-MIT', 'tools/horizon.py', 'tools/check.py', 'tools/check_docs.py',
                 'tools/test_tooling.py', 'tools/smoke.py', 'tools/smoke_power.py', 'tools/smoke_cable_world.py', 'tools/smoke_drive.py', 'tools/smoke_drive_gui.py', 'tools/smoke_drive_automation.py', 'tools/pathing_oracle.py',
                 'tools/energy_oracle.py', 'examples/energy_oracle.rs',
                 'tools/mounts_oracle.py', 'examples/mounts_oracle.rs',
                 'tools/cells_oracle.py', 'examples/cells_oracle.rs', 'docs/cells-oracle.json', 'docs/portable-world-smoke.json',
                 'examples/creative_oracle.rs', 'docs/creative-cell-oracle.json', 'docs/creative-world-smoke.json',
                 'tools/smoke_platform_tags.py', 'docs/platform-tags-report.json', 'docs/platform-tags-world-smoke.json',
                 'docs/tool-cell-oracle.json', 'docs/tool-world-smoke.json', 'docs/upgrade-world-smoke.json',
                 'tools/item_energy_oracle.py', 'examples/item_energy_oracle.rs', 'docs/item-energy-oracle.json', 'tools/smoke_item_charging.py',
                 'tools/charger_oracle.py', 'examples/charger_oracle.rs', 'docs/charger-oracle.json', 'docs/item-charging-world-smoke.json',
                 'tools/smoke_charger_automation.py', 'docs/charger-automation-world-smoke.json', 'docs/charger-automation-drive-regression.json',
                 'tools/smoke_item_bars.py', 'tools/smoke_charger_inventory.py', 'tools/smoke_drive_inventory.py', 'docs/drive-inventory-world-smoke.json', 'docs/shared-codec-charger-regression.json', 'docs/shared-codec-gui-regression.json', 'docs/charger-inventory-world-smoke.json', 'docs/charger-inventory-charging-regression.json', 'docs/item-energy-view-oracle.json', 'docs/item-bars-world-smoke.json', 'docs/component-sync-drive-gui.json', 'docs/component-sync-charging-world.json',
                 'examples/grid_oracle.rs', 'AGENTS.md', 'LICENSES/AE2-API-MIT',
                 'tools/upstream.py', 'tools/prepare_content.py', 'tools/package.py', 'tools/item_stack_limits.py', 'docs/item-stack-source-audit.json',
                 'tools/smoke_item_stacks.py', 'docs/item-stack-world-smoke.json', 'docs/item-stack-charger-regression.json', 'docs/item-stack-drive-gui-regression.json',
                 'tools/smoke_inventory_signal.py', 'docs/inventory-signal-source-audit.json', 'docs/inventory-signal-world-smoke.json',
                 'tools/energy_cell_item_oracle.py', 'examples/energy_cell_item_oracle.rs', 'docs/energy-cell-item-oracle.json',
                 'tools/smoke_energy_cell_item.py', 'docs/energy-cell-item-world-smoke.json', 'docs/energy-cell-item-base-regression.json', 'docs/energy-cell-item-charging-regression.json',
                 'tools/energy_cell_oracle.py', 'examples/energy_cell_oracle.rs', 'docs/energy-cell-visual-oracle.json',
                 'tools/smoke_energy_cell_display.py', 'docs/energy-cell-world-smoke.json', 'docs/energy-cell-charging-regression.json',
                 'tools/node_data_oracle.py', 'examples/node_data_oracle.rs', 'docs/node-data-oracle.json', 'docs/cable-bus-nbt-power-regression.json',
                 'tools/cable_render_oracle.py', 'examples/cable_render_oracle.rs', 'docs/cable-render-oracle.json', 'docs/cable-world-smoke.json',
                 'tools/part_placement_oracle.py', 'examples/part_placement_oracle.rs', 'docs/part-placement-oracle.json', 'tools/smoke_cable_items.py', 'docs/cable-items-world-smoke.json',
                 'tools/ctrl_oracle.py', 'examples/ctrl_oracle.rs', 'docs/ctrl-oracle.json', 'tools/smoke_cable_input.py', 'docs/cable-input-world-smoke.json',
                 'tools/smoke_adventure.py', 'docs/adventure-world-smoke.json',
                 'tools/block_orientation.py', 'tools/orientation_oracle.py', 'examples/orientation_oracle.rs', 'docs/orientation-oracle.json',
                 'tools/smoke_quartz.py', 'docs/quartz-world-smoke.json',
                 'tools/inscriber_oracle.py', 'examples/inscriber_oracle.rs', 'docs/inscriber-oracle.json', 'tools/smoke_controller_craft.py', 'docs/controller-craft-world-smoke.json', 'tools/smoke_inscriber.py', 'docs/inscriber-world-smoke.json',
                 'tools/multipart_oracle.py', 'examples/multipart_oracle.rs', 'docs/multipart-oracle.json', 'docs/multipart-cable-regression.json', 'docs/multipart-power-regression.json',
                 'tools/part_geometry_oracle.py', 'examples/part_geometry_oracle.rs', 'docs/part-geometry-oracle.json', 'docs/part-geometry-world-smoke.json', 'docs/part-geometry-placement-world-smoke.json',
                 'tools/reporting_oracle.py', 'examples/reporting_oracle.rs', 'docs/reporting-oracle.json', 'docs/reporting-cable-regression.json',
                 'tools/shared_power_oracle.py', 'examples/shared_power_oracle.rs', 'docs/shared-power-oracle.json', 'docs/shared-power-cable-regression.json',
                 'tools/me_chest_oracle.py', 'examples/me_chest_oracle.rs', 'examples/support/me_chest_fixture.rs', 'docs/me-chest-oracle.json',
                 'tools/smoke_me_chest.py', 'docs/me-chest-native.json', 'docs/me-chest-cable-regression.json',
                 'tools/inventory_update_oracle.py', 'examples/inventory_update_oracle.rs', 'docs/inventory-update-oracle.json',
                 'tools/storage_helper_oracle.py', 'examples/storage_helper_oracle.rs', 'docs/storage-helper-oracle.json',
                 'tools/incremental_update_oracle.py', 'examples/incremental_update_oracle.rs', 'docs/incremental-update-oracle.json',
                 'tools/me_interaction_oracle.py', 'examples/me_interaction_oracle.rs', 'docs/me-interaction-oracle.json',
                 'tools/storage_menu_oracle.py', 'examples/storage_menu_oracle.rs', 'examples/support/storage_menu_fixture.rs', 'docs/storage-menu-oracle.json',
                 'tools/meteorite_oracle.py', 'examples/meteorite_oracle.rs', 'docs/meteorite-oracle.json',
                 'tools/tag_entry_reference.py', 'docs/tag-entry-reference.json', 'tools/smoke_meteorite_fallout.py', 'docs/meteorite-native.json',
                 'LICENSES/JOML-MIT', 'tools/quad_rotation_oracle.py', 'examples/quad_rotation_oracle.rs', 'docs/quad-rotation-oracle.json',
                 'LICENSES/Horizon-MIT', 'tools/item_tick_source.py', 'tools/transform_oracle.py', 'examples/transform_oracle.rs', 'docs/transform-oracle.json', 'tools/smoke_fluix_throw.py', 'docs/fluix-world-smoke.json', 'docs/fluid-inscriber-regression.json', 'docs/item-tick-source-audit.json',
                 'tools/smoke_cell_1k.py', 'docs/cell-1k-world-smoke.json', 'tools/smoke_charge_certus.py', 'docs/charge-certus-world-smoke.json',
                 'tools/smoke_charger_orientation.py', 'tools/smoke_wrench_rotation.py', 'tools/smoke_crank_states.py',
                 'docs/charger-orientation-world-smoke.json', 'docs/wrench-rotation-world-smoke.json', 'docs/crank-states-world-smoke.json',
                 'tools/adventure_oracle.py', 'examples/adventure_oracle.rs', 'docs/adventure-oracle.json',
                 'tools/minecraft_reference.py', 'tools/source_properties.py', 'tools/java/MinecraftAdventureReference.java',
                 'tools/part_use_oracle.py', 'examples/part_use_oracle.rs', 'docs/part-use-oracle.json', 'tools/smoke_cable_actions.py', 'docs/cable-actions-world-smoke.json',
                 'tools/cable_oracle.py', 'examples/cable_oracle.rs', 'docs/cable-oracle.json', 'docs/cable-controller-regression.json',
                 'tools/controller_oracle.py', 'examples/controller_oracle.rs', 'docs/controller-visual-oracle.json',
                 'tools/smoke_controller.py', 'docs/controller-world-smoke.json', 'docs/controller-charging-regression.json', 'docs/controller-drive-gui-regression.json',
                 'docs/PARITY.md', 'docs/PLAN.md', 'docs/upstream-inventory.json',
                 'docs/baseline-port-pack.json', 'docs/pathing-oracle.json',
                 'docs/energy-oracle.json', 'docs/energy-overlay-oracle.json',
                 'docs/mounts-oracle.json', 'docs/drive-world-smoke.json', 'docs/drive-gui-smoke.json', 'docs/drive-automation-smoke.json', 'docs/PROGRESS.md', 'VALIDATION.md']:
        source_files[name] = (ROOT / name).read_bytes()
    # The loader accepts at most 512 entries. Keep all corresponding source,
    # tools and evidence in one deterministic archive under the allowed data/.
    files[f'mixins/{mod["id"]}/data/source.zip'] = zip_bytes(source_files)
    for name in ['LICENSE', 'NOTICE.md']:
        files[f'mixins/{mod["id"]}/data/{name}'] = (ROOT / name).read_bytes()
    if len(files) > 512:
        raise SystemExit("native payload exceeds Horizon's 512-entry limit")
    out = ROOT / 'target' / f"{mod['id']}-{mod['version']}.hnmod"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(zip_bytes(files))
    print(json.dumps(dict(file=str(out), size=out.stat().st_size,
                          sha256=hashlib.sha256(out.read_bytes()).hexdigest()), indent=2))


if __name__ == '__main__':
    package()
