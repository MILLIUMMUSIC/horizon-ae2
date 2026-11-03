# Verification evidence

Last updated: 3 November 2026. Development host: macOS arm64, Rust 1.99.1,
Python 3.12 and JDK 21. Pins: AE2 db17504, Horizon aa27eca, mixins 3f99a5c.

## Current Charger/Drive orientation checkpoint

- Formatting, Clippy, 133 standalone Rust tests and 14 Python tooling tests pass.
  Native check/build pass; 193 game tests and 17 native library tests pass.
- The native library test
  `mod_content::ae2_block_states::tests::oriented_charger_and_drive_get_source_facing_spin_states_and_rotation_mappings`
  builds real content over the pinned resources and checks 24 contiguous rows
  for each of Charger and Drive: facing-major/spin-minor order, the source
  north/0 default, model/render/gameplay/behaviour/command/worldgen tables, NBT
  state arrays, structure rotate/mirror (facing only), the engine's derived state
  table and all 24 bound models. Test models are empty fixtures; their binding
  proves the state-to-model wiring, not the carried geometry.
- `tools/block_orientation.py` derives the Horizon x/y/z triple for every one of
  the 24 source `BlockOrientation` matrices and refuses a prepared variant whose
  source x/y/ae2:z keys disagree. The tooling tests compare all 24 matrices, the
  prepared fixture and the refusal path. Preparation now writes all 24
  Charger/Drive variants instead of collapsing to six facing models.
- [Charger orientation world scenario](docs/charger-orientation-world-smoke.json):
  six actual player placements read back the native state: Charger horizontal
  north/0, steep-down up/3, steep-up down/0 and down/1; Drive horizontal north/0
  and steep-down up/3. All 24 models bind without a model-load warning.
- The Charger inventory/charging smoke passes again (five checks) with the new
  24-state schema: original inv.item0 reconciliation, live charging, malformed
  refusal and restart.
- Limits: orientation is applied by the server placement hook; the integrated
  client's local prediction still starts from the dispenser template state until
  the block update. Wrench rotation/disassembly, complete Charger/Drive machine
  menus/models, crank orientation, the template's inherited redstone behavior and
  independent multiplayer are not covered. These partial classes remain partial.

## Adventure stack-use checkpoint (previous)

- Formatting, Clippy, 128 standalone Rust tests and 12 Python tooling tests pass.
  Native check/build and 188 game tests pass; the library suite passes 16 tests.
- [Original Java comparison](docs/adventure-oracle.json): 73,399 cases, comprising
  3,970 saved-predicate cases, 625 NBT comparisons, 3,255 SNBT parses and 65,549
  integer parses. The verified original 1.21.1 classes execute their real codecs
  and matchers over original states and chest entities. LevelReader is a read-only
  proxy and per-case tags are synthetic bindings; this is not a full Java game.
  Each component object is fresh. Original Level/LevelChunk bytecode separately
  establishes on-demand `IMMEDIATE` lookup; those methods are not exercised by
  the proxy-level comparison. Class/helper/input/output hashes are in the report.
- [Adventure world scenario](docs/adventure-world-smoke.json): 24 checks use
  actual mouse input and actual stacks in an isolated integrated native game.
  Holding server packets distinguishes client preview from server mutation and
  verifies unchanged preview counts, one successful swing and later server
  consumption. Missing/denied permissions return PASS and permit the offhand
  shield. A permitted/denied offhand cable consults its own component without
  spending the main-hand stick. Block reactions still precede disallowed items.
- World conditions cover FULL/SIMPLE/fallback forms, empty holder sets, real block
  tags, Java enum ordinal ranges and invalid enum-domain bounds, Unicode integer
  bounds and invalid property-domain values, direct compound and SNBT masks on
  Energy Cell/chest BEs, and rejected trailing SNBT. Client NBT lookup uses the
  registered BE constructor if a loaded block has no client instance; it retains
  existing instances and does not fabricate NBT or copy the server's save.
- The same final client passes the 14-case survival/creative center-action
  regression: five cable types, one preview/swing/consumption, obstruction/FAIL,
  offhand suppression, acknowledgement, correction and block priority. Its report
  is nested in the adventure world report.
- Property facts are generated from 1,060 original vanilla block definitions and
  seven supported AE2 hosts, under the mod's own `data/ae2/` namespace. Reference
  artifacts are hash-verified build/check dependencies in ignored `target/`;
  no installed Horizon resources are edited and no Java runs in the native game.

