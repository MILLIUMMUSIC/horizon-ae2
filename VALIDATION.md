# Verification evidence

Last updated: 9 November 2026. Development host: macOS arm64, Rust 1.99.1,
Python 3.12 and JDK 21. Pins: AE2 db17504, Horizon aa27eca, mixins 3f99a5c.

## Retained native ME Chest host

- Formatting, Clippy with warnings denied, 196 standalone tests, 17 Python
  tests and source audits pass. The inventory records 154 partial classes out
  of 1,429; no whole class is marked complete.
- Native compilation/build, 259 game tests and 20 library tests pass. New tests
  use the actual save codec to check transient identity, retained cell contents,
  power, empty native padding, malformed refusal and stale-host protection. The
  actual state table/model catalog loads all 48 facing/lights_on/spin rows and
  selects both multipart layers without waterlogging or dispenser behavior.
- `tools/smoke_me_chest.py` passes seven isolated world groups: all 48 states,
  unpowered input refusal, own-cell insertion and 1.5 AE/tick local idle drain,
  own-cell isolation from a connected Drive, malformed inventory refusal,
  replacement and native save/restart. `docs/me-chest-native.json` records the
  exact native source/binary hashes and samples; the driver checks that neither
  changes during the run. The original multipart housing is visible in the
  captured game image; dynamic cell/color rendering is still pending.
- Eight center CableBus regression groups pass on the same completed binary,
  including power/color topology and restart. Evidence and scope are in
  `docs/me-chest-cable-regression.json`.
- The actual native-mod reader accepts the 168-entry package. Its extracted
  corresponding source independently passes all 196 standalone tests and
  contains the host, world driver and both native regression reports.
- This command-initialized host checkpoint does not establish either Chest
  screen, external item/fluid/power interfaces, live CONFIG, source client
  streams, full native event ordering or complete same-tick save/drop/command-
  load lifecycle. Source malformed/nonfinite recovery is not claimed.
- The native build retains unused-code warnings for pending bindings and the
  known nonfatal rust-objcopy/libLLVM warning. Original Horizon resources and
  source checkouts remain untouched; generated roots and worlds are in target.

## Shared powered-host network store (previous)

- Formatting, Clippy with warnings denied, 196 standalone tests, 17 Python
  tests and source audits pass. Source coverage remains 153 partial classes.
- `tools/shared_power_oracle.py` compares 256 scenarios / 29,907 operations
  against the original energy classes with a retained WRITE-only owner and
  three grid services. It checks local work before attachment and while
  detached, node migration, simulation/commit, resizing, buffer shares, events
  and ticks. Source/binding/driver/input/output hashes are in its report.
  Math/collection/grid/event infrastructure is controlled; finite values and
  a fixed unique priority are covered with 1e-11 relative / 1e-8 absolute tolerance.
- The graph tests retain one store through split/merge and reattachment, isolate
  a replacement physical host from old handles, and check WRITE-only local power,
  all six exposed faces, REQUIRE_CHANNEL and initial node idle 1 AE/tick.
- Native check, 253 game tests and build pass. Existing dead-code warnings and
  the nonfatal rust-objcopy/libLLVM warning remain. Native Chest registration,
  world-host/callback/menu binding and full lifecycle are not active yet.
- Eight existing center CableBus world groups pass on the completed build,
  including power/color topology and restart. Runtime/source hashes and scope
  are in `docs/shared-power-cable-regression.json`. This is a regression of
  existing centers, not native Chest host or UI validation.
- The actual native-mod reader accepts the 167-entry source package. Extracted
  corresponding source independently passes all 196 standalone tests; archive
  checks include the new oracle, driver and world-regression evidence.
- Regression comparisons pass: 66,048 multipart cases, 200 single-grid energy
  scenarios / 40,482 operations and 200 overlay scenarios / 52,083 operations.
  Detailed regression reports remain under ignored target/review-2026-11-08/.

## ME Chest host callback core (previous)

- Formatting, Clippy with warnings denied, 193 standalone tests and 17 Python
  tests pass. Inventory coverage is 153 partial classes of 1,429, none complete.
