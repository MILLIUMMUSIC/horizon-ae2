# Implemented behavior and remaining parity

All claims refer to AE2 19.2.18 and the pinned Horizon revision. Source coverage,
compilation, carried resources, and rule-level tests are different evidence.
The full mod is not yet complete.

## Subsystem boundaries

| Subsystem | Implemented layer | Remaining requirements |
| --- | --- | --- |
| Basic cells | All five tiers, portable/tool layouts, bytes/types/rounding, partitions, distribution/void, simulation/extraction, native codec and Drive mounts | Full malformed/missing-content recovery, addon handlers, workbench and upgrade lifecycle |
| Keys/counters/fuzzy | Component-aware item/fluid identities, signed counters, source fuzzy damage intervals, zero/reset behavior | Registry prototype normalization, arbitrary addon key types, identity-hash ordering and all codec recovery |
| Creative cells | Configured exact keys, unlimited transactions, source listing/status/drain, nesting and native mounts | Config writer/UI, images/descriptions, full handler/addon APIs and source iteration order |
| Portable cells | Source capacities/limits/drain, nested-cell restrictions and component-preserving backend | Handheld terminal/menu, energy use, disassembly, colors/LEDs and full upgrades |
| Tool cells | Matter Cannon/Color Applicator layouts, recipe/tag admission, restricted backend and energy retention | Shooting/painting/cycling, live reload, unknown conditions, complete item APIs and ordering ties |
| Grid/pathing | Nodes/edges, routing/finalized channels, controller validation, compressed/multiblock paths, ad-hoc and dense limits | Full source service identity, priorities/ties, events/listeners, ownership/readiness and every host |
| Energy | Finite/creative stores, priorities, local buffer, simulation/commit, multipliers, overlays, thresholds, merge/split | Complete watcher/addon callbacks, config reload, event ordering and external power capabilities |
| Network storage | Preferred passes, mount priorities, mutation queues, recursion protection and weak links | Full handler filters, all providers/consumers, security and lifecycle |
| Storage service | Node/global provider identities, refresh/unmount, public/private counts, caches and watchers | Java service oracle, live world watcher consumers, iterator/callback/hash-order fidelity and debug UI |
| Drive | Ten actual slots, all 23 built-in backend families, power/channel/priority mounting, source inventory, filtered screen and automation | Priority UI, bulk actions, cell models/LEDs, external capabilities, all destruction/chunk/reload paths |
| ME Chest | Retained native input/cell stacks and own-cell backend, shared local energy, grid provider, original inventory persistence, 48 full-orientation/light states and restart checks | Both screens, external item/fluid/power interfaces, source config/color/cell rendering and complete save/drop/load/readiness lifecycle |
| Controller | 8,000 public AE, 3 AE/tick idle, source flags and structures, channel supply, 18 state/type models and world conflict/restart checks | Status menu, power funnel, immediate source event/readiness/shape ordering and spatial lifecycle |
| Energy cells | Network storage, original energy/node NBT, finite fullness states/models and local comparator, deferred notifications | Exact TickManager/base settings, material/lifecycle completeness and every saved-BE/explosion/pick/loot path |
| Finite cell items | Fixed capacities/rates, source removal threshold, Charger support, float item-model thresholds and placement/first-drop energy | Complete tooltip/settings/capability APIs, charged creative variants and every loot/recovery path |
| Charger | Source recipe work, local/grid/refill power order, powered items, original inventory, automation, comparator, full-orientation placement and wrench turns | Live config, capability API, exact render geometry/effects/drop RNG, wrench disassembly and complete lifecycle |
| Inscriber | Four source slots, all 15 built-in recipe data, press/inscribe callbacks, local/grid/crank power, speed-card rules, input capacity, separated automation/auto-export/comparator, original inventory and 48 wet/oriented states | Name-press components and component-result codecs, source menu/settings/upgrades installation and drops, full cache/TickManager/node lifecycle, client prediction and render packets |
| Wooden Crank | Rotation/hits/energy, 18-tick turns, busy window, failure/break rules, Charger-side restriction, source six facing states, per-facing shape boxes and wrench turns | Placement/survival/fake-player/statistics, animation, synchronization and full lifecycle |
| CableBus | Five types/17 colors, center NBT/geometry/render/sync, item placement/drop, names, water/buckets, owner arrays, synchronized opposite-side key, center Pick Block and center placement prediction/results | Full multipart prediction/settings/security, parts/facades, all shapes/fluid/light and source event/selection lifecycle |
| Certus quartz buds/clusters | Matching amethyst-stage facing/waterlogged states, geometry and original loot tables; missing-entity guard, plain/Silk Touch/Fortune tool handling | Natural budding growth/decay, richer explosion actor contexts, complete source APIs and independent multiplayer |
| Fluid transforms | Both built-in Fluix recipes from carried data, source signed timer/threshold/reset, greedy claims, surviving per-entity post-movement/merge/lifetime/environment phase, six level RNG draws and immediate zero-delay fresh entities | Source lightning, explosion recipes/frequency assembly, arbitrary fluid tags and result components, original query/recipe/cache/reload order, command object replacement and complete identity lifecycle |
| Platform/content | Pinned original resources, source stack limits, 461 generated NeoForge common tags and recipe preparation | Full platform services, recipe types/conditions/IDs, registries, progression, addons and coexistence checks |

