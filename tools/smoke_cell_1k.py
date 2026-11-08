#!/usr/bin/env python3
"""Craft one ae2:item_storage_cell_1k on a crafting table.

Reads the shaped patterns from the pinned AE2 jar and checks they match the
carried content pack. Supplies the logic processor, certus quartz crystal and
certus quartz dust. Does not run an Inscriber and does not grow certus.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import zipfile

ROOT = Path(__file__).resolve().parents[1]
SOURCES = (
    "ae2:decorative/quartz_glass",
    "ae2:network/cells/item_storage_components_cell_1k_part",
    "ae2:network/cells/item_storage_cell_1k",
)
# Concrete item for each jar ingredient. Crafted outputs are never given.
CHOICE = {
    "ae2:logic_processor": "ae2:logic_processor",
    "ae2:quartz_glass": "ae2:quartz_glass",
    "ae2:cell_component_1k": "ae2:cell_component_1k",
    "#c:dusts/redstone": "minecraft:redstone",
    "#ae2:all_certus_quartz": "ae2:certus_quartz_crystal",
    "#ae2:all_quartz_dust": "ae2:certus_quartz_dust",
    "#c:glass_blocks/cheap": "minecraft:glass",
    "#c:ingots/iron": "minecraft:iron_ingot",
    "#c:ingots/copper": "minecraft:copper_ingot",
}
GIVEN = {
    "ae2:logic_processor",
    "ae2:certus_quartz_crystal",
    "ae2:certus_quartz_dust",
    "minecraft:redstone",
    "minecraft:glass",
    "minecraft:iron_ingot",
    "minecraft:copper_ingot",
}
CELL = "ae2:item_storage_cell_1k"


def ingredient(value):
    if isinstance(value, list):
        return [ingredient(v) for v in value]
    if isinstance(value, dict):
        if "item" in value:
            return value["item"]
        if "tag" in value:
            return "#" + value["tag"]
        raise ValueError(f"unsupported crafting ingredient: {value}")
    return value


def shaped(data):
    result = data["result"]
    return {
        "type": data["type"],
        "pattern": data["pattern"],
        "key": {k: ingredient(v) for k, v in data["key"].items()},
        "result": {"id": result["id"], "count": result.get("count", 1)},
    }


def tag_members(content):
    raw = {}

    def add(tag, values):
        slot = raw.setdefault(tag, [])
        for value in values:
            if isinstance(value, str):
                slot.append(value)
            elif isinstance(value, dict) and "id" in value:
                slot.append(value["id"])

    for path in sorted((content / "data").glob("*/tags/item/**/*.json")):
        rel = path.relative_to(content / "data")
        namespace = rel.parts[0]
        name = Path(*rel.parts[3:]).with_suffix("").as_posix()
        add(f"{namespace}:{name}", json.loads(path.read_text()).get("values", []))

    def resolve(name, out, depth=0):
        if depth > 32:
            return
        for value in raw.get(name, []):
            if value.startswith("#"):
                resolve(value[1:], out, depth + 1)
            elif value not in out:
                out.append(value)

    resolved = {}
    for name in raw:
        items = []
        resolve(name, items)
        resolved[name] = items
    return resolved


def load_recipes(jar, content):
    remap = json.loads((content / "ae2-recipe-map.json").read_text())
    tags = tag_members(content)
    recipes = []
    with zipfile.ZipFile(jar) as archive:
        for source in SOURCES:
            namespace, path = source.split(":", 1)
            origin = shaped(json.loads(archive.read(f"data/{namespace}/recipe/{path}.json")))
            carried_id = remap[source]
            carried_ns, carried_name = carried_id.split(":", 1)
            carried_path = content / "data" / carried_ns / "recipe" / f"{carried_name}.json"
            carried = shaped(json.loads(carried_path.read_text()))
            if carried != origin:
                raise AssertionError(f"{source} carried recipe differs from the jar")
            if origin["type"] != "minecraft:crafting_shaped":
                raise AssertionError(f"{source} is not a shaped crafting recipe")
            pattern = origin["pattern"]
            if len(pattern) != 3 or any(len(row) != 3 for row in pattern):
                raise AssertionError(f"{source} pattern is not 3x3: {pattern}")
            item_of = {}
            for symbol, spec in origin["key"].items():
                if spec not in CHOICE:
                    raise AssertionError(f"{source} ingredient {symbol}={spec} has no supplied item")
                item = CHOICE[spec]
                if spec.startswith("#") and item not in tags.get(spec[1:], []):
                    raise AssertionError(f"{item} is not in {spec}")
                item_of[symbol] = item
            recipes.append({
                "source": source,
                "carried": carried_id,
                "pattern": pattern,
                "key": origin["key"],
                "item_of": item_of,
                "result": origin["result"]["id"],
                "count": origin["result"]["count"],
            })
    return recipes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--content", type=Path, required=True)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    args = parser.parse_args()
    recipes = load_recipes(args.jar, args.content)
    parent = ROOT / "target/smoke"
    parent.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="cell-1k-", dir=parent))
    game_dir = work / "game"
    game_dir.mkdir()
    socket = work / "control.json"
    env = {**os.environ, "HORIZON_HOME": str(work / "home"), "HORIZON_NO_AUDIO": "1"}
    transcript = []
    processes = []
    logs = []
    checks = []

    def ctl(*command):
        run = subprocess.run(
            [str(args.ctl.resolve()), "--socket", str(socket), "--timeout", "25", *map(str, command)],
            cwd=game_dir, env=env, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout)
        transcript.append({"command": command, "exit": run.returncode, "result": data})
        (work / "commands.json").write_text(json.dumps(transcript, indent=2) + "\n")
        if run.returncode:
            raise RuntimeError(f"{command}: {data}")
        return data

    def launch():
        log = (work / "game-1.log").open("w")
        logs.append(log)
        process = subprocess.Popen([
            str(args.game.resolve()), "--world", "Cell1kSmoke", "--creative", "--seed", "42",
            "--offscreen", "--control-socket", str(socket), "--render-distance", "3",
            "--width", "960", "--height", "640", "--game-dir", str(game_dir),
            "--resources", str(args.resources.resolve()), "--dev-mod", str(args.content.resolve()),
        ], cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(process)
        deadline = time.monotonic() + 120
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"game exited: {log.name}")
            try:
                ctl("wait-ready")
                if "using the default resources" in Path(log.name).read_text(errors="replace"):
                    raise AssertionError("mod content fallback")
                return
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(0.1)
        raise RuntimeError(f"game not ready: {log.name}")

    def occupied():
        inv = ctl("inventory")
        return [slot for slot in inv["hotbar"] + inv["main"] if slot.get("item")]

    def item_count(item):
        return sum(slot["count"] for slot in occupied() if slot["item"] == item)

    def slots():
        screen = ctl("screen")
        if screen.get("screen") != "inventory" or screen.get("craft_size") != 3:
            state = ctl("state")
            raise AssertionError({
                "screen": screen.get("screen"),
                "craft_size": screen.get("craft_size"),
                "position": (state.get("player") or {}).get("position"),
                "block": ctl("block", 8, 73, 6),
            })
        return {slot["slot"]: slot for slot in screen["slots"]}

    def place(recipe):
        item_of = recipe["item_of"]
        need = {}
        for row in recipe["pattern"]:
            for symbol in row:
                if symbol != " ":
                    item = item_of[symbol]
                    need[item] = need.get(item, 0) + 1
        for item, count in need.items():
            have = item_count(item)
            if item in GIVEN and have < count:
                given = ctl("give", item, count - have)
                if given.get("given") != count - have:
                    raise AssertionError(f"give {item} returned {given}")
                ctl("wait", 2)
            elif have < count:
                raise AssertionError(f"need {count} {item} from an earlier craft, have {have}")
        for item, count in need.items():
            slot = next(s["slot"] for s in occupied() if s["item"] == item and s["count"] >= count)
            ctl("click-slot", slot)
            ctl("wait", 1)
            for r, row in enumerate(recipe["pattern"]):
                for c, symbol in enumerate(row):
                    if symbol != " " and item_of[symbol] == item:
                        ctl("click-slot", 100 + r * 3 + c, "right")
                        ctl("wait", 1)
            ctl("click-slot", slot)
            ctl("wait", 1)
        grid = slots()
        for r, row in enumerate(recipe["pattern"]):
            for c, symbol in enumerate(row):
                expect = None if symbol == " " else item_of[symbol]
                actual = grid[100 + r * 3 + c].get("item")
                if actual != expect:
                    raise AssertionError(f"{recipe['source']} slot {r},{c} is {actual}, expected {expect}")

    def take(recipe):
        result = slots()[200]
        if result.get("item") != recipe["result"] or result.get("count") != recipe["count"]:
            raise AssertionError(f"{recipe['source']} result {result}")
        before = item_count(recipe["result"])
        ctl("click-slot", 200, "shift")
        ctl("wait", 2)
        if item_count(recipe["result"]) < before + recipe["count"]:
            carried = ctl("screen").get("carried") or {}
            if carried.get("item") == recipe["result"]:
                empty = next(slot["slot"] for slot in ctl("inventory")["hotbar"] + ctl("inventory")["main"] if not slot.get("item"))
                ctl("click-slot", empty)
                ctl("wait", 1)
        after = item_count(recipe["result"])
        if after != before + recipe["count"]:
            raise AssertionError(f"{recipe['result']} {before} -> {after}, expected +{recipe['count']}")
        leftover = [slot.get("item") for i, slot in slots().items() if 100 <= i <= 108 and slot.get("item")]
        if leftover:
            raise AssertionError(f"grid not empty after {recipe['source']}: {leftover}")
        checks.append({
            "source": recipe["source"],
            "carried": recipe["carried"],
            "pattern": recipe["pattern"],
            "key": recipe["key"],
            "result": recipe["result"],
            "count": recipe["count"],
        })

    try:
        launch()
        ctl("wait", 20)
        ctl("command", "fill 5 72 3 11 72 10 stone")
        ctl("command", "fill 5 73 3 11 78 10 air")
        ctl("command", "gamemode creative @s")
        ctl("command", "tp @s 8.5 73 4.5")
        ctl("wait", 10)
        ctl("command", "setblock 8 73 6 minecraft:crafting_table")
        ctl("wait", 5)
        ctl("command", "clear @s")
        block = ctl("block", 8, 73, 6)
        pos = (ctl("state").get("player") or {}).get("position") or [0, 0, 0]
        if block.get("block") != "minecraft:crafting_table" or abs(pos[0] - 8.5) > 1.5 or abs(pos[2] - 4.5) > 1.5:
            raise AssertionError({"block": block, "position": pos})
        ctl("look-at", 8.5, 73.5, 6.5)
        ctl("wait", 4)
        ctl("interact", 8, 73, 6)
        ctl("wait", 8)
        opened = ctl("screen")
        if opened.get("screen") != "inventory" or opened.get("craft_size") != 3:
            raise AssertionError({
                "screen": opened.get("screen"),
                "craft_size": opened.get("craft_size"),
                "looking": ctl("looking-at"),
                "position": (ctl("state").get("player") or {}).get("position"),
                "block": ctl("block", 8, 73, 6),
            })
        for recipe in recipes:
            place(recipe)
            take(recipe)
        ctl("close")
        ctl("wait", 2)
        made = item_count(CELL)
        if made != 1:
            raise AssertionError(f"{CELL} count is {made}, expected 1")
        checks.append({"inventory": CELL, "count": made})
        ctl("quit")
        report = {
            "passed": True,
            "item": CELL,
            "count": made,
            "supplied": sorted(GIVEN),
            "crafts": checks,
            "logs": str(work),
            "limits": [
                "three carried crafting-table recipes only",
                "logic processor, certus quartz crystal and certus quartz dust are given",
                "no Inscriber and no certus growth",
            ],
        }
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, indent=2))
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