- `tools/me_chest_oracle.py` compares 4,005 scenarios of original MEChestBlockEntity
  methods, monitor and fluid-handler code. The original parent multiplier/local
  extraction methods, complete StorageHelper/StoredEnergyAmount and four AppEng
  inventory notification methods execute with actual Minecraft Mth.clamp.
- Ordered traces and final cache/handler/idle/status/priority, inventories, two
  controlled cells and local/grid energy match. This covers grid/local fallback,
  retained idle, cached failed lookups, own-cell transfers, menu choices, guarded
  reentrant inventory callbacks, cell/input replacement during commits, multiplier
  changes, fluid fill and the differing capacity/validity predicates. Source,
  binding, reference-dependency, driver, input and output hashes are recorded in
  `docs/me-chest-oracle.json`; only the enclosing class name is adapted where
  the original fluid handler uses a qualified `this`.
- Cells, stacks, registry, grid and world/menu callbacks are controlled. Finite
  positive CONFIG and finite requests/storage are covered; transfers require an
  attached level. Source nonfinite energy and pre-level content faults, native
  capabilities/registration/menu rendering/transport and complete lifecycle are
  not established by this comparison.
- The actual native-mod reader accepts the 167-entry package. It includes the
  complete Chest core, oracle, shared fixture and evidence; the extracted source
  independently passes all 193 standalone tests. The source/reference checkouts
  for Horizon, horizon-mixins and horizon-create remain unmodified.
- That standalone comparison did not change any registered native module or
  injection; native suites/world checks were not repeated at that checkpoint.

## Shared storage-menu core (previous)

- Formatting, Clippy with warnings denied, 193 standalone tests and 17 Python
  tests pass. The inventory records 151 partial classes of 1,429, none complete.
- `tools/storage_helper_oracle.py`: 12,960 whole-original-class callback traces
  compare power-limited simulate/commit, item/fluid factors, signed boundaries,
  nonfinite power and statistics against the Rust core.
- `tools/incremental_update_oracle.py`: 30,015 operations run the actual original
  helper with Guava BiMap/HashSet and controlled equality/object identities.
  Pending sets are sorted for comparison; Java HashSet iteration is excluded.
- `tools/me_interaction_oracle.py`: 44,253 original-record encode/decode/dispatch
  cases execute actual Minecraft FriendlyByteBuf, VarLong and VarInt with real
  Netty. This includes malformed consumption, trailing bytes, all action ordinals
  and current-menu/window refusal. Menu and player bindings are controlled.
- `tools/storage_menu_oracle.py`: 18,096 source-method scenarios compare complete
  ordered traces and reachable cursor/slot/network/power/container state. Seven
  original interaction/fill/empty methods, original StorageHelper/update helper
  and source remainder/placeable helpers run with controlled stack identity,
  two destination slots and bucket/tank strategies. Reentrant callbacks replace
  cursors/slots or change storage; partial fluid commits and null-key faults
  retain source boundaries. Reports record exact input/source/binding/output hashes.
- `tools/inventory_update_oracle.py`: 1,258 whole-original-packet/builder and
  extracted Repo-method scenarios compare exact bytes, retained helper state
  and final client entries. This includes 32,767-entry/512-KiB flush boundaries,
  repeated build, full/change filters, removal/reassignment and partial key-codec
  errors. Actual MC buffer varints run with controlled optional-key encoding.
  HashSet iteration is aligned by eight small unique key hashes; arbitrary hash
  order, actual key registry/components and full Repo views are excluded.
- Packaging retains all five drivers, Rust examples, the shared fixture and
  reports. The actual native-mod reader accepts the 166-entry archive; all 193
  tests also pass from its extracted corresponding source.
  These cores are not native-registered; no Horizon runtime payload
  changed and native game suites were not repeated for this standalone change.

Actual native session/host/window authority, real slot/capability bindings,
client updates/search/view cells/settings, crafting UI, statistics/sounds and
complete lifecycle/multiplayer remain required for the terminal and ME Chest PRs.

## Meteorite placement checkpoint (previous)