## Powered-host storage identity

`SharedPower` retains one actual StoredEnergyAmount-compatible store across
managed-node readiness, detachment, reattachment and grid transfers. Cloned
handles alias that store. The EnergyService adapter retains WRITE/public rules;
a Chest can consume its own local energy while that energy remains unavailable
to the grid's provider list. Simulation leaves the store and owner queue alone.
Committed threshold events enter an ordered owner queue for the level binding
to drain outside service locks. The owning Chest still filters PROVIDE into
its display path and forwards RECEIVE to its current grid only when attached.
Native callback delivery at the original interior event sites remains pending.

The graph has a parameterized Chest construction profile: bounded 500*CONFIG
capacity, source priority, REQUIRE_CHANNEL, all six exposed grid faces and a
shared WRITE-only power store. Its initial node idle is 1 AE/tick, inherited
from ManagedGridNode/GridNode; the distinct local idlePowerUsage starts at zero.
A readable cell changes the node/local idle through the host callback program.
Kind.from_block now admits native Chest hosts. The world adapter binds their
actual retained SharedPower before topology snapshots, including while detached.
Both menus and external capabilities remain unbound.

256 original-energy-class scenarios / 29,907 operations compare a retained host
across three grids, local work while detached, simulation/commit, resizing,
owner events, buffer shares and ticks. Math/collections/grid/node/event bindings
are controlled; finite inputs and a fixed unique priority are covered. Numerical
tolerance is 1e-11 relative / 1e-8 absolute. Graph regressions check a fresh host's
separate store, old-handle survival, split/merge and all six faces. Dynamic power-
priority tree mutation, nonfinite power, native callback reentrancy, complete
unload/drop/session identity and complete physical lifecycle remain required.

## ME Chest host rules

`me_chest.rs` keeps the source's two inventories: input at combined slot 0 and
cell at slot 1. The front exposes the cell inventory to item capabilities; other
faces expose the input. Input extraction is denied. The terminal reads the
Chest's own cell even when a grid is connected; grid mounting is a separate
online/priority-gated use of that same cell. The top opens BasicCellChestMenu;
other faces open the separate MEChestMenu. No withdrawal slot is introduced.
Native face routing, menus and capability admission remain unbound.

`me_chest_glue.rs` retains the two native stack handles and cell backend across
ticks, shares that same cell with the grid StorageService, and writes current
contents into the actual cell item. Unchanged source/native echoes cannot reload
spent power or old contents. A transient per-block identity survives the engine's
tick take/put-back but is excluded by the actual native save codec; stale hosts
cannot flush into a replacement. Empty menu padding is retained without becoming
a third inventory. Nonempty extra slots, malformed source inventory and nonfinite
power are preserved through refusal, not claimed as source recovery behavior.
Seven command-initialized world groups and native save/model tests are recorded
in `me-chest-native.json`. Grid-powered operation does not imply automatic
charging of the local buffer. Source CONFIG loading, dynamic power priority,
color/cell client streams, synchronous external callbacks and complete same-tick
save/drop/command-load lifecycle still need binding.

The callback core caches a nonempty cell lookup, including failed lookups, until
a cell change or priority reset. Empty slots stay uncached. Removing or failing
to read a cell does not reset its last idle drain. The readable-cell drain is
1 + cell drain. Cell changes invalidate handler/cache, request storage refresh
and, with a level attached, invalidate capabilities and mark the block. Priority
changes request refresh even when the new value equals the old one. Node online
transitions request storage refresh and recalculate display.

Chest power extraction uses grid energy first, then the local remainder, inside
the inherited multiplier wrapper. CONFIG is read for multiplication and again
for division, preserving a change during callbacks. A locally powered Chest
requires strictly more than 1 AE, while a powered node satisfies the power
check independently. Idle drain runs when the grid is absent or unpowered;
input processing follows it. Input transfers use the actual own-cell handler
and MachineSource, preserve stack identity and delegate guarded onChange/save
notifications to the inventory binding. Successful cell commits persist through
the current handler, mark the block changed, and recalculate the displayed state.

The insertion-only fluid sink exposes no stored fluid or drain result. Its
unindexed capacity/validity queries depend on a current handler, whereas indexed
tank 0 remains 1,000 mB/valid independently. Filling refreshes the handler and
uses StorageHelper's powered insertion with the machine source; it does not
invent a separate isPowered gate or persistent tank inventory.

4,005 original-method scenarios compare ordered callbacks and final state,
including reentrant input/cell replacement, mutable power multipliers, guarded
inventory notifications, unreadable-cell caching, stale fluid-handle predicates
and menu dispatch. Original StorageHelper, StoredEnergyAmount and Minecraft Mth
execute in Java; cells/grid/registry/world/stacks and menus are controlled.
Requests/factors/storage are finite, CONFIG is positive and transfers have an
attached level. Faulting pre-level content mutations and nonfinite energy edges
are excluded. Complete native host/node/power/slot/capability/UI binding, config/
NBT/stream/color/model behavior, full fluid interfaces and lifecycle remain open.
This standalone core does not resolve PR #6 by itself.

## Shared storage-menu interaction core

