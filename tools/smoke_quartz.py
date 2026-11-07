#!/usr/bin/env python3
"""Break an ae2:quartz_cluster with an unenchanted pickaxe.

A survival break without Silk Touch must put exactly four
ae2:certus_quartz_crystal into the break drops or the player inventory.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
PREPARED = Path("/home/cachyfire/.ao/data/worktrees/horizon-ae2/horizon-ae2-2/target/content/ae2")
CLUSTER = (8, 73, 8)
CRYSTAL = "ae2:certus_quartz_crystal"


def ensure_content():
    content = ROOT / "target/content/ae2"
    if not (content / "ae2-recipe-map.json").is_file():
        if content.exists():
            shutil.rmtree(content)
        shutil.copytree(PREPARED, content, symlinks=True)
    budding = content / "data/ae2/horizon/block/flawless_budding_quartz.json"
    budding.write_text(json.dumps({"template": "minecraft:budding_amethyst"}) + "\n")
    for name in ("small_quartz_bud", "medium_quartz_bud", "large_quartz_bud", "quartz_cluster"):
        path = content / "data/ae2/horizon/block" / f"{name}.json"
        path.write_text(json.dumps({
            "template": "minecraft:amethyst_cluster",
            "properties": {"requires_tool": False},
        }) + "\n")
    return content


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    args = parser.parse_args()
    content = ensure_content()
    parent = ROOT / "target/smoke"
    parent.mkdir(exist_ok=True)
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

    try:
        launch()
        ctl("tp", CLUSTER[0] + 0.5, CLUSTER[1], CLUSTER[2] - 2.5)
        for _ in range(30):
            if ctl("block", *CLUSTER).get("loaded"):
                break
            ctl("wait", 5)
        else:
            raise AssertionError(f"cluster chunk did not load: {ctl('block', *CLUSTER)}")
        ctl("command", "fill 4 72 4 12 72 12 stone")
        ctl("command", "fill 4 73 4 12 78 12 air")
        ctl("setblock", *CLUSTER, "ae2:quartz_cluster[facing=up,waterlogged=false]")
        placed = None
        for _ in range(40):
            placed = ctl("block", *CLUSTER)
            if placed.get("block") == "ae2:quartz_cluster":
                break
            ctl("wait", 5)
        else:
            below = ctl("block", CLUSTER[0], CLUSTER[1] - 1, CLUSTER[2])
            raise AssertionError(f"cluster was not placed: {placed}; below: {below}")
        ctl("command", "gamemode survival")
        ctl("command", "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1")
        held = None
        for _ in range(20):
            held = ctl("select", 1).get("held")
            if isinstance(held, dict) and held.get("item") == "minecraft:diamond_pickaxe":
                break
            ctl("wait", 2)
        else:
            raise AssertionError(f"diamond pickaxe was not in the hotbar: {held}")
        ctl("tp", CLUSTER[0] + 0.5, CLUSTER[1], CLUSTER[2] - 1.2)
        ctl("wait", 5)
        broken = ctl("break", *CLUSTER)

        def reported():
            drops = int((broken.get("drops") or {}).get(CRYSTAL, 0))
            picked = int((broken.get("picked_up") or {}).get(CRYSTAL, 0))
            return drops + picked

        def visible():
            inventory = int((ctl("inventory").get("counts") or {}).get(CRYSTAL, 0))
            ground = 0
            entities = ctl("entities", 16, "minecraft:item").get("entities") or []
            for entity in entities:
                stack = entity.get("item") or {}
                if stack.get("item") == CRYSTAL:
                    ground += int(stack.get("count") or 0)
            return inventory + ground

        count = max(reported(), visible())
        for _ in range(40):
            if count == 4:
                break
            ctl("wait", 5)
            count = max(reported(), visible())
        if count != 4:
            raise AssertionError(
                f"cluster put {count} certus quartz crystals in drops or inventory, source loot is 4: {broken}"
            )
        report = {"passed": True, "logs": str(work), "cluster": CLUSTER, "crystals": count}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"PASS: harvested 4 {CRYSTAL}; logs: {work}", flush=True)
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