- Formatting, Clippy with warnings denied, 184 standalone Rust tests, 17 Python
  tests and source/stack-limit/native-tick audits pass. Source inventory covers
  143 partial classes out of 1,429; none is claimed complete.
- `tools/meteorite_oracle.py` compares the whole original placer/putter/fallout
  program in 329 controlled-world cases: 202,360 ordered events, 162,875 writes,
  33,350 placement-RNG calls, 301 separate level-RNG calls and 5,053 water ticks.
  It checks float radii/config flags, all crater/fallout modes, clipped/disjoint
  slices, separate/aliased streams, complete state copying, bedrock/equal-state
  refusal, discard callbacks, source bounding-box center and final written maps.
  Source/dependency/binding/driver/input/output hashes are in
  `docs/meteorite-oracle.json`. Terrain/properties/tags/entity queries are
  controlled; this is not a natural worldgen or native physics comparison.
- Native check/build pass; 250 game tests, 19 native library tests, 26 render
  tests and 9 native item-crate tests pass. Cube state tests retain vanilla IDs,
  full collision/opacity, correct empty schema/BE flag and non-conductor bits.
  Item tests retain the original quartz actor/RNG guard and check ordered tag
  output, a singleton's unchanged RNG and empty-tag alternatives behavior.
- `tools/tag_entry_reference.py` executes actual Minecraft 1.21.1 TagEntry in
  20 reference cases, using controlled vanilla-item tag members. Expanded and
  unexpanded choices, ordering and successful empty tags match the contract used
  by the native tests. This is a reference check, not a whole-pool differential
  comparison; predicates, luck/weighted pools and world contexts are excluded.
- `docs/meteorite-native.json` records twenty private-world groups on the built
  binary: full crater height, source shell/core, all seven fillers and five
  fallout/lake modes, bedrock/slice protection, full waterlogged stair copying,
  actual intersecting-item discard, both config toggles, invalid/unloaded refusal,
  four presses plus guide, Silk Touch, tool-free Level destruction and restart.
  Runtime/content/payload hashes remain unchanged throughout the final smoke.
  The same binary/prepared content passes thirteen quartz-loot regression groups.
- The native package passes Horizon's actual `native-mod` reader with 161
  entries, below its 512-entry limit. Complete corresponding sources remain in
  deterministic `data/source.zip`; that extracted Cargo tree independently
  passes all 184 standalone tests. Packaging tests check both archive levels,
  licence notices, path rules, required tools/evidence and repeated byte identity.
- The placement command is an explicit operator-two development probe through
  authenticated player chat. The automation console's vanilla dispatcher does
  not route registered native server commands. A separate preview at (-8,70,40)
  produced a real rendered meteor/crater; this was explicit placement, not a
  naturally selected structure. Preview images stay under ignored `target/`.

Natural structure registration/admission/biome selection, original placer-item
interaction, settings NBT/config reload, compass notifications/discovery, cube
tooltip and full inherited/client BE lifecycle remain open. The cube TagEntry
adapter covers player and generic Level resource drops; explosion/entity loot,
full luck/weighted context and world-pack loot/tag reload need further integration.
Native builds retain existing dead-code warnings and the host's nonfatal
rust-objcopy/libLLVM debug-stripping warning; they are not warning-free builds.

## Packed part-rotation core checkpoint (previous)

- Formatting, Clippy with warnings denied, 168 standalone Rust tests, 16 Python
  tests and source/stack-limit/native-tick audits pass. Coverage is 132 partial
  Java classes out of 1,429; no native side-part feature is claimed.
- `tools/quad_rotation_oracle.py` executes the original QuadRotator class,
  original BlockOrientation constructor/direction methods and extracted Fabric
  accessors with real pinned JOML 1.10.5, Minecraft FaceBakery/FaceInfo/Mth and
  the original EncodingFormat. All 38,689 cases match: 16,128 panel quads,
  12,000 arbitrary/degenerate quad cases and 10,561 packed-normal cases.
- The 24 quaternion/direction rows and face winding indices are independently
  regenerated/checked before comparisons. Identity is a true raw-word no-op;
  signed spin keeps the source unchecked linear enum indexing. Winding changes
  positions/UV only, retains colors/light/normals at their original indices and
  lets the last coincident vertex supply UV. Tests also cover normal clamp,
  signed-byte decoding and opaque header preservation.
