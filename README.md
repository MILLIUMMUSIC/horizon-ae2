# Applied Energistics 2 for Horizon

[Applied Energistics 2](https://github.com/AppliedEnergistics/Applied-Energistics-2) (AE2 19.2.18 for
Minecraft 1.21.1) ported to [Horizon](https://github.com/horizon-rust/horizon), Minecraft: Java Edition
rewritten in Rust. Horizon runs no Java, so AE2's code is ported to Rust and built into the game as a
**native mod** (a [horizon-mixins](https://github.com/horizon-rust/horizon-mixins) bundle). Its blocks,
items, recipes, models and textures are prepared from **your own AE2 jar** as a content mod.

Status: **ME network and storage foundations, Charger, energy cells, Drive, Controller, Inscriber processing, and cable
centers** are ported in part and checked in isolated game worlds. Cable items support placement,
survival drops, names, waterlogging/buckets and persistent ownership. The current input layer adds
the configurable opposite-side key, synchronized server state and center-part Pick Block. Center
placement also has client prediction, source interaction results and server correction.
Charger and Drive use the source 24-state facing/spin orientation, including source player
placement thresholds and source identity structure hooks. Adventure item use evaluates the actual
hand and original block/state/NBT predicates.
Adventure-mode rework is suspended for now: that validated behavior stays in place, while its
remaining cache, block-entity codec and permission work is deferred.
The target is the **complete 1:1 port** of the pinned AE2 release; the full mod is still in development.
See [docs/PLAN.md](docs/PLAN.md) for the scope and next steps, [docs/PARITY.md](docs/PARITY.md) for the
remaining behavior, and [VALIDATION.md](VALIDATION.md) for what was checked and how.

## Progress

| Milestone | Current coverage | Still to port |
|---|---|---|
| 1. Toolchain and delivery | Pinned sources, original content preparation, native builds and deterministic packages | Launcher installation/rebuild verification and release workflow |
| 2. Keys, cells and persistence | Component-aware item/fluid keys; all 23 built-in basic, portable, creative and tool cell profiles; Drive backends | Complete registry recovery, Cell Workbench, config writers and upgrade lifecycle |
| 3. ME networks and cables | Topology, channels, energy routing, Controller validation, five cable types and 17 colors, native cable centers, item placement/drop and waterlogging | Complete part interactions, side parts/facades, full source events, collision and lighting paths |
| 4. Energy and machines | Charger recipes and powered items, finite/creative energy cells, Controller power, source Charger/Drive orientation, Inscriber recipes and automation | Energy Acceptor, Vibration Chamber, complete machine menus/rendering, external capabilities and lifecycle |
| 5. Storage and terminals | Ten-slot Drive, cell mounting, priorities in the storage core, filtered screen and automation | Terminals, ME Chest, storage buses, priority UI and complete storage interactions |
| 6. Autocrafting | Pending | Patterns, providers, assemblers, crafting CPUs, planning and job lifecycle |
| 7. Other network services | Pending | P2P, quantum links, wireless, security, spatial I/O and buses |
| 8. Items, world content and integration | Item energy profiles/bars, finite cell item models and placement/drop energy, original resources and common tags | Complete tools, handheld menus, world generation, effects, guidebook and addon APIs |
| 9. Full validation | Rule tests, Java differential comparisons, native tests and selected game scenarios | Complete survival progression, source-game comparisons, multiplayer and large-network validation |

These are feature checkpoints, not completed milestones. The source inventory tracks all 1,429 Java
files, including 123 partially ported classes. That count is not a gameplay completion percentage.
Blocks, items and recipes carried over as data do not count as working features on their own.

## What is ported

| AE2 | Here |
|---|---|
| `AEItemKey`, `AEFluidKey`, counters and fuzzy matching | `keys.rs`, `cells.rs`: component-aware identity, signed counts, source damage intervals and saved cell contents |
| Basic, portable, creative, Matter Cannon and Color Applicator cell inventories | `storage.rs`, `backend.rs`, `tool_filters.rs`: capacity, byte/type limits, partitions, simulation, insertion/extraction and restricted tool storage |
| Grid nodes, connections, pathing and channel finalization | `grid.rs`, `network.rs`: topology, ad-hoc/controller networks, normal/dense channel limits and colored connections |
| Energy storage and grid energy service | `energy.rs`, `power.rs`: finite/creative stores, provider priorities, local buffers, drain and merge/split rules |
| Storage providers, mounts and network storage service | `mounts.rs`, `service.rs`, `drive.rs`: mounting, priorities, public/private storage and provider changes |
| Charger and Wooden Crank | `charger.rs`, `glue.rs`, `charger_inventory.rs`: recipe work, crank rules, local/grid power, original inventory persistence, source crank facing states and per-facing shape; complete crank placement/animation remains open |
| Charger/Drive/Crank orientation and wrench turns | `block_states.rs`, `orientation.rs`, `wrench.rs`, `block_orientation.py`: 24 source facing/spin states, six crank facing states with source shapes, inherited identity structure hooks, source placement and Certus Quartz Wrench turns; client prediction, crank placement/animation, disassembly and permissions remain open |
| Inscriber | `inscriber.rs`, `inscriber_glue.rs`: four source slots, 15 original recipes, press/inscribe modes, private/grid/crank power, speed-card rules, capacities, separated automation and auto-export; source menu, name presses, upgrade lifecycle and complete packets remain open |
| Energy cells and Controller | `controller.rs`, `energy_cell_glue.rs`, `block_states.rs`: network power, Controller structures/states, finite-cell fullness and comparator output |
| Powered items and finite energy-cell items | `item_energy*.rs`, `energy_cell_item.rs`, `energy_cell_model.rs`: charging, energy bars, item models and implemented placement/drop energy paths |
| ME Drive | `drive_glue.rs`, `gui.rs`, `automation*.rs`: ten actual cell slots, supported backends, power/channel gating, filtered screen and hopper/dropper access |
| Cable parts and CableBus | `cable.rs`, `part_storage.rs`, `cable_bus.rs`, `node_data.rs`: types/colors, center NBT, saved node data and reload identity |
| Cable item placement and drops | `part_placement.rs`, `cable_items.rs`, `player_registry.rs`: target/admission rules, entity obstruction, survival/creative consumption, named part drops, buckets and original owner-array persistence |
| Cable input and Pick Block | `ctrl.rs`, `cable_input.rs`, `cable_pick.rs`: configurable physical key, authenticated per-player opposite-side state and hit-selected bare center item |
| Cable placement results | `part_use.rs`, `cable_actions.rs`, `cable_prediction.rs`: ordered source results, client preview, server consumption and sequence-based correction |
| Adventure stack use | `adventure.rs`, `adventure_snbt.rs`, `adventure_native.rs`: actual-hand PASS gate, original saved predicates, typed property ranges and NBT comparison; component cache and complete BE codecs remain open |
| Cable builders and world rendering | `cube_builder.rs`, `cable_render.rs`, `cable_view.rs`: original cable quads/textures, graph-derived connections/channels, outline and contextual shapes |
| Certus buds and cluster drops | `quartz.rs`, `survival_loot.rs`: original loot tables and native tool/random-sequence handling, missing-entity refusal, matching stage states/shapes; natural growth and full source block lifecycle remain open |
| Fluid transforms | `transform_throw.rs`, `transform_throw_world.rs`: source server timer, recipe ingredient claims and both built-in Fluix recipes at the surviving per-entity tick phase; client effects, explosions, arbitrary fluid tags/components and full lifecycle remain open |
| Client state and stack components | `cable_sync.rs`, `component_sync.rs`: native visual transport and typed item components, preserving authoritative saved data |

The Inscriber currently operates through automation and its original four inventory slots. Its source
menu and name-press workflow are still pending. Controller and Inscriber crafting recipes and the
original sky-stone materials are registered.

The Drive supports all 23 built-in cell profiles, but portable and tool cells do not yet have their
complete handheld behavior. Its screen exposes ten real slots; priority controls and the complete
AE2 menu remain unfinished. Charger supports 16 powered-item profiles and both finite energy-cell
items. All 85 cable items place their own center parts, and survival mining returns those parts with
their custom names. The opposite-side key defaults to Left Control and is configurable in native
mod settings. Pick Block selects the actual center item; creative Ctrl can include block-entity data.
Adventure-mode rework is suspended for now, including its component cache and lifecycle. Side
parts, facades, partial breaking, Color Applicator placement and complete client/security/lifecycle
behavior remain to be implemented.

## Layout

```text
mixins/ae2/
  horizon-mixin.toml     native modules and injections, pinned to Horizon 26.3 (aa27eca)
  keys.rs, cells.rs     keys, counters, component codecs and cell profiles
  storage.rs, backend.rs
                        finite, creative and restricted cell inventories
  grid.rs, network.rs   topology, channels and native network snapshots
  energy.rs, power.rs   energy stores and grid energy service
  mounts.rs, service.rs, drive.rs
                        storage providers, mounts and Drive rules
  charger.rs, controller.rs, cable.rs
                        machine, Controller and cable rules
  inscriber.rs, inscriber_glue.rs
                        source processing callbacks, recipes, inventory and power/automation
  cube_builder.rs, cable_render.rs
                        source-derived cable geometry and textures
  *_glue.rs, glue.rs    Horizon world, inventory, energy and lifecycle adapters
  part_placement.rs, cable_items.rs, player_registry.rs
                        cable placement, drops, waterlogging and player ownership
  cable_view.rs, cable_sync.rs
                        native cable rendering, shapes and client state
  orientation.rs       source Charger/Drive facing+spin placement rules
  *.rs                  menu policies, item views and small injection helpers
src/lib.rs              engine-independent rules as a library for cargo test
examples/               Rust drivers for Java differential comparisons
tools/
  upstream.py           fetch pinned sources/jar/engines and audit source coverage
  prepare_content.py    prepare the content mod from your AE2 jar
  horizon.py            export pinned Horizon, apply the bundle, check/test/build
  package.py            produce the source-only .hnmod
  block_orientation.py  source BlockOrientation -> Horizon blockstate rotations
  smoke_*.py            isolated native game scenarios
  *_oracle.py           differential checks against pinned Java behavior
docs/                   full plan, parity boundaries, inventory and evidence
```

## Build and run

Needs Rust 1.99.1, Python 3.12, Git, prepared Horizon 26.3 resources, and the native build dependencies
for your platform. Content preparation and Java differential comparisons need JDK 21;
Java is never required at game runtime. The engine revision is `aa27eca`
and horizon-mixins is `3f99a5c`; full pins are in [compatibility.json](compatibility.json) and
[upstream.lock.json](upstream.lock.json).

Fetch the pinned checkouts and run the standalone checks:

```sh
python3.12 tools/upstream.py fetch
python3.12 tools/upstream.py fetch-engines
python3.12 tools/upstream.py fetch-platform
python3.12 tools/check.py
```

Build the native game with the bundle:

```sh
HORIZON_MIXINS=target/upstream/horizon-mixins \
  python3.12 tools/horizon.py build --horizon target/upstream/horizon
```

Prepare the content mod from your own `appliedenergistics2-19.2.18.jar` and the resources of the same
Horizon version. The launcher keeps prepared resources under
`<launcher data>/versions/<version>/.local/horizon-client/26.3`.

```sh
cargo build --release -p horizon-port-pack --manifest-path target/upstream/horizon/Cargo.toml
export HORIZON_PORT_PACK="$PWD/target/upstream/horizon/target/release/horizon-port-pack"
export HORIZON_RESOURCES=/path/to/prepared/26.3
export JAVA_HOME=/path/to/jdk21

python3.12 tools/prepare_content.py \
  --jar /path/to/appliedenergistics2-19.2.18.jar \
  --resources "$HORIZON_RESOURCES" --port-pack "$HORIZON_PORT_PACK"

target/horizon-cargo/fast/horizon-client \
  --resources "$HORIZON_RESOURCES" --dev-mod target/content/ae2
```

Content preparation downloads hash-verified original Minecraft 1.21.1 reference dependencies
into ignored `target/` and derives property domains for AE2-owned metadata. Installed Horizon
resources remain read-only; the mod loader builds a separate effective resource cache.

Use a separate game directory for development worlds. Add `--offscreen` for runs driven by
`horizon-ctl`; screenshots still work. Native checks and tests use the same pinned checkouts:

```sh
python3.12 tools/horizon.py check
python3.12 tools/horizon.py test
python3.12 tools/horizon.py test --library
```

For packaging, `python3.12 tools/package.py` writes `target/ae2-0.1.0.hnmod`. Build the separate content
archive with `cargo-horizon-mod build` from `target/content/ae2`. The native archive contains sources;
the content archive contains the prepared data/assets. Both use mod ID `ae2`. Launcher installation
and rebuild remain separate from the built-client checks documented in [VALIDATION.md](VALIDATION.md).

## Licences

Derived AE2 code is **LGPL-3.0-or-later**, with the original MIT notice for its API. Fabric-derived
square-coordinate rules retain their Apache-2.0 notice; adapted build tooling retains its MIT notice.
See [LICENSE](LICENSE), [NOTICE.md](NOTICE.md) and [LICENSES](LICENSES).

AE2's textures/models are **CC BY-NC-SA 3.0**, translations are CC0, and the guidebook click sound is
CC BY 4.0, as recorded in NOTICE.md. Assets are prepared locally from the player's own jar and are not
included in this source repository. Prepared content also carries the pinned NeoForge common tags
with their licence and provenance. Not affiliated with Team Applied Energistics, Horizon, Mojang or
Microsoft.
