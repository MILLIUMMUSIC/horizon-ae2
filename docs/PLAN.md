# Full AE2 port plan

The target is the complete pinned AE2 release with 1:1 behavior on Horizon.
Temporary implementation limits do not redefine the specification. Source
coverage is tracked in `upstream-inventory.json`; neither a partial class nor
carried assets count as a complete feature.

## Architecture

Engine-independent Rust rules cover keys, inventories, power, topology, storage,
recipes, and eventually crafting/security. Native adapters own authoritative
world access, identity/readiness, chunks/dimensions, components, messages, input,
rendering, and saves. The original Java code remains the specification.

Sources, assets, tools, and platform tags are pinned independently. Work only in
this repository and owned ignored exports. Preserve component-aware key identity,
long counts, byte rounding, simulation/commit, overflow, callbacks, partitions,
nesting restrictions, energy conservation, channels, and permissions.

## Completion requirements

1. **Reproducible toolchain and delivery.** Source/jar/engine pins, inventory,
   licensed content preparation, deterministic native/data packages, real launcher
   installation/rebuild, and isolated game verification. Joint Create planning
   alone does not prove coexistence in a running game.
2. **Keys, components, and persistence.** Item/fluid keys, counters, fuzzy matching,
   generic stacks, upgrade inventories, registry/component codecs, migrations,
   missing-content recovery, creative/portable/tool cells, Cell Workbench,
   and all save/load and nesting paths.
3. **ME graph and multipart hosts.** Grid/node/connection/service events, colors,
   cable bus parts/facades, chunk boundaries, merge/split, ad-hoc and controller
   channels, dense/smart cables, multiblocks, ownership, readiness, security,
   visual indicators, priorities, and exact lifecycle ordering.
4. **Energy and machines.** Watchers/overlays/config, multipliers and rates,
   Energy Acceptor, cells, generators, Vibration Chamber, Charger, Inscriber,
   upgrades, inventories/sides, offline behavior, power capabilities, effects,
   and all supported recipes. The capability layer must permit Create integration.
5. **Storage and terminals.** Complete Drive, ME Chest, storage buses, public/private
   caches, priorities, view cells, terminal menus/search/sorting, item/fluid
   interactions, storage monitor, handheld terminals, and addon providers.
6. **Autocrafting.** Patterns, Pattern Provider, Molecular Assembler, crafting CPUs,
   planning/calculation, storage reservation, simulation, dispatch, completion,
   cancellation, failures, restart, crafting links, and terminal interfaces.
7. **Other networks and services.** P2P, tunnels, quantum bridge, wireless access,
   security terminals/biometric cards, spatial I/O, formation/annihilation planes,
   interfaces, import/export buses, level emitters, and redstone settings.
8. **Items, world content, and integration.** Tools and weapons, painting, batteries,
   wireless devices, upgrades/configuration, disassembly, creative variants,
   world generation, growth, entities, explosions, achievements, guidebook,
   audio/animation/particles, rendering, tags, recipes, addons, and platform APIs.
9. **Full validation and maintenance.** Java/NeoForge world-level evidence, complete
   survival progression, chunk/dimension/unload/restart stress, multiplayer,
   coexistence with Create, performance, resource reload, compatibility, and
   release/install documentation.

## Current connection layer: CableBus

Cable profiles, center NBT, owner-ignoring node-data comparison, and cable-only
quad construction have source-derived implementations and differential tests.
Native center rendering, item placement/drop, buckets and original player-ID
arrays are implemented with isolated world checks. Complete multipart host and
source lifecycle integration remain the current work. The center input layer
includes a configurable physical opposite-side key, ordered server synchronization
and hit-selected Pick Block. Center placement has source interaction results,
client preview and sequence-based server correction; full multipart prediction
and complete settings/security remain required. The adventure stack gate uses
the actual hand, original saved predicates, typed property domains and NBT
matching. Adventure-mode rework is suspended for now at the user's request;
the existing validated gate stays in place, while component-object cache
identity and complete BE codecs/lifecycle are deferred. Keep the original 32
`light_level`/`waterlogged` states. Color, type, connections, and channels belong
to parts and their client state, not invented block properties.

Required native paths include outline/raycast, contextual movement, piston and
other direct collision queries, block support/occlusion, chunk/light updates,
renderer/atlas access, client synchronization, and source selection/partial break.
A static glass/core shape cannot replace dynamic connected cable geometry.

`CubeBuilder` and `CableBuilder` preserve quad order, float32 operations, UV
flips/rotations, fullbright layers, dense channel groups, and source vertex-color
state. Their emitter cull metadata is not ready-model face culling. The source
uses `transparent` texture suffixes but `fluix` item prefixes.

The world collector must resolve neighbor connection types through the source
variant/size minimum, supply real graph channels/power, identify adjacent cable
buses, and handle forced cores and attachment requirements. Side parts need their
own grid nodes/services; one center node must not impersonate an entire host.

Full part/facade registries, visual packets, direct recoloring, source explicit
BE-load events, readiness/ownership, facade collision, partial removal, drops,
fluid/light interactions, and all lifecycle paths remain completion requirements.

## Evidence discipline

Record source paths/hashes and exact transferred methods, then test boundaries
and state transitions. Compare against executable original Java where practical.
Identify every stub and excluded scope. Verify actual native engine integration
and isolated game behavior separately. A feature is complete only when behavior,
client state, persistence, edge cases, and server authority all match.

All repository documentation must be written in English. Keep documents focused
on the current project contract and evidence; retain machine-readable reports
for detailed historical results.