These checks establish fresh predicate evaluation and the native stack gate.
Adventure-mode rework is suspended for now at the user's request; the validated
codec, matcher and actual-hand gate remain in place, but are not being extended.
Source mutable AdventureModePredicate component-object cache, ItemStack.copy
sharing, full original BE save/default/component codecs and lifecycle, new/addon
property domains, exact permissions/security and independent multiplayer are
deferred. Missing native properties cannot be inferred from the source facts;
Charger/Drive spin states are handled by the separate block-state work.
SNBT nesting above the mod's defensive limit of 512 is not covered. Failed or
interrupted runs are not evidence for this checkpoint; only final-source reports
are retained here.

## Current center placement results checkpoint

- Formatting, Clippy and 128 standalone Rust tests pass. Native check/build pass;
  the game suite passes 186 tests and the library suite passes 16. Game filtering
  includes nested `ae2_` modules, including the remote prediction ledger.
- [Part-use Java comparison](docs/part-use-oracle.json): 10,320 cases of the
  verbatim `PartPlacement.place` method. Results, target/place/failure-update
  order, client/server consumption, nullable/creative players and empty-hand
  replacement match. Target lookup and actual mutation are boolean host stubs;
  real registry/world/platform transport behavior is separate.
- [Placement world scenario](docs/cable-actions-world-smoke.json): previews of
  all five cable types while server packets are held, unchanged client item
  counts, exactly one swing and one subsequent survival consumption, a source
  use-statistic scoreboard, no offhand shield action on FAIL, player/mob
  obstruction, opaque facade refusal, empty-host BE acknowledgement, normal/
  reverse server rejection correction, coalesced placement/break, chest priority
  and vanilla action cleanup. These world checks use survival/creative, not
  adventure mode. The ledger also has a native regression for a
  refused later action receiving host data while the block is locally hidden.
- Both regressions pass with this adapter: all 85 item placements, names/drops,
  water/buckets, ownership/restart and an item-placed cable power chain; then all
  85 Pick Block IDs, Ctrl data, ordered modifier packets and rebound keys.
  Their complete results are nested in the placement world report.

The rejection fixture corrupts only its private native owner map and restores
it afterwards. This exercises the adapter's strict refusal and reconciliation;
it is not a claim about Java corrupt-save recovery. The same integrated bundle
is used on both sides. Full multipart prediction, source security/settings,
deferred adventure component-object cache and complete BE codecs/lifecycle,
unload/replacement/fake-player paths, exact neighbour/event/packet timing and
independent external multiplayer remain unfinished.

## Cable input checkpoint

- Standalone checks pass with 128 Rust tests and 11 Python tests. The source
  inventory has 1,429 Java files, 111 partially ported classes and no complete
  classes. English documentation, formatting, Clippy and stack-limit audit pass.
- Native check and fast build pass. The game suite passes 182 tests, including
  independent world/sender state and mod block-entity clone data preservation.
  The native library suite passes 16 tests.
- [Input world scenario](docs/cable-input-world-smoke.json): real CTRL press/
  release, six coalesced modifier/use packet batches, all 85 bare creative
  center-item clones, creative Ctrl NBT without visual transport fields,
  survival existing/missing stacks, restart, configurable binding and sprint
  remapping independent of the AE2 modifier.

These game checks use the integrated authenticated client/server and private
world/settings directories. The sprint-remap case uses the control tool's repeat
path because its down helper also inserts the canonical action into physical
keys. Independent external clients, complete player replacement/disconnect and
dimension lifecycle, side/facade selection, source-world comparison, full client
prediction and exact FAIL/swing/offhand results remain open.

## Cable input source comparison

[CTRL packet Java comparison](docs/ctrl-oracle.json) passes 11,028 cases using
verbatim packet decode/write/server-handler methods. It covers all 256 byte
values, empty/trailing input and independently updated player attachments.
DataInputStream and a boolean player attachment substitute the Minecraft hosts.
This does not verify NeoForge outer-payload validation, native authentication,
packet ordering, key events or multiplayer lifecycle.

## Cable item placement checkpoint

- Formatting, Clippy with warnings denied, 126 standalone Rust tests, 11 Python
  tests, English documentation, 58 stack-limit registrations and the 1,429-file
  source inventory audit pass. Coverage is 108 partial classes; none are claimed
  complete merely from these tests.
- Native check/build pass. The native game suite passes 178 tests and the library
  suite passes 16. The latter verifies wet/dry fluid rows across all 32 CableBus
  states while retaining vanilla/foreign state and shape mappings.
- [Part placement Java comparison](docs/part-placement-oracle.json): 52,592 cases
  of verbatim target/admission methods, including all directions, CTRL/null
  player, permissions/hosts, and int-boundary coordinates. World mutation,
  registries, collision, settings and events are declared stubs/separate evidence.
