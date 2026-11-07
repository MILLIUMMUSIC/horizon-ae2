#!/usr/bin/env python3
"""Craft one controller, then copy one inscriber press in an isolated world.

The controller comes from the restored shaped recipe
(smooth sky stone / fluix crystal / engineering processor, pattern aba/bcb/aba).
The press copy is the carried ae2:inscriber recipe: iron block in the middle,
the supplied calculation press on top, mode inscribe. That mode keeps the press.
The script gives those ingredients only. It does not generate a meteorite and
does not add recipes. A creative energy cell beside the inscriber, off its
front face, is the power source; the inscriber menu is not part of this client,
so the supplied stacks are placed in the original top and middle slots.

Exits 0 only when the player has the controller item and the inscriber holds
both the original press and a second press in its output.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
SKY = "ae2:smooth_sky_stone_block"
FLUIX = "ae2:fluix_crystal"
PROCESSOR = "ae2:engineering_processor"
PRESS = "ae2:calculation_processor_press"
IRON = "minecraft:iron_block"
CONTROLLER = "ae2:controller"
# Restored network.blocks.controller pattern, row-major crafting-table slots.
GRID = {
    100: SKY, 101: FLUIX, 102: SKY,
    103: FLUIX, 104: PROCESSOR, 105: FLUIX,
    106: SKY, 107: FLUIX, 108: SKY,
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--content", type=Path, required=True)
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    args = parser.parse_args()
    parent = ROOT / "target/smoke"
    parent.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="controller-craft-", dir=parent))
    game_dir = work / "game"
    game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-craft-{os.getpid()}.sock"
    env = {**os.environ, "HORIZON_HOME": str(work / "home"), "HORIZON_NO_AUDIO": "1"}
    transcript = []
    processes = []
    logs = []
    table = (6, 73, 7)
    inscriber = (8, 73, 8)
    cell = (9, 73, 8)

    def ctl(*command):
        run = subprocess.run(
            [str(args.ctl.resolve()), "--socket", str(socket), "--timeout", "25", *map(str, command)],
            cwd=game_dir, env=env, text=True, capture_output=True, timeout=30,
        )
        try:
            data = json.loads(run.stdout)
        except json.JSONDecodeError as error:
            raise RuntimeError(f"{command}: {run.stderr or run.stdout}") from error
        transcript.append({"command": command, "exit": run.returncode, "result": data})
        (work / "commands.json").write_text(json.dumps(transcript, indent=2) + "\n")
        if run.returncode:
            raise RuntimeError(f"{command}: {data}")
        return data

    def launch():
        log = (work / "game.log").open("w")
        logs.append(log)
        process = subprocess.Popen(
            [str(args.game.resolve()), "--world", "ControllerCraft", "--creative", "--seed", "42",
             "--offscreen", "--control-socket", str(socket), "--render-distance", "3",
             "--width", "960", "--height", "640", "--game-dir", str(game_dir),
             "--resources", str(args.resources.resolve()), "--dev-mod", str(args.content.resolve())],
            cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT,
        )
        processes.append(process)
        deadline = time.monotonic() + 90
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"game exited {process.returncode}; see {log.name}")
            try:
                ctl("wait-ready")
                if "using the default resources" in Path(log.name).read_text(errors="replace"):
                    raise AssertionError(f"content resource fallback: {log.name}")
                return
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(0.1)
        raise RuntimeError(f"game did not become ready; see {log.name}")

    def count(item):
        return ctl("inventory")["counts"].get(item, 0)

    def screen_item(slot):
        screen = ctl("screen")
        for entry in screen.get("slots") or []:
            if entry.get("slot") == slot:
                return entry.get("item")
        raise AssertionError(f"slot {slot} missing: {screen}")

    def place(item, slot, one):
        held = next(
            entry["slot"] for entry in ctl("inventory")["hotbar"] + ctl("inventory")["main"]
            if entry.get("item") == item
        )
        ctl("click-slot", held)
        ctl("wait", 2)
        ctl("click-slot", slot, *(["right"] if one else []))
        ctl("wait", 2)
        if ctl("inventory").get("carried"):
            ctl("click-slot", held)
            ctl("wait", 2)

    def stacks():
        slots = ctl("container", *inscriber)["slots"]
        found = []
        for slot in slots:
            if not slot:
                found.append(None)
            else:
                found.append((slot.get("item"), slot.get("count")))
        return found

    def nbt(pos):
        return "\n".join(ctl("command", f"data get block {' '.join(map(str, pos))}")["chat"])

    try:
        launch()
        ctl("tp", 8.5, 80, 4.5)
        ctl("wait", 70)
        ctl("command", "fill 4 72 3 12 72 12 stone")
        ctl("command", "fill 4 73 3 12 78 12 air")
        ctl("tp", 6.5, 73, 5.5)
        ctl("wait", 5)
        ctl("time", 6000)
        ctl("command", "clear @s")
        ctl("wait", 5)
        for item, amount in ((SKY, 4), (FLUIX, 4), (PROCESSOR, 1), (PRESS, 1), (IRON, 1)):
            ctl("give", item, amount)
        ctl("wait", 5)
        for item, amount in ((SKY, 4), (FLUIX, 4), (PROCESSOR, 1), (PRESS, 1), (IRON, 1)):
            assert count(item) == amount, (item, amount, ctl("inventory")["counts"])
        ctl("select", 9)
        ctl("setblock", *table, "minecraft:crafting_table")
        ctl("look-at", table[0] + 0.5, table[1] + 0.5, table[2] + 0.5)
        ctl("wait", 3)
        ctl("interact", *table)
        ctl("wait", 5)
        opened = ctl("screen")
        assert opened.get("screen") == "inventory" and opened.get("craft_size") == 3, opened
        for slot, item in GRID.items():
            place(item, slot, item != PROCESSOR)
        ctl("wait", 5)
        placed = {slot: screen_item(slot) for slot in range(100, 109)}
        assert placed == {slot: item for slot, item in GRID.items()}, placed
        assert screen_item(200) == CONTROLLER, ctl("screen")
        ctl("click-slot", 200, "shift")
        ctl("wait", 5)
        if count(CONTROLLER) != 1:
            ctl("click-slot", 200)
            ctl("wait", 2)
            ctl("click-slot", 0)
            ctl("wait", 2)
        assert [screen_item(slot) for slot in range(100, 109)] == [None] * 9, ctl("screen")
        assert count(CONTROLLER) == 1, ctl("inventory")["counts"]
        assert count(SKY) == count(FLUIX) == count(PROCESSOR) == 0
        assert count(PRESS) == 1 and count(IRON) == 1
        ctl("key", "escape", "down")
        ctl("key", "escape", "up")
        ctl("wait", 5)
        print("PASS: crafting table produced one controller from the restored recipe", flush=True)

        ctl("setblock", *inscriber, "ae2:inscriber[facing=north]")
        ctl("setblock", *cell, "ae2:creative_energy_cell")
        ctl("wait", 40)
        assert ctl("getblock", *inscriber)["block"].startswith("ae2:inscriber"), ctl("getblock", *inscriber)
        ctl("command", (
            f"data merge block {inscriber[0]} {inscriber[1]} {inscriber[2]} "
            '{inv:{item0:{id:"ae2:calculation_processor_press",count:1},item1:{},'
            'item2:{id:"minecraft:iron_block",count:1},item3:{}}}'
        ))
        ctl("wait", 10)
        loaded = stacks()
        assert loaded[0] == (PRESS, 1) and loaded[2] == (IRON, 1) and loaded[3] is None, (loaded, nbt(inscriber))
        ctl("command", f"clear @s {PRESS}")
        ctl("command", f"clear @s {IRON}")
        ctl("wait", 5)
        assert count(PRESS) == 0 and count(IRON) == 0 and count(CONTROLLER) == 1
        assert stacks()[0] == (PRESS, 1) and stacks()[3] is None, stacks()
        copied = None
        for _ in range(40):
            ctl("wait", 10)
            now = stacks()
            if now[0] == (PRESS, 1) and now[1] is None and now[2] is None and now[3] == (PRESS, 1):
                copied = now
                break
        else:
            raise AssertionError(f"press was not copied: {stacks()} {nbt(inscriber)}")
        # The controller item and the copied second press must both still be present.
        if count(CONTROLLER) != 1 or copied[0] != (PRESS, 1) or copied[3] != (PRESS, 1):
            raise AssertionError(f"missing controller or second press: {count(CONTROLLER)} {copied}")
        saved = nbt(inscriber)
        if saved.count(PRESS) < 2:
            raise AssertionError(f"saved inscriber does not contain two presses: {saved}")
        controller_count = count(CONTROLLER)
        ctl("tp", 8.5, 73, 5.5)
        ctl("wait", 5)
        ctl("look-at", 8.5, 73.5, 8.5)
        ctl("wait", 5)
        shot = work / "press-copy.png"
        ctl("screenshot", str(shot))
        ctl("quit")
        report = {
            "passed": True,
            "controller": controller_count,
            "press_slots": copied,
            "screenshot": str(shot),
            "logs": str(work),
        }
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"PASS: controller item and copied calculation press are both present; logs: {work}", flush=True)
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=10)
        for handle in logs:
            handle.close()
        socket.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
