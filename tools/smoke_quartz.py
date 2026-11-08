#!/usr/bin/env python3
"""Check original certus bud/cluster loot in a private native world.

Verifies plain, Silk Touch and Fortune player breaks and support-loss refusal.
Uses prepared content unchanged; quartz growth and worldgen are separate work.
"""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
CLUSTER = (8, 73, 8)
CRYSTAL = "ae2:certus_quartz_crystal"


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    parser.add_argument("--content", type=Path, default=ROOT / "target/content/ae2")
    args = parser.parse_args()
    content = args.content.resolve()
    if not (content / "ae2-recipe-map.json").is_file():
        parser.error("--content must name a prepared AE2 content mod")
    parent = ROOT / "target/smoke"
    parent.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="quartz-", dir=parent))
    game_dir = work / "game"
    game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-quartz-{os.getpid()}.sock"
    env = {**os.environ, "HORIZON_HOME": str(work / "home"), "HORIZON_NO_AUDIO": "1"}
    processes, logs, transcript = [], [], []

    def ctl(*command):
        run = subprocess.run(
            [str(args.ctl.resolve()), "--socket", str(socket), "--timeout", "40", *map(str, command)],
            env=env, text=True, capture_output=True, timeout=50,
        )
        data = json.loads(run.stdout)
        transcript.append({"command": command, "exit": run.returncode, "result": data})
        (work / "commands.json").write_text(json.dumps(transcript, indent=2) + "\n")
        if run.returncode:
            raise RuntimeError(f"{command}: {data}")
        return data

    def launch():
        log = (work / "game.log").open("w")
        logs.append(log)
        process = subprocess.Popen(
            [str(args.game.resolve()), "--world", "QuartzSmoke", "--creative", "--seed", "42",
             "--offscreen", "--control-socket", str(socket), "--render-distance", "3",
             "--width", "960", "--height", "640", "--game-dir", str(game_dir),
             "--resources", str(args.resources.resolve()), "--dev-mod", str(content)],
            cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT,
        )
        processes.append(process)
        deadline = time.monotonic() + 40
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"game exited: {log.name}")
            try:
                ctl("wait-ready")
                return
            except (RuntimeError, subprocess.TimeoutExpired, json.JSONDecodeError):
                time.sleep(0.1)
        raise RuntimeError("game did not become ready")

    def visible():
        counts = dict(ctl("inventory").get("counts") or {})
        for entity in ctl("entities", 16, "minecraft:item").get("entities") or []:
            stack = entity.get("item") or {}
            if stack.get("item"):
                item = stack["item"]
                counts[item] = counts.get(item, 0) + int(stack.get("count") or 0)
        return {item: count for item, count in counts.items()
                if item.startswith("ae2:") and count}

    def setup(block, enchantment=None):
        ctl("command", "kill @e[type=minecraft:item]")
        ctl("command", "clear @s")
        ctl("setblock", 8, 72, 8, "minecraft:stone")
        ctl("setblock", *CLUSTER, block + "[facing=up,waterlogged=false]")
        ctl("command", "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1")
        ctl("select", 1)
        ctl("wait", 3)
        if enchantment:
            ctl("command", f"enchant @s {enchantment[0]} {enchantment[1]}")
        ctl("wait", 3)
        assert ctl("block", *CLUSTER)["block"] == block
        assert not visible(), "each case must start without previous AE2 drops"

    try:
        launch()
        ctl("tp", 8.5, 82, 5.5)
        for _ in range(60):
            if ctl("block", *CLUSTER).get("loaded"):
                break
            ctl("wait", 5)
        else:
            raise AssertionError("fixture chunk did not load")
        ctl("command", "fill 4 72 4 12 72 12 stone")
        ctl("command", "fill 4 73 4 12 78 12 air")
        assert ctl("getblock", 8, 72, 8)["block"] == "minecraft:stone"
        ctl("tp", 8.5, 73, 6.8)
        ctl("command", "gamemode survival")
        ctl("wait", 5)
        checks = []
        buds = ["ae2:small_quartz_bud", "ae2:medium_quartz_bud", "ae2:large_quartz_bud"]
        for block in ["ae2:quartz_cluster", *buds]:
            for silk in [False, True]:
                setup(block, ("minecraft:silk_touch", 1) if silk else None)
                result = ctl("break", *CLUSTER)
                ctl("wait", 3)
                expected = {block: 1} if silk else (
                    {CRYSTAL: 4} if block == "ae2:quartz_cluster"
                    else {"ae2:certus_quartz_dust": 1})
                actual = visible()
                assert actual == expected, (block, silk, expected, actual, result)
                checks.append({"block": block, "silk_touch": silk, "drops": actual})
        setup("ae2:quartz_cluster", ("minecraft:fortune", 3))
        result = ctl("break", *CLUSTER)
        ctl("wait", 3)
        actual = visible()
        assert set(actual) == {CRYSTAL} and 4 <= actual[CRYSTAL] <= 7, (actual, result)
        checks.append({"block": "ae2:quartz_cluster", "fortune": 3, "drops": actual})
        for block in ["ae2:quartz_cluster", *buds]:
            setup(block)
            ctl("setblock", 8, 72, 8, "minecraft:air")
            ctl("wait", 5)
            assert ctl("getblock", *CLUSTER)["block"] == "minecraft:air"
            actual = visible()
            assert not actual, (block, actual)
            checks.append({"block": block, "support_removed": True, "drops": actual})
        report = {"passed": True, "logs": str(work), "checks": checks,
                  "limits": ["integrated server only; independent multiplayer is not covered",
                             "natural quartz growth/decay and full world generation are not implemented",
                             "explosion source-entity semantics and AE2 loot-mine commands remain open"]}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, indent=2))
        ctl("quit")
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=10)
        for log in logs:
            log.close()
        socket.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