- [Cable item world scenario](docs/cable-items-world-smoke.json): all 85 held-item
  placements; opaque facade-host preservation; a real item-placed ME cable chain powering a Charger; five survival
  placement/name/drop cases; obstruction rollback; Level destruction; real bucket
  drain/refill; water restoration; creative/no-drop rules; source names/node data
  and player-ID persistence across restart and another placement.
- Separate finite/creative power, front-face refusal and finite cell/Charger
  energy conservation across restart pass with the current adapter changes.

Full side parts/facades, partial breaking, security/settings,
exact interaction-result/animation behavior, complete event/readiness lifecycle,
all direct shape callers/fluid/light and external multiplayer remain open.

## Current native CableBus stage

- Standalone verification: formatting, Clippy with warnings denied, 122 Rust
  tests, 11 Python tests, English documentation check, source inventory and all
  58 source stack-limit registrations pass.
- Native check and fast-profile build pass. The game adapter suite passes 172
  tests. The native library suite passes 16 tests, including visual wire
  roundtrip/malformed payload rejection, arbitrary saved-field preservation,
  32 CableBus states, appended block-entity type and foreign shape mappings.
- The isolated world check registers all 32 states and 85 center IDs, confirms
  a real client outline/raycast hit, powers a Charger across five cables, splits
  unlike colors, reconnects through transparent cable, and survives save/restart.
- The separate power regression passes finite/creative supply, front-face refusal
  and finite cell/Charger energy conservation across restart.

This checkpoint covers command-created center-only hosts. Contextual movement
and piston shape hooks compile, but all physical movement/direct-query behaviors
have not been proven in a source game. Full AO/lighting, facades, side parts,
survival placement/drop, source lifecycle and external multiplayer remain open.
The quad-builder oracle does not prove the GPU. The native screenshot was visually reviewed after fixing the quad-batch
vertex contract. See [CableBus world report](docs/cable-world-smoke.json). No
complete Java/NeoForge world or screenshot comparison is claimed.

## Verified source and rule layers

- Cable quad builders: 24,405 Java cases, 467,940 quads, 9,358,800 float fields;
  unchanged builder/texture classes and verbatim square/model methods. Exact
  float bits, textures, colors/fullbright, face/cull metadata and order match.
  Sprite atlas and pre-pack emission are controlled stubs; packed normals/AO,
  actual GPU, world collection, attachments/facades and caches are separate work.
- Cable profiles/shapes/payload: 278,956 cases with unchanged API/storage classes
  and verbatim cable methods. Geometry compares raw box calls. Base-part prefix,
  world services, neighbor queries, registries and full host packets are stubbed.
- Node-data equality: 20,064 cases with a verbatim GridNode method. Tag equality
  is a map/string substitute; actual typed NBT roundtrip and graph identity are
  tested natively. Readiness/destruction/service events are not covered by that
  Java oracle.
- Finite cell item power: 100,000 operations; base powered-item/bar regression:
  400,000 operations. Original methods preserve double bits, thresholds, overflow
  and simulation, with declared component/world stubs.
- Controller/finite-cell visual oracles and pathing, energy, storage, basic,
  creative and tool-cell traces retain their exact input/source hashes and
  limitations in the machine-readable reports below.

## Previous native and world checkpoints

The CableBus NBT checkpoint passed 120 standalone tests, 172 native game tests,
11 Python tests and native check/build. Earlier unchanged library/assets payloads
had 14 library and seven asset-model tests. The geometry-only checkpoint passed
122 standalone and 11 Python tests; its 84 previous native mixin files and native
manifest were unchanged, so no new engine or game run was claimed for that step.

Actual isolated worlds have exercised Charger recipes/powered items, finite and
creative grid power, Drive inventories/screens/automation/backends, comparators,
Controller forms/conflicts/drain, cell fullness, item models, placement/drops,
component transport, stack limits, platform predicates and save/restart. These
scenarios establish only their recorded checks, not the entire mod.

World power-_xs22713 verified finite cell -> Charger, front-face exclusion,
creative power and exact energy across recipe/save/restart. World
controller-5_celrs8 passed 14 checks including 18 states, exact 3 AE/tick idle,
a real powered Charger, geometry/conflicts and restart. World
energy-cell-item-z5e22fr2 passed 13 checks for charging, finite-cell item models,
placement/drop boundaries and exact restart energy. Reports list exclusions.