The standalone menu core retains full server keys behind per-menu serials.
MEInteractionPacket preserves all 21 InventoryAction ordinals, Minecraft wire
encoding (including malformed/noncanonical varint consumption), and dispatches
only to the current compatible handler with the matching container ID. Native
sender/window/host identity and lifecycle must still bind this core.

StorageHelper preserves source simulation/commit ordering, the 0.9 tolerance,
Java floating-to-long saturation, item statistics, and the extraction/insert
energy asymmetry: extraction charges for the original simulated amount while
insertion charges for the reduced amount. The binding must apply CONFIG and
use the actual action source, key and player. IncrementalUpdateHelper retains
serials across full refreshes, discards them on reset/removal, and keeps the
newest equal pending object. Native pending iteration is sorted; it does not
claim Java HashSet order fidelity.

MEStorageMenu's server callback program covers source link/serial/null-key
gates, item actions, retained versus fresh carried-stack references, copied
insertion remainder, quick-move loops and bucket fallback. Inherited fill/empty
loops retain the 256-transfer cap, simulation, error and sound boundaries.
A source null-key filling fault becomes a typed rejection before transfer.
The Java comparison executes the original methods against controlled identity-
preserving stacks, two player slots, storage/power and bucket/tank strategies.
It compares every ordered callback and reachable final state in 18,096 cases,
including reentrant cursor/slot/storage changes. Separate original-class checks
cover 12,960 StorageHelper scenarios, 30,015 incremental-update operations and
44,253 packet cases; reports record hashes and excluded scope.

MEInventoryUpdatePacket's builder preserves full/delta/filter behavior, nullable
keys, zero-removal and commit ordering. It flushes after writing an entry at
512 KiB or 32,767 entries, keeps full=true only on the first packet, retains
built packets across calls, and sends no empty packet when nothing was written.
A partial key-codec failure retains source buffer/serial/pending mutations;
callers must discard failed transmissions. Field encodings use source VarInt/
VarLong/boolean semantics while optional-key registry/component codecs remain
binding callbacks. A full-key union keeps the first equal key object, matching
source storage/craftable/requestable precedence.

The client entry core indexes by serial, retains keys for keyless deltas,
refuses unknown keyless serials with a warning, and removes nonmeaningful entries.
Equal client-visible keys do not merge distinct serials. Full refresh clears
entries; actual view/cache/pin clearing and updateView callbacks are separate.
1,258 original packet/builder and Repo-method scenarios compare exact bytes and
server/client state, including flush boundaries and controlled codec failures.
Small controlled key hashes align Java iteration with native sorted order;
arbitrary HashSet order and real AEKey registry codecs are excluded.

These modules are standalone dependencies for the remaining terminal/ME Chest
PRs. They are not registered as native payloads yet. Actual native menus, client
updates/view cells/settings, host/window authority, real slot semantics and
container capabilities, crafting UI, player statistics/sounds, multiplayer and
full menu lifecycle remain required. No extra ME Chest inventory slot or empty
generic terminal screen is introduced.

## Primary multipart node foundation

The standalone packed-quad rotation core follows QuadRotator, source Fabric
accessors and the original BlockOrientation/JOML constructor. Its 24 rows are
derived as float bits rather than approximated cardinal matrices. Identity
preserves raw data/cache state; winding rewrites only positions/UVs and keeps
last-match UV behavior on degenerate faces. All 38,689 source comparisons pass
with real Minecraft winding and default scalar JOML. This core is not wired
into native part model collection yet. Face-normal/cache, full model import,
AO/lighting, native GPU and complete side-part activation remain open.

Reporting state preserves signed spin and source numeric NBT conversion,
including Float/DoubleTag floor-before-byte narrowing. Vertical placement uses
the source float32 yaw arithmetic; horizontal sides retain spin. Wrench turns
use signed Java remainder and mutate only on the server. Client power/missing-
channel flags use the original low two bits, separate notification predicates,
two-byte reporting stream and off/on/has-channel model choice. Opacity is queried
lazily even when off; only the mounted-side neighbor invalidates the cache.
The native saved-part codec writes spin alone and preserves other part data.
249,125 original callback comparisons execute the actual Minecraft numeric tags,
CompoundTag and Mth.floor. Their player/world/host/node/model interfaces are
controlled, not native side-part input/light/transport or full lifecycle proofs.

`part_geometry.rs` carries BusCollisionHelper's side/explicit-axis transforms,
Java floating-point bounds, reporting-part boxes and source primary selection
order (Down through East, then center). Selection inflates by 0.002 and keeps
Minecraft's half-open upper bound. The original helper class and extracted
Reporting/CableBus/cable methods match 20,410 cases while running the verified
Minecraft 1.21.1 AABB constructor, inflation and containment. Direction vectors,
part storage and neighbor state are controlled bindings, and facades are absent.
NaNs are compared canonically, without asserting payload-bit identity.

Native center outlines, contextual collision and Pick Block consume this
helper. Dense visual bounds are 3..13 pixels; physical bounds are 4.9..11.1.
Selection uses local coordinates before inflation rather than inflating already
translated world coordinates. Saved primary reporting shapes retain separate
slot identities and use the source inherited three-pixel cable connection
length. Unknown parts and facade geometry are refused by that primary-shape
binding; complete facade codecs, shape caching, all direct collision callers
and native side-part activation remain open.

