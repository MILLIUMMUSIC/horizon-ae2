# Current port checkpoint

The goal remains the full 1:1 port of AE2 19.2.18 to Horizon. Work uses no
subagents. The source inventory contains 1,429 Java files; partial coverage is
not a gameplay completion percentage. See `upstream-inventory.json` for current
per-file status and `PARITY.md` for unresolved behavior.

## Current work

The native center-only CableBus checkpoint integrates all 32 source physical
states, five cable types and 17 colors, graph-derived visuals, original atlas
sources, dynamic outline/raycast, contextual movement/piston shapes, and a
wire-only native client transport. All 85 cable items place real center parts, with survival/creative consumption,
entity obstruction/rollback, named drops, waterlogging and bucket interactions.
An item-placed cable chain transfers real ME power. Original center/node data,
custom names and the world-wide player-ID map survive restart. Full CableBus and full
mod parity remain unfinished; the limits below are still completion requirements.
The input layer synchronizes a configurable physical opposite-side key before
dependent use packets. Center Pick Block returns the actual bare cable item,
with optional creative block-entity data collected from the supported Mod host.
Center placement predicts a real local part, checks entities, preserves source
interaction results and uses native sequences to settle authoritative blocks
and host data. A client success swings once; a failure stops before the off hand. The generic
adventure stack gate instead returns PASS on denied use, checks the actual hand
and evaluates original saved block/state/NBT predicates. Adventure-mode rework
is suspended for now; the validated gate remains in place without extension.
Charger and Drive now use the source 24 facing/spin state product: the dispenser
template rows are replaced, prepared models use the equivalent Horizon rotation
triples, and server placement applies the source 65-degree/SpinMapping
orientation. The Certus Quartz Wrench also turns supported oriented hosts
(Charger, Drive, Crank) around the clicked face through the common wrench tag,
preserving block-entity data and restart. The Wooden Crank now also uses the
source six facing-only states and its per-facing shape boxes, replacing the
dispenser template's triggered dimension. Client-predicted orientation, crank
placement/survival/animation, shift-disassembly, permissions and the network
tool UI remain open.

Inscriber recipe processing and automation are now native. Its callback state
machine matches four verbatim Java methods in 4,504 scenarios, 261,592 callbacks
and 4,504 initial sleeping queries. All 15 built-in recipes retain their source
inputs, result counts and press/inscribe consumption rules. The source machine
menu, dynamic name presses, complete upgrades/drop callbacks and render packets
remain required work.

## Verified foundations

| Layer | Evidence and current boundary |
| --- | --- |
| Cable geometry | 24,405 Java cases, 467,940 quads, 9,358,800 float fields; GPU/world collector, attachments/facades and full lighting remain separate |
| Cable node data | 20,064 Java equality cases; real typed NBT roundtrip and native graph-buffer identity tests; explicit source load/readiness events remain open |
| Cable profiles | 85 type/color profiles, source flags and channel visuals; 278,956 Java cases; dense trunk and normal bottleneck tested in the binding |
| Cable input | 11,028 Java boolean-packet cases; native per-world/sender state and hit-selected center clone; full multipart behavior and external multiplayer remain open |
| Charger/Drive orientation | 105,489 original-Java placement/direction cases; 24 native facing+spin states, inherited identity structure hooks, equivalent Horizon blockstate triples, source placement and wrench turns; client-predicted orientation, wrench disassembly, permissions and machine lifecycle remain open |
| Wooden Crank states | Six source facing-only states, per-facing outline/collision boxes, inherited identity structure hooks and wrench turns; placement orientation/survival, animation and synchronization remain open |
| Adventure predicates | 73,399 original-Java comparisons: saved predicates, SNBT, NBT comparison and integer parsing; actual-hand native gate. Rework is suspended for now; its deferred cache and full BE codecs are unchanged |
| Part use | 10,320 Java placement-result/order/consumption cases; supported center client preview and server correction; full multipart/settings/security and source event lifecycle remain open |
| Finite cell items | Fixed power profiles, original item-model thresholds, Charger support, placement/first matching drop energy transfer; full base APIs and all loot paths remain open |
| Energy cells | Original fullness states/models, local comparator formula, deferred ticking-chunk notifications, grid power and persistence; complete event/TickManager lifecycle remains open |
| Controller | Public 8,000 AE store, 3 AE/tick idle, source flags/validation/channel rules, 18 states/models, conflict/restart world checks; menus/capabilities/full lifecycle remain open |
| Drive | Ten real cell slots, all 23 built-in backend profiles, source inventory, power/channel gating, filtered screen, automation and restart; full menu/capability/render lifecycle remains open |
| Charger | Recipe and powered-item work, original inventory, automation, grid/local power and item energy display/sync; full-orientation states and placement; complete geometry/effects/config/capabilities remain open |
| Inscriber | Four original slots, 15 carried recipes, source callback state machine, private/grid/crank power, capacities, speed-card saved data, sided automation, auto-export, configured comparator and 48 wet/oriented states; source menu, name-press components and full lifecycle remain open |
| Keys/storage/services | Component-aware identities, finite/creative/restricted cells, byte/type rules, routing/mounts and caches; full registry recovery, callbacks, consumers and addon APIs remain open |
| Certus quartz loot | Original carried bud/cluster tables, source missing-entity refusal and matching stage templates; growth/decay and complete explosion actor context remain open |
| Content/tooling | Pinned original assets, 461 NeoForge tags, 58 unstackable registrations, deterministic source packaging and isolated builds; carried content does not imply implemented mechanics |

Current validation: 144 standalone Rust tests, 203 native game tests, 19 native
library tests, 7 native item-crate tests and 15 Python tests. Native check/build and the isolated CableBus
world checks pass, including all 85 center Pick Block items, Ctrl data, packet
ordering, configured keys and restart. The earlier finite/creative power and
restart regression also passes.
The center-result world check passes client/server prediction, obstruction,
offhand failure, acknowledgement and interleaved break cases; placement and
input regressions pass again with the current adapter.
The adventure world check passes 24 actual-hand/client/server scenarios,
including generic PASS/offhand fallback, enum/integer domains, original compound/
SNBT masks and real Energy Cell/chest entities. It exercises the registered
client BE constructor on demand; complete source BE defaults/codecs and mutable
component-object cache remain required. The earlier adventure checkpoint also includes its
14-case survival/creative center-action regression.
See `VALIDATION.md` for evidence and the exact boundaries of these runs.

## Next required work

Complete multipart client prediction, settings/security and all lifecycle paths,
remaining direct collision/support/fluid/light paths, full
CableBus part interactions, partial removal, facades, full source visual packets,
and source event/readiness model. Finish Charger/Drive client-prediction
orientation, wrench disassembly/permissions, crank placement/animation and their
complete menu/model behavior. Continue
with the remaining Inscriber menu/name-press/upgrades lifecycle, real storage
hosts/terminal parts and menus, fluid-transform entity lifecycle, quartz growth,
meteorite admission/press loot, capabilities, menu/render gaps, crafting,
security, wireless/spatial services, tools, world content, addons, and full
survival/multiplayer validation. The full plan is unchanged.

Adventure-mode rework is suspended for now at the user's request. Keep the
validated decoder, matcher and actual-hand gate working, but do not extend the
deferred mutable component-object cache, ItemStack.copy sharing, full BE
save/default/component codecs or permission/security paths. Typed property
domains for new/addon blocks are deferred with that work.

All sources/builds/worlds used for verification live under ignored `target/`.
Original repositories, installed resources, player saves, and launcher settings
are not modified. Repository documentation is English-only.