- Source/binding/dependency/table/driver/output hashes are in
  `docs/quad-rotation-oracle.json`. JOML's licence is preserved from the verified
  dependency archive. Reference-init logs and artifacts remain under `target/`.

This is a standalone rotation core for PR #3's rendering dependency. It is not
registered as a native payload until the actual model collector consumes it.
No runtime Horizon payload changed, so native builds/world tests were not
repeated for this core-only change; the previous 234 game/26 render tests and
eight center-world groups concern the previous runtime checkpoint. Default
scalar JOML options, non-null nominal faces and valid cull indices are covered.
Nonidentity NaN position outputs are compared canonically; identity compares
raw words. Face-normal/geometry cache, baked-model loading, AO/lighting, GPU
and side-part world activation remain required. Big-endian conversion is
implemented but not exercised by a big-endian Java/native platform run.

## Reporting-part state checkpoint (previous)

- Formatting, Clippy with warnings denied, 163 standalone Rust tests, 16 Python
  tests and source/stack-limit/native-tick audits pass. Source coverage remains
  125 partial classes out of 1,429; no complete-terminal claim is made.
- `tools/reporting_oracle.py` executes eleven verbatim AbstractReportingPart
  callbacks and AEBasePart's two flag-stream methods. All 249,125 cases match:
  171,660 placement, 1,024 wrench, 65,536 flag/stream/model, 8,313 numeric-NBT and
  2,592 light/cache cases. The verified original Minecraft Mth.floor,
  CompoundTag and numeric tags execute through reflection. This includes
  arbitrary float bits, integer boundaries, signed spin/remainder, suppressed
  flag notifications, lazy off-state opacity queries, sentinel invalidation and
  coordinate/brightness overflow. See `docs/reporting-oracle.json` for hashes.
- The native saved-part bridge reads the source numeric spin and saves only a
  Byte spin, preserving node/unknown data. Opacity and client flags stay transient.
  Node idle and reporting geometry consume the same source profile/constants.
- The 20,410 geometry and 66,048 primary-topology comparisons pass again with
  updated dependency hashes. Their world-level scope is unchanged.
- Native compilation/build, 234 game tests and 26 native render-crate tests
  pass on the frozen source set. The new native codec case checks that signed
  spin changes preserve `gn` and unknown fields without saving opacity.
- `docs/reporting-cable-regression.json` records eight existing center CableBus
  world groups passing on the final binary, including dense edge Pick Block,
  real power/color topology and restart. This is a center regression, not a
  world-level validation of reporting-part callbacks. Native source-package
  validation passes; prior native client-library/item suites concern unchanged
  payloads and are not counted as newly repeated runs.

Player, host, world/node, inherited callbacks and model interfaces in the state
oracle are controlled bindings. These rule/codec proofs do not activate native
side-part placement, wrench handling, lighting, rendering, menus or complete
managed-node/packet lifecycle. The full-port objective and active center-only
world boundary remain unchanged.

## Multipart geometry checkpoint (previous)

- Formatting, Clippy with warnings denied, 159 standalone Rust tests and 16
  Python tests pass. The source inventory contains 1,429 Java files and 125
  partial classes. Stack-limit and carried native item-tick source audits pass.
- `tools/part_geometry_oracle.py` runs the original BusCollisionHelper class,
  Reporting boxes, CableBus selection and existing extracted cable box methods.
  Its 20,410 comparisons include 7,378 side transforms, 3,456 explicit-axis
  transforms and 9,576 host-shape/selection cases. The verified original
  Minecraft 1.21.1 AABB constructor, inflation and containment execute through
  reflection. Exact/adjacent boundaries, reversed coordinates, signed zero,
  infinities and canonical NaNs are included. Binding and result hashes are in
  `docs/part-geometry-oracle.json`.
- Direction vectors, host storage, connection/neighbor state and absent facades
  remain controlled bindings. These comparisons do not execute VoxelShape union,
  GPU rendering, native movement callers or complete part lifecycle. Native
  centers consume the helper; supported saved primary reporting shapes remain
  behind the inactive side-part boundary.