The network binding addresses primary managed nodes by physical position and
optional part side. Co-located nodes retain separate graph identity, services,
saved buffer shares and channel requirements. A supported terminal profile uses
the original Reporting constructor's REQUIRE_CHANNEL and 0.5 AE/tick idle.
Parts connect internally to a cable center with a null-direction connection.
Occupied and blocked sides remove the center's exposed world face; a primary
terminal node never becomes an external-facing node. Side-only hosts have
separate isolated nodes. Removing one part preserves siblings, while physical
BE replacement forgets all hosted nodes.

The CableBus codec can decode primary terminal snapshots and observe/persist
each slot's own `gn`, retaining raw settings/facades/unknown data. Owner-only
changes preserve identity; changed item/node data replaces only the affected
slot. Writes refuse a stale live part rather than overwriting newer data.
Dense cable admission and unknown/malformed saved hosts remain guarded; complete
source registry/admission/recovery is not established by these bindings.

`cable_stream` exposes original raw channel counts, including internal parts;
`cable_visual` preserves the source powered getter and smart-cable rendering
gate. Ad-hoc finalization
stamps the total demand on each connection; controller routing accounts for
individual leaf channels. The host Java oracle controls finalized counts and
does not replace the existing separate pathing/energy proofs.

The active native decoder deliberately still rejects side-bearing hosts until
part shape/selection, placement/removal, light, input, rendering and a real
storage-menu path are bound. This checkpoint is not a playable terminal or a
resolution of PR #3. External/secondary tunnel/fiber nodes, complete source
managed-node identity/readiness/events, packet transport and multiplayer remain
required work. The full-port objective is unchanged.

## Fluid item transforms

The server adapter reads ordinary water/lava recipes from native custom recipe
data, including both original Fluix recipes. It matches all ingredients before
consumption and refuses unrepresentable component/special results without
falling through to a different recipe. A missing circumstance means water;
flowing water and source waterlogged states use the same built-in fluid tag.
Arbitrary addon fluid tags and complete codecs remain open.

The timer preserves Java signed wrapping, the strict `> 60` condition, failure
reset, success retention and removed/ineligible refusal. It runs at each live
entity's end of movement, merging, lifetime and environment processing, before
the next entity ticks. Fresh outputs are added immediately to the entity list
and spatial index, with pickup delay zero; the original range bound prevents
them from ticking during the same pass. They remain visible to later queries,
merging and pickup. Six draws use the level RNG; native bobbing uses the native
entity RNG binding.

The pinned mixin API cannot select an interior site in this closure-bearing
method. The mod therefore carries the pinned `Game::tick_items` body with three
marked insertions under Horizon's MIT notice. `tools/item_tick_source.py`
rejects changes outside those regions. All original engine sources/resources
remain untouched. This bridge still needs joint-mod ordering/coexistence proof.

Transient clocks are scoped by native Game, world and dimension, pruned on
removal, cleared at world lifecycle and checked against native entity ticks.
They are not written into stacks or saves. This does not establish all source
object identity: native command NBT replacement can reset entity tick state;
full command/reload/unload recovery and exact Java entity-query/cache iteration
remain open. Source client lightning and explosion transformations, including
quantum frequency assembly, are separate unfinished features.

## Components, keys, and inventories

Keys retain component identity rather than only registry IDs. Native unknown typed
components are preserved, including genuine `custom_data`, nested stacks, and
source saved keys. Full effective-component normalization against registry
prototypes and all Minecraft 1.21.1 codec recovery remain open.

Malformed/unknown/duplicate/nonpositive or over-capacity cell entries are refused
without mutation. This is stricter than source missing-content recovery. Native
tooltip key lookup currently covers known item IDs and vanilla water/lava;
custom fluids and the external fluid capability API remain open.

Upgrades use the original slotted item-container form, for example
`[{slot:0,item:{id:"ae2:void_card",count:1}}]`, not a flat card list. Slot/card
limits follow pinned registrations; stack count does not multiply installed cards.
Energy cards use their physical count as in source. Writer/callback/workbench
behavior and capacity expansion on real inventory edits remain incomplete.

Drive and Charger share optional-stack inventory decoding: empty compound,
omitted default count one, original `inv.itemN`, and a native `Items` transport
mirror. Positive loaded counts above normal slot limits are preserved. Normal
menu/automation insertion still has source limit one. Malformed source inventory
stays blocked until explicit repair; a changed native echo cannot erase it.
Unchanged echoes cannot restore spent power or removed contents.

Drive has ten real slots and a native `generic_9x2` transport screen with eight
hidden, prohibited padding slots. Its policy is keyed by authoritative actor,
window, and actual block. Padding is not extra storage, and this is not AE2's
wire protocol. Priority UI, ghost icons, special bulk actions and external
multiplayer are still incomplete.

## Power, lifecycle, and controller state

The native world adapter polls loaded block entities before/after level ticks.
It preserves same-tick energy use and detects NBT changes, but polling is not the
entire source event/readiness/chunk model. Equal saved data alone does not prove
object identity after every replacement path. Finite cell item placement explicitly
forgets the previous node even at an identical position/kind/saved amount.

Local node buffer capacity defaults to 25 AE per node. Source energy is restored
from node `e` only when the tag is Double. Native power validation refuses
negative/nonfinite values along guarded paths; this does not establish the same
recovery as every Java NaN/infinity/negative-number branch.

