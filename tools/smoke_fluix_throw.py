#!/usr/bin/env python3
"""Throw charged certus, redstone, and nether quartz into water.

The fluix crystals recipe treats a missing circumstance as the
minecraft:water fluid tag and yields two ae2:fluix_crystal. The item-entity
tick waits until the charged certus has stayed in that fluid for more than
60 ticks, then consumes one of each ingredient.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
ITEMS = (
    "ae2:charged_certus_quartz_crystal",
    "minecraft:redstone",
    "minecraft:quartz",
)
# ItemEntityMixin transforms only when ae2_transformTime > 60, after the fall.
SETTLE_TICKS = 200


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument(
        "--game",
        type=Path,
        default=ROOT / "target/horizon-cargo/fast/horizon-client",
    )
    args = parser.parse_args()
    parent = ROOT / "target/smoke"
    parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="fluix-throw-", dir=parent))
    game_dir = work / "game"
    game_dir.mkdir()
    socket = work / "control.json"
    env = {**os.environ, "HORIZON_HOME": str(work / "home"), "HORIZON_NO_AUDIO": "1"}
    content = ROOT / "target/content/ae2"
    processes, logs, transcript = [], [], []

    def ctl(*command):
        run = subprocess.run(
            [
                str(args.ctl.resolve()),
                "--socket",
                str(socket),
                "--timeout",
                "30",
                *map(str, command),
            ],
            env=env,
            text=True,
            capture_output=True,
            timeout=40,
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
            [
                str(args.game.resolve()),
                "--world",
                "FluixThrowSmoke",
                "--creative",
                "--seed",
                "42",
                "--offscreen",
                "--control-socket",
                str(socket),
                "--render-distance",
                "3",
                "--width",
                "960",
                "--height",
                "640",
                "--game-dir",
                str(game_dir),
                "--resources",
                str(args.resources.resolve()),
                "--dev-mod",
                str(content),
            ],
            cwd=game_dir,
            env=env,
            stdout=log,
            stderr=subprocess.STDOUT,
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
        # Open 1x1 shaft so the three throws fall into the same water column.
        ctl("command", "fill 7 72 7 9 72 9 stone")
        ctl("command", "fill 7 73 7 9 80 9 glass")
        ctl("command", "fill 8 73 8 8 80 8 air")
        ctl("command", "fill 8 73 8 8 76 8 minecraft:water")
        ctl("tp", 8.5, 82, 8.5)
        ctl("look-at", 8.5, 74.2, 8.5)
        ctl("command", "clear @s")
        for slot, item in enumerate(ITEMS):
            ctl("command", f"item replace entity @s hotbar.{slot} with {item} 1")
            ctl("select", slot + 1)
            ctl("look-at", 8.5, 74.2, 8.5)
            ctl("wait", 1)
            ctl("drop")
        ctl("tp", 8.5, 82, 24.5)
        ctl("wait", SETTLE_TICKS)
        ctl("tp", 8.5, 75, 8.5)
        ctl("wait", 40)
        found = ctl("inventory")["counts"].get("ae2:fluix_crystal", 0)
        if found < 2:
            nearby = ctl("entities", 24, "item")
            raise AssertionError(
                f"expected two fluix crystals, inventory={found}, entities={nearby}"
            )
        report = {"passed": True, "logs": str(work), "fluix": found}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"PASS: water formed {found} fluix crystals; logs: {work}", flush=True)
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