- Native game tests pass (229), including saved-host primary geometry and facade
  refusal. Native render-crate tests pass (22), including actual World/registry/
  block-entity instances for all five cable types at ordinary and large
  coordinates. They check outline bounds, physical VoxelShape bounds and local
  center selection. Run that suite with `tools/horizon.py test --package
  horizon-client-render --no-audio`; `--library` alone targets a different crate.
- Native compilation/build and source-package validation pass. On the same
  final binary, `docs/part-geometry-world-smoke.json` records eight CableBus
  groups, including dense edge rays and real bare Pick Block for both dense
  types. `docs/part-geometry-placement-world-smoke.json` records sixteen groups:
  all 85 real held-item placements, five survival mining/name cases, actual
  player obstruction, both dense types on either side of the physical boundary,
  consumption/rollback, ME power, water/buckets, ownership and restart. Fixtures
  await loaded chunks and assert their floor. Content is not rewritten.

These are isolated integrated-client worlds with supplied inputs. They do not
prove source-game GPU equivalence, every collision caller, side-part placement,
full lifecycle or independent multiplayer. Original repositories, installed
resources and player saves/settings remain unchanged.

## Primary multipart node checkpoint (previous)

- Formatting, Clippy with warnings denied, 155 standalone Rust tests, 16 Python
  tests, source inventory (1,429 Java files / 124 partial classes), stack-limit
  and carried native item-tick source audits pass. Native compilation and 224
  game tests pass. The previous 19 native library and 7 native item-crate tests
  concern unchanged payloads and are not newly repeated here.
- `tools/multipart_oracle.py` compares 66,048 valid primary-host configurations
  with verbatim CableBus connection/add/remove/node-query methods, the Reporting
  constructor and CablePart channel/stream methods. Cases cover occupied and
  blocked sides, four neighbor patterns, all cable types and side-only hosts.
  Registry, node lifecycle callbacks, grid helper and finalized ad-hoc counts
  are controlled bindings. Existing separate pathing/energy oracles cover those
  algorithms; this run does not establish full managed-node lifecycle parity.
  Source, binding, driver and result hashes are in `docs/multipart-oracle.json`.
- Core regressions cover independent co-located nodes, internal null-direction
  edges, external-face blocking, channel limits, per-slot replacement and host
  replacement. Native codec regressions cover per-part node persistence, owner
  identity, stale-write refusal and bounded lookup among 256 observed hosts.
- Raw stream channels remain independent of power. Rendering applies both the
  original powered getter and CableBusContainer's smart-cable gate, including a
  powered five-cable-type regression.
- The final native build passes seven CableBus world check groups: 32 physical
  states, 85 center IDs and node saves, center outline picking, power through
  five cables, color isolation/reconnection and restart. Four power scenarios
  pass on the same binary: finite supply, front-face refusal, creative supply
  and restart conservation. `docs/multipart-cable-regression.json` and
  `docs/multipart-power-regression.json` record binary/binding hashes. These
  exercise existing center behavior, not side-part activation or Java-world
  rendering equivalence. Native source-package validation also passes.

The active native adapter remains center-only. Primary terminal snapshots and
per-slot persistence are groundwork, not a playable terminal. Source geometry,
selection, placement/removal, light, rendering, real storage menus, external/
secondary tunnel nodes and complete lifecycle/packets remain required. PR #3
is not resolved by this checkpoint. Original engine checkouts, installed
resources and player saves remain untouched.

## Fluid-transform checkpoint (previous)

- Formatting, Clippy with warnings denied, 149 standalone Rust tests, 16 Python
  tests, source inventory (1,429 Java files / 123 partial classes) and stack-limit
  audit pass. Final native compilation/build and 213 native game tests pass.
  The previous 19 native library and 7 native item-crate tests concern unchanged
  payloads; those suites were not rerun for this game-only adapter change.