EnergyCellBlockItem has fixed 200,000/1,600,000 AE capacities and 800/1,600 rates;
foreign capacity/upgrades are preserved but do not override that profile. Its
removal threshold is 0.00001, unlike powered items' 0.0001. It does not have the
base powered-item green bar. Its original model uses float fifths; placed finite
cells use the separate double quarter/fullness formula and local comparator.

The render-only cell model projection never writes `block_state` into real stacks,
packets, saves or click hashes. A private render bridge carries NaN/infinity
through an otherwise lossy JSON view. Java model predicate bytecode and native
resolver tests cover those cases; arbitrary malformed native recovery is open.

Placement imports finite cell `stored_energy`. Implemented destruction paths
copy positive local energy into only the first matching drop, without `proxy.e`.
Live guards do not overwrite newer external NBT or restore same-tick expenditure.
All saved-BE/command/explosion/pick/loot/base-settings paths still require work.

Controller status/type updates currently run after the server tick. Original
immediate shape, readiness, listener and spatial ordering are not fully reproduced.
Finite-cell comparator/fullness notifications wait for ticking chunks and execute
external callbacks after releasing the network lock.

Current native configuration uses source defaults, power multiplier/charge factor
one, and default channel mode. Rule APIs accept more configurations; live suppliers,
reload and all upgrade callbacks are not fully connected.

## Cable profiles, persistence, and geometry

Cable profiles preserve five types and 17 colors, enum ordering, independent
variant/size min/max, palette/tints, connection lengths, raw boxes and channel
scaling. Dense flags replace `PREFERRED`; they do not add to it. Normal/dense
capacity is eight/32 before mode scaling. Transparent/equal colors permit
in-world edges; full blocks use transparent grid color.

Multipart storage keeps center, six side references, and six facade references.
`setPart(null)` allocates its array; `setFacade(null)` removes without allocation.
This storage rule is not an implemented facade item or complete part registry.

The saved-host adapter reads `cable.id`, `cable.gn`, six side names and
`facadeDown/Up/North/South/West/East`. It never guesses a fluix glass cable from
the host block ID. It retains raw unknown host/part/facade/service fields.
Unsupported side parts keep the host inactive because they need their own nodes
and services. Facade BlockState codecs and a full source host writer remain open.

Raw NBT item-ID changes, including externally changed colors, replace the part;
direct source painting instead recolors its existing node. Non-owner node data
changes trigger replacement; absent/empty/owner-only data compare equal. Own
saves update the observed snapshot, and persistence refuses a newer unmatched
center rather than overwriting it. `gn.p` is an Int on save; positive buffer is
written to `gn.e`, and zero removes that field.

Java `GridNode.savedData` represents its previous load/grid transition. The polling
adapter compares against observed/own-save data. Explicit BE-load events after
same-value edits, complete readiness/destruction and service callbacks still need
a source-faithful event layer. Unsupported IDs, side parts, malformed YesNo tags
and negative/nonfinite energy use a stricter refusal policy than source recovery.

Cable quad construction preserves source float32 operation order, UV rotations/
flips/custom rectangles, texture IDs, colors/fullbright, core overrides, straight
shortcuts, internal connections and machine end caps. Dense groups use wrapping
`(channels+3)/4`; straight dense segments extend to -0.01/16.01 pixels. The smart
machine-cap path retains its bright vertex color for the following small segment.

`Quad.cull` is emitter bookkeeping; source cable BakedQuads are returned unculled.
Applying that field as ordinary world-face culling would be incorrect. Texture
suffix `transparent` differs from item prefix `fluix`. The world collector must
resolve connection types with the source minimum and use actual graph power and
visual channels. All side/facade rendering and source packet behavior remain
completion requirements.

The native center stage registers all 32 light/waterlogged states. It renders
original cable quads through the entity quad batch and appends the original part
atlas source through an AE2-owned atlas extension. Selection/raycast and the
SceneCollisions contextual movement/piston shape paths consume center geometry.
Other direct collision callers still use the static glass-core fallback; full
support/occlusion/fluid/light behavior and all collision callers remain open.

An owned effective resource registry appends the real `ae2:cable_bus` block
entity type without renumbering vanilla types. Chunk and block updates carry
center NBT and a versioned, transient visual payload derived from graph channels.
The client unwraps it only at packet receipt; save decoding preserves arbitrary
source NBT. This same-bundle transport is not AE2's complete source packet or
external-client negotiation. Full lighting/AO, attachment/facade geometry,
complete survival part interactions, partial breaking and multipart drops remain unfinished.

## Cable items, drops, and ownership

All 85 cable PartItems enter the native item-use path. Central parts try the
clicked position first, then the adjacent position on the opposite face. An
occupied center is never overwritten. New hosts use the source waterlogged
state. The full connected cable shape is tested against native building-blocking
entities; failed placement removes the provisional part/empty host and consumes
nothing. Explicit placement forgets the previous node identity even for equal
kind/position/NBT. Topology refresh does not advance the energy tick again.

Survival consumes one item; creative consumes none. Player mining and Level
resource destruction return the original cable part ID and dismantle custom name,
without carrying node owner/buffer or arbitrary item components. Creative mining
and disabled block drops produce none. Native Pending carries the part-drop
snapshot only until the corresponding drop transaction is cleared; this private
transport never enters real item components or source saves.

