#!/usr/bin/env python3
"""Place one meteorite and require its shell, cube, and a crater or fallout block.

Exits 0 only when the center is the mysterious cube, a neighbor is sky stone,
and some block in the dug bowl is lava or another crater/fallout block.
Does not add a press recipe.
"""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
PROOF = {
    "minecraft:lava",
    "minecraft:obsidian",
    "minecraft:cobblestone",
    "minecraft:dirt",
    "minecraft:gravel",
    "minecraft:grass_block",
    "minecraft:glass",
    "minecraft:sand",
    "minecraft:terracotta",
    "minecraft:snow",
    "minecraft:snow_block",
    "minecraft:ice",
    "minecraft:magma_block",
}


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
        log = (work / "game.log").open("w")
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
        ctl("command", "fill 0 72 0 20 72 20 stone")
        ctl("command", "fill 0 73 0 20 90 20 air")
        ctl("tp", 8.5, 73, 2.5)
        ctl("command", "ae2meteorite 8 73 8 4")
        ctl("wait", 5)
        center = ctl("block", 8, 73, 8)["block"]
        shell = ctl("block", 10, 73, 8)["block"]
        proof = None
        seen = []
        for x, y, z in ((13, 72, 8), (14, 72, 8), (8, 72, 14), (2, 72, 8)):
            block = ctl("block", x, y, z)["block"]
            seen.append({"x": x, "y": y, "z": z, "block": block})
            if block in PROOF:
                proof = seen[-1]
                break
        if center != "ae2:mysterious_cube" or shell != "ae2:sky_stone_block" or proof is None:
            raise AssertionError(
                f"meteorite center={center} shell={shell} crater_or_fallout={proof} seen={seen}"
            )
        report = {
            "passed": True,
            "logs": str(work),
            "center": center,
            "shell": shell,
            "crater_or_fallout": proof,
        }
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(
            f"PASS: cube, sky stone, and {proof['block']} at "
            f"{proof['x']} {proof['y']} {proof['z']}; logs: {work}",
            flush=True,
        )
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