- `tools/transform_oracle.py` extracts the verbatim original
  `ItemEntityMixin.handleEntityTransform` and `TransformLogic.tryTransform`
  methods. Both match 5,014 controlled scenarios / 212,490 callbacks, including
  strict `>60`, signed overflow, failure reset, success retention, greedy
  repeated-ingredient claims, removal and six level RNG draws. Fluid/cache,
  query/recipe order, ingredient tests, assembly and RNG are controlled bindings;
  identity-map iteration, native physics and source client effects are excluded.
  Source/method/driver/input/output hashes are in `docs/transform-oracle.json`.
- `tools/item_tick_source.py` verifies the carried pinned native tick body after
  removing exactly three marked AE2 insertion regions. Its regression rejects
  an unmarked pickup-rule mutation. The bridge is part of the mod under the
  preserved Horizon MIT notice; original engine checkouts/resources are unchanged.
- `tools/smoke_fluix_throw.py` compares both recipes semantically with the pinned
  jar, then uses normally prepared content in a private world. Its 18 checks
  cover actual player throw/pickup, the 60/61 boundary, the Fluix Dust recipe,
  zero output pickup delay, three successive successes, last-tick expiry,
  failure/retry reset, post-movement fluid sampling, dry refusal, and ordinary
  item merging/despawn/pickup. Fixtures await loaded chunks and authoritative
  drops; exact boundary cases use frozen-world server steps. No content is rewritten.
- The same final binary passes `tools/smoke_inscriber.py`: all 15 carried source
  recipes and the full 23-group machine/automation/comparator regression. The
  shared ingredient bridge preserves original item/tag objects and flat
  alternatives, refusing unrepresentable alternatives instead of dropping them.
- `docs/fluix-world-smoke.json`, `docs/fluid-inscriber-regression.json` and
  `docs/item-tick-source-audit.json` record this checkpoint. Native source-package
  validation passes. These are separate rule, native, packaging and world proofs.

Remaining fluid-transform requirements include source client lightning,
explosion injection/quantum frequency assembly, arbitrary fluid tags and result
components, original entity-query/recipe/cache iteration and reload, native
command object replacement and complete identity/unload lifecycle, joint-mod
hook ordering and independent multiplayer. Timers are transient and scoped by
Game/world/dimension; unrelated native items do not acquire sidecar entries.
This is not full TransformLogic, ItemEntityMixin or full-mod parity.

## Inscriber dependency checkpoint (previous)

- Formatting, Clippy with warnings denied, 144 standalone Rust tests, 15 Python
  tests, source inventory (1,429 Java files / 119 partial classes) and stack-limit
  audit pass. Native compilation/build pass, with 203 game tests and 19 native
  client-library tests. The unchanged item-catalog payload retains its 7-test
  quartz checkpoint; it is not counted as a newly repeated run here.
- [Original Java callback comparison](docs/inscriber-oracle.json): 4,504 scenarios,
  261,592 callbacks and 4,504 initial sleeping queries execute four verbatim
  Inscriber methods. State, modulation and callback order match, including
  power thresholds, local/grid selection, output blocking, partial insertion,
  input consumption and smash phases. Inventory/energy/recipe/configuration
  interfaces are controlled bindings; real external auto-export is a stub there.
- [Controller crafting and press copy](docs/controller-craft-world-smoke.json)
  passes on the native client. The source jar and carried Controller/press
  recipes are compared before play. A real 3x3 crafting table makes the
  Controller, and the actual Inscriber copies a calculation press using grid
  power while retaining the original. Ingredients are supplied and the machine
  menu is explicitly outside this scenario.
- [Inscriber world scenario](docs/inscriber-world-smoke.json): 23 check groups
  cover all 15 original recipes, both modes and reversed plates, four real slots,
  nominal default 2,020 AE per operation, inventories/power across restart,
  refused front power, thirteen crank turns / sixty AE remaining, four speed
  cards / 2,500 AE, AUTO_EXPORT wake/export, live hopper admission/capacity/side
  extraction, six actual comparator/dust configurations and wrench inventory
  retention. Binary and binding hashes identify the verified code.
