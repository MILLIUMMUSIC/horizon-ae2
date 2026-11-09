#!/usr/bin/env python3
"""Verify the source meteorite placement probe and cube loot in a private world.

Consumes normally prepared content unchanged. Natural structure generation,
compass discovery, source physics and external multiplayer are not covered.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]



def main():
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--content", type=Path, required=True)
    parser.add_argument(
        "--game",
        type=Path,
        default=ROOT / "target/horizon-cargo/fast/horizon-client",
    )
    args = parser.parse_args()
    parent = ROOT / "target/smoke"
    parent.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="meteorite-fallout-", dir=parent))
    game_dir = work / "game"
    game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-meteorite-fallout-{os.getpid()}.sock"
    env = {**os.environ, "HORIZON_HOME": str(work / "home"), "HORIZON_NO_AUDIO": "1"}
    processes, logs, transcript = [], [], []

    def ctl(*command):
        run = subprocess.run(
            [
                str(args.ctl.resolve()),
                "--socket",
                str(socket),
                "--timeout",
                "40",
                *map(str, command),
            ],
            env=env,
            text=True,
            capture_output=True,
            timeout=50,
        )
        data = json.loads(run.stdout)
        transcript.append({"command": command, "exit": run.returncode, "result": data})
        (work / "commands.json").write_text(json.dumps(transcript, indent=2) + "\n")
        if run.returncode:
            raise RuntimeError(f"{command}: {data}")
        return data

    def launch():
        log = (work / f"game-{len(processes) + 1}.log").open("w")
        logs.append(log)
        process = subprocess.Popen(
            [
                str(args.game.resolve()),
                "--world",
                "MeteoriteFalloutSmoke",
                "--creative",
                "--seed",
                "42",
                "--offscreen",
                "--control-socket",
                str(socket),
                "--render-distance",
                "4",
                "--width",
                "960",
                "--height",
                "640",
                "--game-dir",
                str(game_dir),
                "--resources",
                str(args.resources.resolve()),
                "--dev-mod",
                str(args.content.resolve()),
            ],
            cwd=game_dir,
            env=env,
            stdout=log,
            stderr=subprocess.STDOUT,
        )
        processes.append(process)
        deadline = time.monotonic() + 90
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"game exited: {log.name}")
            try:
                ctl("wait-ready")
                startup = Path(log.name).read_text(errors="replace")
                assert "content mods not loaded" not in startup and "using the default resources" not in startup, startup[:2000]
                return
            except (RuntimeError, subprocess.TimeoutExpired, json.JSONDecodeError):
                time.sleep(0.1)
        raise RuntimeError("game did not become ready")

    checks = []
    def inputs():
        paths = [args.game.resolve(),
                 args.content.resolve() / "data/ae2/loot_table/blocks/mysterious_cube.json",
                 args.content.resolve() / "data/ae2/tags/item/inscriber_presses.json"]
        paths += sorted((ROOT / "mixins/ae2").glob("*.rs"))
        paths += [ROOT / "mixins/ae2/horizon-mixin.toml"]
        return {str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path):
                hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    source_inputs = inputs()

    def command(line):
        return "\n".join(ctl("command", line).get("chat") or [])

    def native_command(line):
        since = ctl("events")["last"]
        ctl("chat", "/" + line)
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            events = ctl("events", "--since", since)["events"]
            answers = [event.get("text", "") for event in events
                       if event.get("event") == "chat" and "AE2 meteorite" in event.get("text", "")]
            if answers:
                return "\n".join(answers)
            ctl("wait", 2)
        raise AssertionError(f"No server probe reply: {line}; messages={ctl('messages')}")

    def probe(crater="lava", fallout="default", pure=True, lake=False,
              flawless=False, presses=True, bounds="0 -64 0 15 319 15", sea=63):
        values = [str(v).lower() for v in [pure, lake, flawless, presses]]
        answer = native_command(f"ae2meteorite 8 73 8 4 {crater} {fallout} "
                         + " ".join(values) + f" {bounds} {sea}")
        assert "probe finished" in answer, answer
        return answer

    def block(x, y, z):
        return ctl("getblock", x, y, z)["block"].split("[", 1)[0]

    def visible():
        counts = dict(ctl("inventory").get("counts") or {})
        for entity in ctl("entities", 32, "minecraft:item").get("entities") or []:
            stack = entity.get("item") or {}
            if stack.get("item"):
                item = stack["item"]
                counts[item] = counts.get(item, 0) + int(stack.get("count") or 0)
        return {item: n for item, n in counts.items() if item.startswith("ae2:") and n}

    try:
        launch()
        command("tp @s 8.5 92 8.5")
        ctl("tp", 8.5, 92, 8.5)
        for corner in [(0, 72, 0), (31, 72, 15)]:
            for _ in range(60):
                if ctl("block", *corner).get("loaded"):
                    break
                ctl("wait", 5)
            else:
                raise AssertionError(f"fixture chunk did not load: {corner}")
        command("tick freeze")
        command("fill 0 64 0 31 90 15 air")
        command("fill 0 72 0 31 72 15 stone")
        assert block(8, 72, 8) == "minecraft:stone"
        ctl("setblock", 8, 180, 8, "minecraft:stone")
        ctl("setblock", 15, 73, 15, "minecraft:bedrock")
        ctl("setblock", 16, 72, 8, "minecraft:diamond_block")
        ctl("setblock", 15, 69, 8, "minecraft:oak_stairs[facing=east,half=top,shape=straight,waterlogged=true]")
        ctl("setblock", 15, 68, 8, "minecraft:air")
        command('summon minecraft:item 8.5 78 8.5 {Item:{id:"minecraft:diamond",count:1},PickupDelay:32767s}')
        command('summon minecraft:item 24.5 78 8.5 {Item:{id:"minecraft:emerald",count:1},PickupDelay:32767s}')
        answer = probe()
        ctl("wait", 2)
        assert block(8, 73, 8) == "ae2:mysterious_cube"
        assert block(10, 73, 8) == "ae2:sky_stone_block"
        assert block(13, 72, 8) == "minecraft:lava"
        assert block(8, 180, 8) == "minecraft:air", "crater must reach the level top"
        assert block(15, 73, 15) == "minecraft:bedrock"
        assert block(16, 72, 8) == "minecraft:diamond_block"
        copied = {"state": ctl("getblock", 15, 68, 8)["block"]}
        copied["block"] = copied["state"].split("[", 1)[0]
        assert copied["block"] == "minecraft:oak_stairs", copied
        copied_state = copied.get("state") or ""
        assert all(value in copied_state for value in ["facing=east", "half=top", "waterlogged=true"]), copied
        entities = ctl("entities", 64, "minecraft:item").get("entities") or []
        items = [e["item"]["item"] for e in entities if e.get("item")]
        assert "minecraft:diamond" not in items and "minecraft:emerald" in items, items
        nbt = command("data get block 8 73 8")
        assert "ae2:mysterious_cube" in nbt, nbt
        checks.append({"source_phases": answer, "full_state_copy": copied,
                       "bedrock_and_slice_preserved": True, "item_discard": items,
                       "cube_block_entity": nbt})
        # A one-column slice excludes the center. The source must not place a
        # cube there or replace unrelated columns, even with presses enabled.
        for kind, expected in [("none", "stone"), ("normal", "air"),
                               ("lava", "lava"), ("obsidian", "obsidian"),
                               ("water", "water"), ("snow", "snow_block"), ("ice", "ice")]:
            ctl("setblock", 15, 72, 8, "minecraft:stone")
            probe(crater=kind, bounds="15 -64 8 15 319 8")
            assert block(15, 72, 8) == "minecraft:" + expected, (kind, block(15, 72, 8))
            checks.append({"crater": kind, "filler": expected})
        # Config flags remain separate: flawless-only still performs nextInt(1)
        # and places the source flawless block; disabling presses leaves air.
        command("fill 0 64 0 15 90 15 air")
        probe(crater="none", flawless=True, presses=False)
        assert block(8, 73, 8) == "minecraft:air"
        assert block(8, 72, 8) == "ae2:flawless_budding_quartz"
        checks.append({"flawless_only": True, "spawn_presses": False})
        for mode in ["none", "default", "sand", "terracotta", "ice_snow"]:
            command("fill 15 64 8 15 90 8 air")
            ctl("setblock", 15, 72, 8, "minecraft:stone")
            probe(crater="normal", fallout=mode, pure=False, lake=True,
                  bounds="15 -64 8 15 319 8", sea=76)
            assert block(15, 75, 8) == "minecraft:water", (mode, block(15, 75, 8))
            checks.append({"fallout": mode, "crater_lake_surface": "minecraft:water"})
        # Invalid and unloaded requests must refuse before the first write.
        before = block(10, 73, 8)
        for line in ["ae2meteorite 8 73 8 NaN lava default true false false true 0 -64 0 15 319 15 63",
                     "ae2meteorite 8 73 8 4 lava default true false false true 20000 -64 0 20015 319 15 63"]:
            answer = native_command(line)
            assert "probe refused" in answer, answer
            assert block(10, 73, 8) == before
            checks.append({"refused": answer})
        # Original non-silk loot contains all four tag entries and the guide;
        # silk returns the cube only. No smoke-time loot/content rewrites.
        command("fill 0 64 0 31 90 15 air")
        command("fill 0 72 0 31 72 15 stone")
        command("kill @e[type=minecraft:item]")
        command("tick unfreeze")
        command("tp @s 8.5 73 6.8")
        ctl("tp", 8.5, 73, 6.8)
        command("gamemode survival")
        for silk in [False, True]:
            command("kill @e[type=minecraft:item]")
            command("clear @s")
            ctl("setblock", 8, 73, 8, "ae2:mysterious_cube")
            command("item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1")
            ctl("select", 1)
            ctl("wait", 3)
            if silk:
                command("enchant @s minecraft:silk_touch 1")
            ctl("wait", 3)
            result = ctl("break", 8, 73, 8)
            ctl("wait", 3)
            expected = {"ae2:mysterious_cube": 1} if silk else {
                "ae2:calculation_processor_press": 1, "ae2:engineering_processor_press": 1,
                "ae2:logic_processor_press": 1, "ae2:silicon_press": 1, "ae2:guide": 1}
            actual = visible()
            assert actual == expected, (silk, actual, expected, result)
            checks.append({"silk_touch": silk, "cube_drops": actual})
        command("kill @e[type=minecraft:item]")
        command("clear @s")
        # The Level caller has no tool, even when the player just used Silk
        # Touch. It must use the passed loot context, not the player's hand.
        command("item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1")
        ctl("select", 1)
        command("enchant @s minecraft:silk_touch 1")
        ctl("setblock", 8, 73, 8, "ae2:mysterious_cube")
        command("setblock 8 73 8 minecraft:air destroy")
        ctl("wait", 3)
        level_drops = visible()
        expected = {"ae2:calculation_processor_press": 1, "ae2:engineering_processor_press": 1,
                    "ae2:logic_processor_press": 1, "ae2:silicon_press": 1, "ae2:guide": 1}
        assert level_drops == expected, level_drops
        checks.append({"level_destroy_without_tool": level_drops})
        ctl("setblock", 8, 73, 8, "ae2:mysterious_cube")
        ctl("save")
        ctl("quit")
        processes[-1].wait(timeout=30)
        socket.unlink(missing_ok=True)
        launch()
        assert block(8, 73, 8) == "ae2:mysterious_cube"
        restored = command("data get block 8 73 8")
        assert "ae2:mysterious_cube" in restored, restored
        checks.append({"restart": "cube block and tracking block entity persisted"})
        assert inputs() == source_inputs, "native/content/source inputs changed during the smoke"
        report = {"passed": True, "logs": str(work), "checks": checks, "input_sha256": source_inputs,
                  "limits": ["explicit placement-probe inputs, not natural structure admission/placement or biome selection",
                             "integrated server; external multiplayer and compass lifecycle remain open",
                             "native neighbor/physics behavior is separate from Java controlled-world traces"]}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, indent=2), flush=True)
        ctl("quit")
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
        for log in logs:
            log.close()
        socket.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
