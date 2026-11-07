#!/usr/bin/env python3
"""ME Chest keeps its local buffer when alone and uses the cable network when joined.

The connected chest has no local AE. Diamonds go into the drive cell and come
back out through the chest. A separate unconnected chest still uses its 500 AE buffer.
The script fails if a right-click leaves the client on playing.
"""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def ensure_content():
    content = ROOT / "target/content/ae2"
    block = content / "data/ae2/horizon/block/chest.json"
    block.parent.mkdir(parents=True, exist_ok=True)
    block.write_text(json.dumps({
        "template": "minecraft:dispenser",
        "properties": {"light": 0, "hardness": 2.2},
    }, indent=2) + "\n")
    states = {
        "variants": {
            face: {"model": "ae2:block/chest/base"}
            for face in ("down", "up", "north", "south", "west", "east")
        }
    }
    (content / "assets/ae2/blockstates/chest.json").write_text(json.dumps(states, indent=2) + "\n")
    return content


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    args = parser.parse_args()
    parent = ROOT / "target/smoke"
    parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="me-network-", dir=parent))
    game_dir = work / "game"
    game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-me-net-{os.getpid()}.sock"
    env = {**os.environ, "HORIZON_HOME": str(work / "home"), "HORIZON_NO_AUDIO": "1"}
    content = ensure_content()
    processes, logs, transcript = [], [], []
    local = (4, 73, 7)
    chest = (8, 73, 7)
    drive = (8, 73, 9)
    cell = "ae2:item_storage_cell_1k"

    def ctl(*command):
        run = subprocess.run(
            [str(args.ctl.resolve()), "--socket", str(socket), "--timeout", "30", *map(str, command)],
            env=env, text=True, capture_output=True, timeout=40,
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
            [str(args.game.resolve()), "--world", "MeNetwork", "--creative", "--seed", "42",
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

    def nbt(pos):
        return "\n".join(ctl("command", f"data get block {' '.join(map(str, pos))}")["chat"])

    def slot_of(item_id):
        inv = ctl("inventory")
        return next(slot["slot"] for slot in inv["hotbar"] + inv["main"] if slot.get("item") == item_id)

    def open_block(pos):
        if ctl("screen")["screen"] != "playing":
            ctl("key", "escape", "down")
            ctl("key", "escape", "up")
            ctl("wait", 5)
        ctl("interact", *pos)
        ctl("wait", 5)
        screen = ctl("screen")
        if screen["screen"] == "playing":
            raise AssertionError(
                "right-click left the client on playing: " + json.dumps(screen)
            )
        if screen["screen"] != "container":
            raise AssertionError(screen)

    try:
        launch()
        ctl("command", "fill 2 72 2 12 72 12 stone")
        ctl("command", "fill 2 73 2 12 78 12 air")
        ctl("tp", 4.5, 73, 4.5)
        ctl("command", "clear @s")
        ctl("setblock", *local, "ae2:chest[facing=north]")
        ctl("command", f"data merge block {local[0]} {local[1]} {local[2]} {{internalCurrentPower:400.0d}}")
        ctl("command", f"item replace entity @s hotbar.0 with {cell} 1")
        ctl("command", "item replace entity @s hotbar.1 with minecraft:emerald 3")
        ctl("wait", 5)
        open_block(local)
        ctl("click-slot", slot_of(cell))
        ctl("click-slot", 301)
        ctl("click-slot", slot_of("minecraft:emerald"))
        ctl("click-slot", 300)
        ctl("wait", 20)
        local_saved = nbt(local)
        if "minecraft:emerald" not in local_saved:
            raise AssertionError(f"unconnected chest did not keep the local insert: {local_saved}")
        print("PASS: unconnected chest used its local buffer", flush=True)
        ctl("tp", 8.5, 73, 4.5)
        ctl("command", "clear @s")
        ctl("setblock", 8, 73, 8, "ae2:cable_bus")
        ctl("command", 'data merge block 8 73 8 {hasRedstone:2,cable:{id:"ae2:fluix_glass_cable"}}')
        ctl("setblock", 9, 73, 8, "ae2:energy_cell")
        ctl("command", "data merge block 9 73 8 {internalCurrentPower:100000.0d}")
        ctl("setblock", *drive, "ae2:drive[facing=south]")
        ctl("command", "data merge block 8 73 9 {priority:1}")
        ctl("setblock", *chest, "ae2:chest[facing=north]")
        ctl("command", f"item replace entity @s hotbar.0 with {cell} 1")
        ctl("command", "item replace entity @s hotbar.1 with minecraft:diamond 7")
        ctl("wait", 10)
        open_block(drive)
        ctl("click-slot", slot_of(cell))
        ctl("click-slot", 300)
        ctl("wait", 5)
        ctl("command", "clear @s ae2:item_storage_cell_1k")
        ctl("command", f"item replace entity @s hotbar.0 with {cell} 1")
        open_block(chest)
        ctl("click-slot", slot_of(cell))
        ctl("click-slot", 301)
        ctl("click-slot", slot_of("minecraft:diamond"))
        ctl("click-slot", 300)
        ctl("wait", 30)
        drive_saved = nbt(drive)
        chest_saved = nbt(chest)
        if "minecraft:diamond" not in drive_saved:
            raise AssertionError(f"network did not store the diamonds in the drive: drive={drive_saved} chest={chest_saved}")
        if "internalCurrentPower" in chest_saved and "400" in chest_saved:
            raise AssertionError(f"connected chest used a filled local buffer: {chest_saved}")
        print("PASS: diamonds entered the drive through the chest", flush=True)
        ctl("command", "clear @s")
        open_block(chest)
        ctl("click-slot", 302, "shift")
        ctl("wait", 10)
        found = ctl("inventory")["counts"].get("minecraft:diamond", 0)
        if found != 7:
            raise AssertionError(f"network extract returned {found}, nbt={nbt(drive)}")
        report = {"passed": True, "logs": str(work), "diamonds": found}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"PASS: chest network insert and extract; logs: {work}", flush=True)
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
