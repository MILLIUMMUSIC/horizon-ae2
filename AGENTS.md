# Applied Energistics 2 for Horizon

User requirement: complete 1:1 parity with the pinned AE2, not an AE2-like replacement.
All README files and repository documentation must be written exclusively in English.
The user explicitly authorized creating and publishing the public GitHub repository
MILLIUMMUSIC/horizon-ae2 with gh. Keep game assets/builds/worlds out of Git.
Work without subagents. Read README.md, docs/PLAN.md, docs/PARITY.md,
upstream.lock.json and compatibility.json before editing.

Keep this repository separate from horizon-create, Horizon and horizon-mixins.
Never edit their original sources, player resources, saves or launcher settings.
Export only committed pinned engine sources into ignored target trees. Reference
AE2 Java is the specification; do not approximate missing behaviour silently.
NeoForge's generated common c tags come from the pinned read-only checkout in
target/upstream/neoforge. Preserve its licence/provenance and resource layering;
never replace missing platform tags with ad-hoc item lists.

Rules live in mixins/ae2/*.rs and are exposed by src/lib.rs for standalone tests.
cable.rs feeds network.rs with source cable types/colors/flags and core shape/
visual rules. Cable profiles are binding-only: do not infer a fluix glass cable
from ae2:cable_bus, and do not claim native placement/renderer from profile tests.
Dense flags replace PREFERRED. Recolor preserves node identity; side masks and
color compatibility rebuild in-world edges. cable_bus.rs now consumes
part_storage.rs and node_data.rs for native saved-host parsing and snapshot
identity. It reads cable.id/cable.gn, never root proxy, and preserves raw side/
facade/unknown fields. Unsupported side parts keep the whole host inactive;
registry/facade codecs and full save/load recovery are still pending. Owner p-only
edits preserve identity; external part-item ID or other gn edits reset it. This
NBT load path differs from direct painting's in-place recolor. Persist only a
still-matching observed center, and update the observed gn after our own save.
Native center-only registration/render and outline/contextual movement hooks are
implemented. cable_items.rs/part_placement.rs add all 85 central-item placement
and drop paths, full-shape entity obstruction, names and water/buckets. Per-part
nodes/readiness, facades, direct collision callers, partial breaking, complete
settings/security, multipart input results and source packets remain open.
ctrl.rs/cable_input.rs carry default-false per-player CTRL state through the native
authenticated mod channel. Process it in guest_native_net before subsequent use
packets, charging the shared PER_TICK allowance exactly once. Never defer this
input behind the packet batch. Read physical_keys, not canonical action keys:
rebinding sprint must not change the opposite-side binding. Reset world/player
state on world lifecycle/death; connection identity uses a retained Arc marker.
cable_pick.rs selects only hit-tested supported centers and returns a bare part
item; optional creative BE data encodes the actual Mod BE and uses the native
component collector. saved_block_entity's vanilla canonicalizer rejects mod IDs.
part_use.rs owns the source ordered result flow; cable_actions.rs scopes the
boolean hand bridge by Game identity and RAII. A failed cable click stops with
no swing/offhand action; client SUCCESS swings, server CONSUME does not.
Keep block reactions and other native claims ahead of part use. Server success
consumes only for an actual noncreative player and awards the use statistic/
criterion. cable_prediction.rs runs in the remote module: never use native_set_block,
the server network, or the persistent owner map for a client preview. Newly
constructed client cable connections are empty until streamed. Retain server
blocks and BE snapshots under native action sequences; received BE data takes
precedence over the original preview snapshot. Reset on connection/dimension/
respawn/leave. The guest packet wrapper sends queued CableBus updates while it
owns the hub, before normal acknowledgements; do not read game.lan inside a
guest's use (the hub is temporarily outside Game). Wire occupancy fields preserve
raw unsupported side/facade admission. Full multipart prediction, settings,
security, all replacement/unload/fake-player and external multiplayer paths
remain open. Native test filtering includes nested ae2_ modules, not only direct
children of game.
adventure.rs/adventure_snbt.rs implement the original 1.21.1 saved predicate codec,
typed property ranges, numeric parsing and NBT subset comparison. The native
adapter checks the actual passed stack before claiming the item, so a denied
can_place_on is generic PASS rather than part FAIL. Client NBT lookup creates a
missing registered BE on demand through new_block_entity; it must not synthesize
an NBT mask or borrow the server's private save. Existing instances are retained.
Property domains come from AE2-owned data/ae2/source-property-domains.json, never
modified base tables. Missing domains/properties refuse that state condition.
The current matcher evaluates fresh predicates. Adventure-mode rework is
suspended for now at the user's request: keep the validated codec, matcher and
actual-hand gate working, but do not extend them. Source mutable component-object
cache, ItemStack.copy sharing, full BE save/component codecs, new/addon property
domains and complete security/fake-player lifecycle remain deferred. Do not
equate fresh oracle/world results with those remaining contracts. Native
Charger/Drive facing/spin states are now present; extending adventure
predicates beyond the validated codec is still suspended.
Charger/Drive use the source full orientation: 24 facing+spin rows replace the
dispenser template's facing/triggered rows in the engine's canonical order.
Prepared blockstates must use Horizon's x/y/z composition, not the source-only
ae2:z key; tools/block_orientation.py derives the equivalent triples from the
original BlockOrientation table. Server placement applies the source
FacingWithSpinStrategy (65-degree pitch thresholds and SpinMapping). Client
prediction still starts from the engine template state. The Certus Quartz
Wrench turns supported oriented hosts through the `c:tools/wrench` common tag,
excluding `ae2:network_tool`, using the source Direction.getClockWise rotation
and spin derivation. Never rotate vanilla blocks; a crank turn is refused unless
the new attached side is a crankable, and shift mode must not rotate. Disassembly,
permissions/claims and the network tool UI remain unported; do not present this
as the complete WrenchHook.
Physical states must stay source light_level/waterlogged (32), not invented
color/type/connection properties. CableBuilder texture names use enum names:
TRANSPARENT texture suffix is transparent, whereas the item ID prefix is fluix.
cube_builder.rs/cable_render.rs now feed cable_view.rs in the native render
library. Center-only CableBus uses 32 source light/waterlogged states. They preserve
source float32 operation order, UV flips/rotations, dense +3/4 wrapping, sprite
suffixes and smart machine-cap vertex-color retention. Quad.cull records emitter
metadata only: original cable BakedQuads are returned unculled, not by face.
RenderState expects already resolved source connection types/core/channel inputs;
The collector uses actual graph visual counts and source neighbor minima.
player_registry.rs uses source UUID/int allocation and parallel arrays. Its native
owner bridge reads/writes world_root/data/ae2_players.dat, shared by dimensions;
never use a global settings folder, UUID hash or process-only ID counter. Preserve
existing NBT fields and refuse corrupt mapping overwrites. Cable drops carry only
dismantle settings, never node power/owner or arbitrary input components. Pending's
private copied map entry is scoped to the position/transaction and cleared normally;
it must never be attached to a real stack. grid_glue::node_placed forgets the old
node before actual cell/cable placement is reconciled, including equal saved data.
Native center visuals use a wire-only transport in the same bundle. Attachment/facade
quads, packed normals/AO, full lighting and the complete source packet remain open.
Preserve FabricMC Apache-2.0 attribution for QuadEmitter.square coordinate rules.
glue.rs owns Horizon types. Only modules actually consumed by the adapter belong
in horizon-mixin.toml. keys.rs, storage.rs and cells.rs feed the native cell
tooltip/codec adapter. energy.rs/grid.rs/power.rs feed network.rs and the
grid_glue.rs world adapter (Charger + three energy-cell kinds + Drive + Controller).
controller.rs supplies source 8000 public AE/idle 3, node profile and visual rules.
block_states.rs wraps only owned Controller/energy-cell/CableBus, Charger/Drive
and Crank content builds. It expands single physical cube rows to Controller 18
state/type and finite-cell five fullness variants, replaces the dispenser
template's twelve facing/triggered rows with the source 24 facing/spin states
for Charger/Drive, and shrinks the Crank to the source six facing-only states.
Oriented rows are ordered facing major (spin minor where present), keep the
source facing-north default, and structure rotation/mirror move only facing.
The Crank may carry the source per-facing shape boxes; Charger/Drive must not.
Preserve vanilla ids and foreign shape/model references. The adapter supports
template ranges both growing and shrinking. This is not a general custom-state
API. Controller model schema requires named values; finite-cell fullness uses
integer [0,4]; Charger/Drive use facing named values and integer spin [0,3]; the
Crank uses facing only. Cells use source cube glass properties,
never dispenser behaviour/inventory. energy_cell_glue.rs feeds local stored power
to analog output and defers comparator/fullness notifications to block-ticking
chunks, preserving pending NBT. Exact TickManager/readiness and the full item/drop energy
component lifecycle remain open. Do not call external callbacks while NETWORKS is locked.
Controller status/type currently update at server post_tick; immediate shape/
ready/events, external power funnel, Network Status Menu and full lifecycle
remain open. Do not mark Controller fully ported on these tests alone.
item_energy.rs/item_energy_glue.rs feed 16 AEBasePoweredItem profiles and the two
finite EnergyCellBlockItems into Charger. Cell items have fixed block capacities/
rates, a distinct 0.00001 removal threshold and no AEBasePoweredItem green bar.
energy_cell_model.rs projects only render views to the original float-ratio item
models (20/40/60/80 percent), never the block's quarter-based fullness. Never write
that temporary block_state component into real stacks, packets, saves or hashes.
energy_cell_item.rs imports stored energy on actual placement and exports local
energy only to the first matching block drop through Pending; proxy.e is not an
item component. Live snapshot reads before player/Level drops must not restore
spent same-tick energy or overwrite a newer external NBT replacement. Real finite
cell placement forgets the old node even for identical position/kind/saved NBT;
never retain a destroyed BE's spent store for the newly placed item. General
block/chunk/command identity/readiness lifecycle is still pending. Full base
names/ownership/settings, raw saved-BE/loot-command/explosion lifecycle and exact
source tooltip formatting remain open.
charger.rs preserves local/direct-grid/overflow/refill order. Command registry hooks
add only supported component IDs, preserve vanilla types and validate energy numbers.
item_energy_view.rs implements the source green 13-pixel bar, preserving count/cooldown.
component_sync.rs lives in the native client library, sends AE2 typed component
maps through a versioned wire-only envelope and unwraps before normal decoding.
It must preserve real custom_data, nested items, original saved keys and click hashes.
inventory_codec.rs is shared by Drive/Charger for original optional ItemStacks,
count validation/default omission and source-versus-native echo selection.
Loaded positive stack counts are not the same as normal input slot limits.
inventory_signal.rs/glue feed the native Containers.signal hook only for
Drive/Charger: source 10/1 slots and slot limit one, never menu padding or a
silent fullness clamp. Air/zero are empty. Preserve the existing malformed gate;
do not present it as source recovery. Other hosts keep native signal rules.
charger_inventory.rs reconciles original inv.item0 with native Items by value;
unchanged echoes cannot restore stale contents/power. Malformed original inv stays
blocked until an explicit repair; never overwrite it from a changed native echo.
Charger slot edits flush the original format immediately; working stays transient.
Handheld actions, config reload, item energy tooltip and upgrade callbacks remain open.
mounts.rs/service.rs/drive.rs are consumed by drive_glue.rs and belong in the
native manifest. Drive currently has ten real cell slots/providers, supports
all twenty-three built-in basic/portable/creative/tool cell types, original
inv persistence and power/channel gating; its filtered ten-slot screen works,
while priority UI and special bulk actions remain open. automation*.rs enforces
the same ten-slot limits/filters for native hoppers/droppers and flushes cells
before slot mutation; a general external capability API is still pending.
slot_rules.rs is shared with native library menu policy; guest_slots.rs scopes
server rules by authoritative player/window/block identity, never by chest size.
backend.rs dispatches finite/creative inventory rules; cells.rs owns component
codecs, including 63-slot nullable config limits. upgrade_rules.rs supplies finite
host slot/card limits and per-slot counts; cells.rs reads canonical slot/item
upgrade entries. Workbench/writer/callbacks and addon upgrades remain open.
tool_filters.rs/tool_glue.rs
expand recipe/tag admission into a restricted finite backend; never load a tool
as an unrestricted basic cell. Live data reload and handheld tool actions remain open.
Injected payloads cannot use inner attributes/comments, external mod files or
include!. Helpers are tiny, uniquely named ae2_* functions.

Keep AEKey identity component-aware. Do not substitute registry strings for full
item/fluid keys. Preserve simulation/commit semantics, long counts, partial
insertion, byte rounding, type limits, partition/extraction rules and nested-cell
restrictions. Never fake ME channels, energy, crafting jobs or security permissions.

Use upstream LGPL-3.0-or-later for derived behaviour; preserve notices and licences.
Assets come from the pinned jar into ignored target/content only. Do not commit
jars, Minecraft resources, models, textures, exported sources or binaries.
Tooling adapted from horizon-create keeps tools/LICENSE-MIT.
tools/item_stack_limits.py owns the 58 built-in unstackable registrations and
audits their pinned factories/builder/inheritance. prepare_content.py sets their
limit to one and the other carried item definitions to 64, preserving other fields.
These are registration defaults, not component-specific stack overrides or full item APIs.

Verify cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings,
cargo test --locked, Python 3.12 tools/test_tooling.py and tools/upstream.py audit
--check. Horizon changes also require tools/horizon.py check against the pinned
engine. tools/item_stack_limits.py --check must match its committed source audit.
Build and isolated game verification are separate evidence, not implied
by unit tests. Log meaningful engine gaps and known deviations in docs/PARITY.md.
Do not report a partial class or a carried resource as complete feature parity.

All generated/intermediate files stay under target/. Update VALIDATION.md with
what actually ran, including limits. Do not run a game smoke while a native
stage/check/test/build can replace the client binary: the running client can
exit without a crash report. Publication and pushing require explicit user
authorization; the public horizon-ae2 creation/push is authorized above.

The user removed the quota-based automatic pause rule. Continue the full-port
goal without pausing at a remaining-usage percentage. Never reset or purchase
quota on the user's behalf.