The 16 wet states contain a water source in the behaviour table, and the host
implements the native SimpleWaterlogged interface. Actual buckets drain/refill
the host while retaining its cable; removal restores water. This is not evidence
of complete source fluid propagation, shape support or light behavior.

Owning UUIDs map to source integer IDs using original `playerIds` IntArray and
`profileIds` LongArray data in overworld `data/ae2_players.dat`. Native assignment
reads the world-wide map and uses native gzip-NBT writes with backups; all dimensions share
it. Corrupt mappings are refused without overwrite. Native tests cover signed
UUID halves, duplicate IDs, repeated readers and persistence. A complete public
player-registry service and all migration/config/security behavior remain open.

Hosts with raw side/facade payloads refuse central placement until their codecs
and full shapes are available; failed admission never clears those payloads.
Side-part/facade admission, segment targeting, Color Applicator offhand placement,
all generic settings/security callbacks and source readiness/event ordering are
unfinished. The configurable physical opposite-side key defaults to Left Control.
It sends changed boolean state through an authenticated native channel, scoped
by world and player. The server handles it before following use packets in the
same batch and retains the shared native per-tick allowance. World/death reset
and connection initialization are implemented; complete replacement/disconnect,
dimension and external multiplayer lifecycle still require validation.

Pick Block hit-tests supported center geometry and returns the bare part item,
without its dismantle name or node state. Optional creative Ctrl block-entity
data encodes the actual Mod block entity before the native item-component
collector; the vanilla canonicalizer cannot serialize a mod type. Unsupported side/facade hosts
are refused rather than guessed. Survival selection uses the existing native
inventory path. Full facade/side selection and vanilla/source-game data-component
comparisons remain open.

`part_use.rs` preserves the source PASS/FAIL/sided-success control flow. A failed
center placement ends the click without swinging or trying the other hand.
Client success swings once and does not shrink the item; server success consumes
one survival item, awards its use statistic and triggers the block-use criterion.
Scoped adapters retain the block's own reaction and native mod priority; results
do not remain on a player or affect a later vanilla action.

The client constructs a supported center and checks its local obstruction shape,
without writing the server graph, owner registry or saved node state. A new
client cable starts with no streamed connections, as the source constructor does.
Native block sequences retain known server states, including reverse CTRL
targets. A companion BE ledger retains the original/last received host and
settles it with the sequence, so an acknowledgement cannot retain a refused
preview or restore stale empty-host data over a verified part. Later native
actions on the same position extend that host's pending sequence; server host
data is remembered even while a predicted break hides it. Connection,
dimension/respawn and leave paths reset preview state. Explicit CableBus updates
bridge the host while Horizon temporarily lends its hub to a guest packet.

The generic adventure ItemStack gate runs before the part coordinator, following
pinned NeoForge's `ItemStack.onItemUse` and `CommonHooks.onPlaceItemIntoWorld`.
A missing/disallowed `can_place_on` returns PASS, leaving the other hand eligible.
Both server use and client prediction consult the actual passed stack. Block
reactions still precede the item gate; a permitted part can still return FAIL
from its placement checks. Adventure-mode rework is suspended for now; this
existing gate remains in place.

The saved predicate decoder follows original 1.21.1 FULL/SIMPLE alternative
codecs, including fallback after an invalid full form. Block/tag selectors,
exact/ranged typed properties, direct compound/SNBT masks and NBT subset/list
matching use source rules. Enum bounds compare Java ordinals, not spelling;
integer properties use Java's accepted BMP decimal digits and property domain.
SNBT is parsed by the mod's original-format parser, independently of Horizon's
newer syntax. Its defensive nesting limit is 512; deeper source parser behavior
is not claimed. Source facts cover 1,060 original vanilla blocks and seven
currently supported AE2 hosts in an AE2-owned data file. An absent/new/addon
domain or missing native property refuses that state condition; it is not a
complete dynamic property API. Charger/Drive now expose their native facing/spin
states, so those property conditions have native state data; new/addon domains
remain absent.

The matcher evaluates fresh predicates on real native state/entity data. Client
NBT lookup constructs an absent registered BE on demand for a loaded block and
keeps existing instances; it does not copy a server save into the client. Full
original BE save/component codecs and source default-field reconstruction are
still required for every host. Native 26.3 data is not automatically a complete
1.21.1 save. Adventure-mode rework is suspended for now at the user's request:
the validated decoder, matcher and actual-hand gate stay working, but source
AdventureModePredicate's mutable component-object cache, ItemStack.copy sharing,
reload/identity behavior and all permissions/security/fake-player paths are
deferred rather than being extended. Fresh codec comparisons and isolated game
checks do not establish those contracts.

The native visual envelope also carries raw side/facade occupancy fields for
correct refusal. This is not an implementation of those parts or AE2's full
source packet. Exact source neighbour/event/packet timing, full settings/security,
multipart prediction and all unload/replacement/fake-player paths remain open.
These are partial placement paths, not complete PartPlacement/IPart parity.

## Charger and Drive orientation

