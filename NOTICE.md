# Notices

This is a modified native Rust port of Applied Energistics 2, not the original
Java distribution. Behaviour follows AE2 19.2.18 at the commit recorded in
upstream.lock.json. Copyright (c) 2013-2021 AlgorithmX2, TeamAppliedEnergistics
and other AE2 contributors; copyright (c) 2026 Horizon port contributors.

Derived code is distributed under LGPL-3.0-or-later. LICENSE contains both the
LGPLv3 and GPLv3 texts supplied by upstream. Upstream's API is MIT licensed;
see LICENSES/AE2-API-MIT. Key/counter/fuzzy, storage, cable-type and color API rules are adapted from that
MIT-licensed API; its copyright and permission notice are preserved there.

tools/horizon.py and tools/package.py are adapted from horizon-create and the
Horizon native-mod/Veinminer templates. Their MIT notice is tools/LICENSE-MIT.

`mixins/ae2/item_tick.rs` carries the pinned Horizon `Game::tick_items` body
with marked AE2 insertion regions. Copyright (c) 2026 Ludovic Marie and the
Horizon contributors, MIT; see LICENSES/Horizon-MIT. Original engine sources
remain unchanged. `tools/item_tick_source.py` verifies the carried body.

Upstream textures/models: copyright (c) 2020 Ridanisaurus Rid and
(c) 2013-2020 AlgorithmX2 et al, CC BY-NC-SA 3.0. Text/translations: CC0.
Guidebook click sound: EminYILDIRIM, CC BY 4.0. These assets are not bundled in
the source-only native archive. prepare_content.py prepares local content from
the pinned original jar; it is not a permission grant to redistribute assets.

Sources: upstream README.md at the pinned tag, and its LICENSE. This project is
not affiliated with or endorsed by Team Applied Energistics, Horizon, Mojang or Microsoft.

Prepared content also carries generated common `c` tags from NeoForge 21.1.169,
source revision 863802b6dfdb1d5e48c2fda093ed9d01df0f4cd9. The official
[release changelog](https://maven.neoforged.net/releases/net/neoforged/neoforge/21.1.169/neoforge-21.1.169-changelog.txt)
identifies [PR 2245](https://github.com/neoforged/NeoForge/pull/2245), whose merged
commit is pinned in upstream.lock.json. Copyright NeoForged, MinecraftForge and
their contributors; their LGPL-2.1 text is copied from that checkout into
`data/ae2/licenses/neoforge-lgpl-2.1.txt` in prepared content. Tag source hashes
and layer merges are recorded in `data/ae2/platform-tags.json`. These data files
are separate from the source-only `.hnmod`; no NeoForge JVM runtime is included.

CubeBuilder's square coordinates also adapt QuadEmitter.square from the Fabric
Renderer API embedded in AE2's appeng.thirdparty.fabric package. Copyright (c)
2016, 2017, 2018, 2019 FabricMC, Apache-2.0; see LICENSES/Fabric-Apache-2.0.
The implementation is modified/re-expressed in Rust; no Fabric runtime is bundled.
Original attribution: https://github.com/FabricMC/fabric/blob/1.21.1/fabric-renderer-api-v1/src/client/java/net/fabricmc/fabric/api/renderer/v1/mesh/QuadEmitter.java

`quad_rotation.rs` also adapts Fabric's packed normal/color and quad accessor
rules under the same Apache-2.0 notice. Its scalar quaternion-vector arithmetic
is a modified Rust expression of JOML 1.10.5, copyright (c) 2015-2022 JOML,
MIT; see LICENSES/JOML-MIT, preserved from the verified dependency archive.
The 24 immutable quaternion/direction rows are derived from AE2's original
BlockOrientation constructor with that dependency. No JOML Java runtime is
included in the mod. Minecraft winding is checked against the player's
verified reference artifact; that artifact is not distributed here.