- [Placement regression](docs/charger-orientation-world-smoke.json): twelve checks
  cover the existing Charger/Drive poses, dry/steep/wet Inscriber placement and
  explicit item block-state precedence over orientation/water. All 24 Charger/
  Drive and 48 Inscriber model bindings load. The same final binary also passes
  [crank-only Charger charging](docs/charge-certus-world-smoke.json).
- The new native library case loads all 48 states through the actual asset/state
  table/model loaders and checks fluid rows, default state, light blocking and
  source Block behavior. Game regressions cover inventory echo/malformed repair,
  configuration wake, slotted upgrade counts, side/spin rules, insertion limits
  and minimum-limit comparator arithmetic.

Inscriber menu/name presses, arbitrary component-result recipes, upgrades
installation/removal/drop callbacks, complete cache/TickManager/managed-node
lifecycle, client prediction and animated render/source packets remain open.
Sky Stone uses source 50/150 and smooth sky stone 5/150 properties; their original
cooking recipe remains carried, while optional Athena connected textures are not
ported. Strict malformed-inventory refusal is still a documented deviation.
These worlds supply inputs and do not prove full survival or independent
multiplayer parity. Original repositories, installed resources and user saves
remain untouched.

## Certus quartz loot checkpoint (previous)

- Formatting, Clippy with warnings denied, 143 standalone Rust tests, 15 Python
  tooling tests and source/stack-limit audits pass. Native compilation/build
  passes; 198 game tests, 18 native client-library tests and 7 native item-crate
  tests pass. Item tests use `tools/horizon.py test --package horizon-items`.
- Native item-catalog regressions verify that the missing-entity path returns
  before RNG draws or named-sequence creation and leaves vanilla blocks alone.
  Supplied Silk Touch/Fortune tools, sequence selection, untouched level RNG and
  the mandatory zero-Fortune nextInt(1) draw are checked through the real native
  evaluator. Those unit tables are controlled fixtures; the world scenario uses
  the actual carried AE2 tables from the pinned jar.
- [Quartz world scenario](docs/quartz-world-smoke.json): 13 checks cover plain
  and Silk Touch harvesting of the cluster and all three buds, Fortune III's
  four-to-seven cluster crystals, and support loss with no drops for all stages.
  Source tool requirements remain enabled. The test uses the normally prepared
  content read-only, with an isolated game/home and verified loaded floor.
- Normal preparation uses the matching small/medium/large/cluster amethyst
  templates for the original dimensions and facing/waterlogged state schema,
  retaining AE2's models, loot and stage light levels. The old stairs template
  and test-local full-cluster substitution are no longer used for these blocks.
- Limits: natural budding growth/decay, complete source block properties/APIs,
  full nonplayer explosion-actor loot contexts, AE2 `/loot mine` integration and
  independent multiplayer remain open. The unused, incorrect growth helper is
  excluded until its actual random-tick adapter is ported and verified.

## Crafting and crank charging regressions

- [Crafting-table scenario](docs/cell-1k-world-smoke.json): the actual native
  3x3 table crafts quartz glass, a 1k component and one 1k item cell. Patterns,
  tags, counts and ingredients are compared with the pinned jar. Logic processor
  and raw quartz materials are supplied; Inscriber and growth are not exercised.
- [Crank-only charging](docs/charge-certus-world-smoke.json): ten turns supply
  1,600 AE and convert one certus crystal to one charged certus crystal. No
  creative or finite energy cell supplies the Charger. This verifies the recipe
  and existing power adapter, not the full crank placement/animation lifecycle.
- Both scripts load and verify their private fixture chunks before setup, use
  isolated homes/worlds, and leave the prepared content unchanged.

## Orientation and wrench checkpoint (previous)

- Formatting, Clippy, 138 standalone Rust tests and 15 Python tooling tests pass.
  Native check/build pass; 198 game tests and 18 native library tests pass.
- [Original Java orientation comparison](docs/orientation-oracle.json): 105,489
  placement/direction cases pass, including float yaw boundaries. It executes the
  original placement strategy, SpinMapping and Minecraft direction operations;
  the host context is a stub and the quaternion renderer is not executed.
- Facing and spin are written in one authoritative state change. Wrench turns
  require the main hand and a nonspectator, as in WrenchHook. Structure rotation
  and mirror retain the input state, matching the inherited vanilla methods.