Native fast builds can report unused future APIs and rust-objcopy/libLLVM
warnings. A zero Cargo exit plus game verification is not described as a
warning-free build. Launcher installation/rebuild, external multiplayer, joint
Create gameplay and full Java/NeoForge world parity remain separate requirements.

## Verification failures and interpretation

An early native CableBus integration test incorrectly expected 90 AE after
restoring 100 and extracting 10 into one node. The source buffer capacity is
25 AE/node, so the correct remainder is 15. Source reinspection also established
that restored node energy must be Double-typed. Both are covered by subsequent
native tests. Failed runs are not accepted as evidence for the final tree.

Some early mixin probes exposed exact target-ID/return-type requirements and a
Java stub interface declaration omission. These were corrected in the adapters
or stubs; pinned original sources were not changed. Build/test success must
always refer to the actual final source, not a stale staged tree or old binary.

## Reproduction

```sh
python3.12 tools/check.py
python3.12 tools/horizon.py check
python3.12 tools/horizon.py test
python3.12 tools/horizon.py test --library
python3.12 tools/horizon.py build
python3.12 tools/cable_render_oracle.py --java-home /path/to/jdk21
python3.12 tools/adventure_oracle.py --java-home /path/to/jdk21
python3.12 tools/smoke_adventure.py --resources /path/to/prepared/26.3 --ctl /path/to/horizon-ctl
```

Smoke tools require prepared game resources, horizon-ctl and a built client;
see README.md and each tool's --help. They use owned isolated target directories
and close their own processes. Native packages are rebuilt after tests, checked
with the real package validator, and compared byte-for-byte with committed source.
Game assets/jars/exports/binaries/worlds are excluded from Git.

## Recorded report index

Machine-readable reports preserve detailed historical evidence after the English
consolidation of this document. PASS below reflects an explicit report flag;
Recorded means an audit/metadata/report whose scope must be read directly.