The native state adapter replaces the dispenser template's twelve facing/triggered
rows with the source 24 facing+spin states for `ae2:charger` and `ae2:drive`, and
shrinks the template independently for `ae2:crank` to the source six facing-only
states. Rows are ordered facing major (spin minor where the block has spin),
default to facing north (and spin 0), and rewrite the model-state, render,
gameplay, behaviour, command, worldgen and NBT tables together. The crank keeps
its source `createShape` box per facing, applied to the effective outline and
collision tables. Structure rotation and mirror preserve the complete original
state: AEBaseBlock and these hosts do not override BlockBehaviour's identity
hooks. Wrench orientation strategies are independent of those hooks.

Prepared blockstates translate the jar's source-only `ae2:z` rotation into
Horizon's plain x/y/z quarter turns. `tools/block_orientation.py` checks every
source variant against the pinned `BlockOrientation` table and finds the
equivalent Horizon triple by matrix equality; this is a conversion of the same
transform, not an approximation. Server player placement rewrites the placed
state to the source `FacingWithSpinStrategy` orientation, including the 65-degree
pitch thresholds and the `SpinMapping` up reference.

The client's local placement prediction still starts from the engine's template
state until the server block update. The Certus Quartz Wrench turns supported
oriented hosts around the clicked face through the common `c:tools/wrench` tag,
excluding `ae2:network_tool`, using the pinned `Direction.getClockWise` rotation
and the source spin derivation. A turn that would not change the state falls
through to the block's own use, matching `newState != state`; a crank turn is
refused unless the new attached side is a crankable. The wrapped world check
covers four up-face turns, two X-axis turns incl. spin derivation, a Drive turn
without its menu, refused vanilla/network-tool/non-wrench clicks, crank survival,
block-entity data retention and restart. Shift-disassembly, permissions/claims,
the network tool UI, complete Charger/Drive menu and model composition, the
crank's placement orientation/survival, animation and synchronization, and
external multiplayer remain open. The native adapter writes facing and spin together through one authoritative
`Level.setBlock` call, preserving the existing block entity.

## Native networking, recipes, and platform limits

AE2 stack components travel through a versioned wire-only native envelope that
preserves genuine custom data and unwraps before normal decoding. It retains
nested stack data and stable click hashes. It requires the same native bundle;
external multiplayer negotiation and arbitrary addon component APIs remain open.

Original recipe subdirectories are flattened because the native loader currently
requires it. The source-to-native ID map is recorded and collisions are rejected.
Advancements, guidebook/UI references and other source-ID consumers need a proper
engine adapter. Unsupported recipe types remain carried data and do not execute.

Charger execution refuses results with unsupported components; ingredient-based
automation admission does not silently invent a prohibition. Random samples are
taken from native world RNG only under source conditions, but whole-world RNG
call order has not been compared tick-for-tick with Java. Current dropped items'
position/velocity do not yet match the source random spawn rules.

Tool admission expands known recipe/tag conditions. Unknown conditions, including
inside negation, are refused. Native recipe-table order is used for ties; source
RecipeManager ordering and live reload remain unverified.

The 461 NeoForge common tags are pinned and layered before AE2 tags with original
replace/provenance rules. World checks cover selected item predicates/restart,
not every registry/tag. No NeoForge JVM runtime or general platform API is bundled.

The port and Create use separate IDs/helpers. Joint mixin planning has passed;
a joint running-game/capability validation has not. Launcher installation/rebuild,
external LAN/dedicated clients, all survival progression and large-network
performance remain separate unfinished requirements.

## Verification limits

Differential reports preserve source hashes, exact methods, inputs and scope.
Java stub-host oracles prove the covered rules with the listed substitutes;
they do not prove the complete mod, registry codecs, game events or GPU behavior.
Native tests and actual isolated game checks are documented separately in
`../VALIDATION.md`. A successful package check validates the archive contract,
not the gameplay of every carried item.


## Certus quartz loot

`CertusQuartzClusterBlock.getDrops` returns no drops when THIS_ENTITY is absent.
The mod injects that guard at the native item catalog before table evaluation or
named-sequence allocation. Player contexts use the original carried tables and
native evaluator unchanged: the actual passed tool controls Silk Touch/Fortune,
clusters yield four crystals plus the uniform Fortune bonus, and smaller buds
yield one dust. A zero-Fortune cluster still consumes its nextInt(1) draw;
Silk Touch and the smaller buds do not use that bonus function. The level random
is not a substitute for the table's named random sequence.

Normal content preparation binds each stage to its corresponding vanilla
amethyst stage, preserving source dimensions, facing/waterlogged properties,
default up direction and support behavior. It retains AE2's models and loot.
Smoke tests consume the prepared content read-only; they must not rewrite it or
turn off source tool requirements to pass. Natural budding growth/decay is not
yet wired. The native loot context currently represents THIS_ENTITY as a boolean
for player breaks; full nonplayer explosion-actor semantics remain unfinished. The native `/loot mine`
route also constructs vanilla-only table IDs and omits the actor; its AE2 command
integration is not yet ported. Direct table rolling is distinct from block getDrops.


## Inscriber

`inscriber.rs` retains the source callback order and the strict
`processingTime > 200` condition. Default speed is two steps and twenty AE per
callback; with full nominal grants it spends 2,020 AE before starting its smash cycle.
Four speed cards use fifty steps and five hundred AE, requiring five work
callbacks / 2,500 AE. The local buffer is simulated first. If it cannot meet the
threshold, a complete request goes to the grid; two insufficient sources are not
combined. Source `ticksSinceLastCall` does not multiply this work. Smash advances
one step per callback, inserts the result at step eight, and finishes at sixteen.