- The corresponding-source archive includes block_orientation.py. A tooling
  regression imports preparation from the extracted archive in isolation.
- The native state adapter now supports shrinking a template state range. For
  `ae2:crank` it replaces the dispenser template's twelve facing/triggered rows
  with the source six facing-only states; rows are ordered facing north, east,
  south, west, up, down, the default is north, and structure rotation/mirror preserve
  the original state. The crank keeps its source `createShape` box per facing in the
  effective outline/collision tables; Charger and Drive carry no shapes.
- The new native library test
  `mod_content::ae2_block_states::tests::facing_only_crank_gets_source_states_and_shape_boxes`
  checks the six rows, default, schema/commands, render/model tables, structure
  rotation/mirror, the exact encoded collision box for every facing, the engine's
  derived state table and all six bound models. The tooling test checks the
  prepared crank block definition carries exactly those boxes and that the other
  hosts do not.
- [Crank state world scenario](docs/crank-states-world-smoke.json): selection
  through the north offset box hits the crank, a ray below its bottom misses, and
  after a wrench turn to east the north-only x range no longer hits while the
  rotated east box does.
- The [wrench rotation scenario](docs/wrench-rotation-world-smoke.json) passes
  with seventeen checks, including spectator/offhand refusal and the crank's
  facing-only state strings.
- Limits: crank placement orientation/canSurvive, TESR animation and
  synchronization, statistics/fake players, the inherited dispenser behavior
  chain and independent multiplayer remain open. This checkpoint does not
  complete the Crank class.

## Wrench rotation checkpoint (previous)

- Formatting, Clippy, 138 standalone Rust tests and 14 Python tooling tests pass.
  Native check/build pass; 197 game tests and 17 native library tests pass.
- Four new `ae2_orientation` tests pin the source rotation behaviour:
  `Direction.getClockWise`/`getCounterClockWise` for each axis, four quarter
  turns returning the original orientation, spin derivation from the rotated up
  reference, and refusal of unknown directions/faces. These run in the
  standalone suite and the native game and library test builds.
- [Wrench rotation world scenario](docs/wrench-rotation-world-smoke.json): 15
  checks with actual player clicks and explicit faces. Four up-face turns cycle
  a Charger north/east/south/west/north with one swing each; two east-face turns
  derive spin 2 through the X axis; a Drive turns without opening its menu; a
  vanilla dispenser is untouched; the network tool and a non-wrench item do not
  turn the Charger; an attached Crank turns north/east and refuses a side without
  a crankable; the Charger's power and unknown NBT survive turns; rotated states
  and the Charger data survive save/quit/relaunch.
- Rotation is gated on the common `c:tools/wrench` item tag and excludes
  `ae2:network_tool`, matching the source `ConventionTags.WRENCH` check and
  `NetworkToolItem` special case. Shift mode does not rotate.
- Limits: shift-disassembly, permissions/claims, the network tool UI, client-side
  prediction and independent multiplayer are not covered. Only Charger, Drive and
  Crank have native orientation strategies today. Facing and spin now change in one authoritative write. Refused/vanilla
  fallthrough click swings are not compared. These classes remain
  partial.

## Charger/Drive orientation checkpoint (previous)

- Formatting, Clippy, 133 standalone Rust tests and 14 Python tooling tests pass.
  Native check/build pass; 193 game tests and 17 native library tests pass.
- The native library test
  `mod_content::ae2_block_states::tests::oriented_hosts_keep_source_states_and_inherited_structure_identity`
  builds real content over the pinned resources and checks 24 contiguous rows
  for each of Charger and Drive: facing-major/spin-minor order, the source
  north/0 default, model/render/gameplay/behaviour/command/worldgen tables, NBT
  state arrays, structure rotate/mirror (identity), the engine's derived state
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
| [wrench-rotation-world-smoke.json](docs/wrench-rotation-world-smoke.json) | PASS | 15 player turns and refusal checks |
| [crank-states-world-smoke.json](docs/crank-states-world-smoke.json) | PASS | 4 selection/shape checks |
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