| Report | Status | Selected measurements |
| --- | --- | --- |
| [adventure-oracle.json](docs/adventure-oracle.json) | PASS | 73,399 original-class comparisons; fresh components |
| [adventure-world-smoke.json](docs/adventure-world-smoke.json) | PASS | 24 actual-hand/client/server/NBT checks; scope and limits in report |
| [baseline-port-pack.json](docs/baseline-port-pack.json) | Recorded | See report scope and limits |
| [cable-bus-nbt-power-regression.json](docs/cable-bus-nbt-power-regression.json) | PASS | See report scope and limits |
| [cable-controller-regression.json](docs/cable-controller-regression.json) | PASS | checks: 14 |
| [cable-oracle.json](docs/cable-oracle.json) | PASS | cases: 278,956 |
| [cable-render-oracle.json](docs/cable-render-oracle.json) | PASS | cases: 24,405; quads: 467,940; float_fields: 9,358,800 |
| [cells-oracle.json](docs/cells-oracle.json) | PASS | cases: 2,000; operations: 572,000 |
| [charger-automation-drive-regression.json](docs/charger-automation-drive-regression.json) | PASS | See report scope and limits |
| [charger-automation-world-smoke.json](docs/charger-automation-world-smoke.json) | PASS | checks: 8 |
| [charger-inventory-charging-regression.json](docs/charger-inventory-charging-regression.json) | PASS | checks: 7 |
| [charger-inventory-world-smoke.json](docs/charger-inventory-world-smoke.json) | PASS | checks: 5 |
| [charger-orientation-world-smoke.json](docs/charger-orientation-world-smoke.json) | PASS | 6 player placements; 24 bound models |
| [charger-oracle.json](docs/charger-oracle.json) | PASS | cases: 2,000; operations: 200,000 |
| [component-sync-charging-world.json](docs/component-sync-charging-world.json) | PASS | checks: 7 |
| [component-sync-drive-gui.json](docs/component-sync-drive-gui.json) | PASS | See report scope and limits |
| [controller-charging-regression.json](docs/controller-charging-regression.json) | PASS | checks: 7 |
| [controller-drive-gui-regression.json](docs/controller-drive-gui-regression.json) | PASS | See report scope and limits |
| [controller-visual-oracle.json](docs/controller-visual-oracle.json) | PASS | cases: 4,000; top_level_method_comparisons: 8,000 |
| [controller-world-smoke.json](docs/controller-world-smoke.json) | PASS | checks: 14 |
| [creative-cell-oracle.json](docs/creative-cell-oracle.json) | PASS | cases: 2,000; operations: 572,000 |
| [creative-world-smoke.json](docs/creative-world-smoke.json) | PASS | See report scope and limits |
| [drive-automation-smoke.json](docs/drive-automation-smoke.json) | PASS | See report scope and limits |
| [drive-gui-smoke.json](docs/drive-gui-smoke.json) | PASS | See report scope and limits |
| [drive-inventory-world-smoke.json](docs/drive-inventory-world-smoke.json) | PASS | See report scope and limits |
| [drive-world-smoke.json](docs/drive-world-smoke.json) | PASS | See report scope and limits |
| [energy-cell-charging-regression.json](docs/energy-cell-charging-regression.json) | PASS | checks: 7 |
| [energy-cell-item-base-regression.json](docs/energy-cell-item-base-regression.json) | PASS | cases: 2,000; operations: 400,000 |
| [energy-cell-item-charging-regression.json](docs/energy-cell-item-charging-regression.json) | PASS | checks: 7 |
| [energy-cell-item-oracle.json](docs/energy-cell-item-oracle.json) | PASS | operations: 100,000 |
| [energy-cell-item-world-smoke.json](docs/energy-cell-item-world-smoke.json) | PASS | checks: 13 |
| [energy-cell-visual-oracle.json](docs/energy-cell-visual-oracle.json) | PASS | cases: 100,000; top_level_method_comparisons: 200,000 |
| [energy-cell-world-smoke.json](docs/energy-cell-world-smoke.json) | PASS | checks: 40 |
| [energy-oracle.json](docs/energy-oracle.json) | PASS | cases: 1,000; operations: 202,028 |
| [energy-overlay-oracle.json](docs/energy-overlay-oracle.json) | PASS | cases: 1,000; operations: 260,147 |
| [inventory-signal-source-audit.json](docs/inventory-signal-source-audit.json) | Recorded | See report scope and limits |
| [inventory-signal-world-smoke.json](docs/inventory-signal-world-smoke.json) | PASS | See report scope and limits |
| [item-bars-world-smoke.json](docs/item-bars-world-smoke.json) | PASS | See report scope and limits |
| [item-charging-world-smoke.json](docs/item-charging-world-smoke.json) | PASS | checks: 7 |
| [item-energy-oracle.json](docs/item-energy-oracle.json) | PASS | cases: 2,000; operations: 400,000 |
| [item-energy-view-oracle.json](docs/item-energy-view-oracle.json) | PASS | cases: 2,000; operations: 400,000 |
| [item-stack-charger-regression.json](docs/item-stack-charger-regression.json) | PASS | checks: 8 |
| [item-stack-drive-gui-regression.json](docs/item-stack-drive-gui-regression.json) | PASS | See report scope and limits |
| [item-stack-source-audit.json](docs/item-stack-source-audit.json) | Recorded | See report scope and limits |
| [item-stack-world-smoke.json](docs/item-stack-world-smoke.json) | PASS | See report scope and limits |
| [mounts-oracle.json](docs/mounts-oracle.json) | PASS | cases: 2,000; operations: 410,882 |
| [node-data-oracle.json](docs/node-data-oracle.json) | PASS | cases: 20,064 |
| [ctrl-oracle.json](docs/ctrl-oracle.json) | PASS | cases: 11,028; packet methods with declared host substitutes |
| [cable-input-world-smoke.json](docs/cable-input-world-smoke.json) | PASS | Integrated input, 85 center clones, Ctrl data, restart and bindings |
| [part-use-oracle.json](docs/part-use-oracle.json) | PASS | cases: 10,320; source placement control flow with declared host substitutes |
| [cable-actions-world-smoke.json](docs/cable-actions-world-smoke.json) | PASS | Center preview/results, offhand refusal and server correction |
| [pathing-oracle.json](docs/pathing-oracle.json) | PASS | cases: 2,000 |
| [platform-tags-report.json](docs/platform-tags-report.json) | Recorded | See report scope and limits |
| [platform-tags-world-smoke.json](docs/platform-tags-world-smoke.json) | PASS | checks: 7 |
| [portable-world-smoke.json](docs/portable-world-smoke.json) | PASS | See report scope and limits |
| [shared-codec-charger-regression.json](docs/shared-codec-charger-regression.json) | PASS | checks: 5 |
| [shared-codec-gui-regression.json](docs/shared-codec-gui-regression.json) | PASS | See report scope and limits |
| [tool-cell-oracle.json](docs/tool-cell-oracle.json) | PASS | cases: 2,000; operations: 572,000 |
| [tool-world-smoke.json](docs/tool-world-smoke.json) | PASS | See report scope and limits |
| [upgrade-world-smoke.json](docs/upgrade-world-smoke.json) | PASS | See report scope and limits |
