#!/usr/bin/env python3
"""Place ae2:terminal on a cable face and open it.

The part is stored on that side with its own node data. This is not the
crafting-table recipe for the item.
"""
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
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    args = parser.parse_args()
    parent = ROOT / "target/smoke"
    parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="terminal-", dir=parent))
    game_dir = work / "game"
    game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-terminal-{os.getpid()}.sock"
    env = {**os.environ, "HORIZON_HOME": str(work / "home"), "HORIZON_NO_AUDIO": "1"}
    content = ROOT / "target/content/ae2"
    processes, logs, transcript = [], [], []
    cable = (8, 73, 7)

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
            [str(args.game.resolve()), "--world", "TerminalSmoke", "--creative", "--seed", "42",
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
        ctl("command", "fill 5 72 3 11 72 9 stone")
        ctl("command", "fill 5 73 3 11 78 9 air")
        ctl("command", "tp @s 8.5 73 4.5")
        ctl("wait", 40)
        ctl("tp", 8.5, 73, 4.5)
        ctl("command", "item replace entity @s hotbar.0 with ae2:fluix_glass_cable 1")
        ctl("select", 1)
        ctl("wait", 2)
        ctl("place", *cable)
        ctl("wait", 5)
        ctl("command", "item replace entity @s hotbar.0 with ae2:terminal 1")
        ctl("select", 1)
        ctl("wait", 2)
        ctl("interact", *cable, "north")
        ctl("wait", 5)
        saved = "\n".join(ctl("command", "data get block 8 73 7")["chat"])
        if "ae2:terminal" not in saved or "ae2:fluix_glass_cable" not in saved:
            raise AssertionError(f"terminal side part missing: {saved}")
        if saved.count('"gn"') < 1 and saved.count("gn:") < 1 and "gn" not in saved:
            raise AssertionError(f"terminal has no node data: {saved}")
        print("PASS: terminal part is on the cable", flush=True)
        ctl("command", "clear @s")
        ctl("interact", *cable, "north")
        ctl("wait", 5)
        screen = ctl("screen")
        if screen["screen"] != "container":
            raise AssertionError(f"terminal did not open: {screen}")
        report = {"passed": True, "logs": str(work), "screen": screen["screen"]}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"PASS: terminal screen opened; logs: {work}", flush=True)
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