The four real slots remain top, bottom, middle and output. `press` consumes one
of each input; `inscribe` consumes only the middle. Plates may be reversed.
Configured input capacities are ONE, FOUR and SIXTY_FOUR; output keeps its own
limit. `ContainerAdapter.getMaxStackSize` nevertheless takes the minimum of all
host slot limits for comparator arithmetic. The native hook retains that rule
and actual item limits without a synthetic fullness clamp.

Saved inventories use original `inv.item0` through `item3` alongside the native
transport mirror. Count-only input changes retain work, input component changes
reset it, output changes wake it, and AUTO_EXPORT changes wake an idle host.
Legacy `inscriber_buffer_size=NO` migrates to FOUR. Saved speed upgrades use the
original slotted list and count occupied card slots, not stack counts. Native
hoppers enforce the source recipe/base filter, configured capacity, smash locks
and separated top/bottom/side views. Auto-export uses actual neighboring native
containers and source direction order. The private buffer is 1,600 AE, is not a
public network store, and accepts crank turns away from the front. ME connections
exclude the front and expose COVERED cable geometry.

The block owns 48 facing/spin/waterlogged states with source default north/0/dry,
inherited identity structure hooks, metal properties, no occlusion and light
blocking two. Placement retains pre-placement source water in a per-hand scope;
explicit item block-state properties take precedence. The same precedence is
preserved for Charger and Drive. Source Controller/Inscriber crafting and the
normal/smooth sky-stone registrations are prepared from the pinned definitions;
plain sky stone has hardness fifty, smooth five, both resistance 150. Optional
Athena connected textures remain unported.

Remaining requirements include the actual source machine menu, name-press text
components, arbitrary component-result recipes, upgrades installation/removal,
upgrade drops and full capability/settings APIs. Native inventory decoding keeps
the existing strict malformed-save refusal; this is not source recovery parity.
The per-host schedule binds initial rate ten and urgent/slower/sleep behavior,
but complete shared TickManager queue/readiness, recipe-cache identity/reload,
replacement/unload/fake-player lifecycle, client prediction, source notification
and packet timing, and animated rendering remain unfinished. The world scenarios
supply ingredients and do not establish complete survival progression.


## Meteorite placement

The original `MeteoritePlacer` program runs through a world adapter, retaining
its float radius, crater/fallout modes and pure/lake/config flags. It applies
crater clearing through the inclusive maximum build height, bounding-slice
clamps, sky-stone/quartz ordering, bedrock and identical-state refusal,
full-state downward copies, source decay budget and lazy random draws. The
crater discards intersecting dropped items. Lake placement replaces only the
canonical air block, uses separate placement/level random calls and schedules
water at delay zero. The native development caller aliases both random streams
to the actual level RNG, as the original placer item does.

Copy fallout reads tags from the native zoomed biome at the bounding box's
center. It does not substitute the meteor position or an assumed default biome.
The original mysterious cube has metal properties (hardness 10, resistance
1,000), a non-conductor predicate and a server tracking block entity. Its converted original
loot table/tag remain authoritative: normal mining returns the four Inscriber
presses and a guide; Silk Touch returns the cube. The cube adapter supplies
TagEntry to the native player/Level block-loot paths: unexpanded tags are one
weighted choice emitting every ordered member. The input table retains its
dynamic tag reference; preparation translates legacy `name` to native
`items: "#tag"`. Other blocks retain the native evaluator. Explosion/entity
loot routes, custom weighted/luck contexts and complete world-pack loot/tag
reload still require integration. Meteorite quartz blocks use
source hardness 3 and resistance 5. Budding growth/decay remains unported.

The server registers an operator-level-two development command:

```text
/ae2meteorite x y z radius crater fallout pure lake flawless presses minX minY minZ maxX maxY maxZ seaLevel
```

Crater names are `none`, `normal`, `lava`, `obsidian`, `water`, `snow`, `ice`;
fallout names are `none`, `default`, `sand`, `terracotta`, `ice_snow`. The four
flags are `true`/`false`. The diagnostic radius range is 2..8; slices must be
ordered, at most 256 blocks wide on either horizontal axis, and loaded before
the command starts. The center must leave nine blocks below and thirty above
within the level's build range. The explicit sea level is an input to this
probe, rather than an inferred value for every generator. Registry/state and
loaded-chunk preflight refuses incomplete inputs before mutation or RNG use.
For example, on a loaded slice in a normal Overworld:

```text
/ae2meteorite 8 73 8 4 lava default true false false true 0 -64 0 15 319 15 63
```

This command invokes the source placement program on explicit inputs. It is
not natural structure generation, `MeteoriteSpawner` terrain admission, automatic
biome/crater selection, the original placer item's interaction, structure-piece
NBT persistence or configuration loading/reloading. Those callers, worldgen
registries, biome-tag reload, compass discovery/change notifications, cube tooltip,
full inherited block-entity/component/packet lifecycle and source world-physics
comparisons remain required. Java traces use controlled terrain/entity/biome
bindings; native game tests are separate evidence. External multiplayer remains
to be validated.
