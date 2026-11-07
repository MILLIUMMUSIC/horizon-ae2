#!/usr/bin/env python3
"""Isolated proof: a crank turns one certus crystal into a charged crystal.

Places ae2:charger with ae2:crank as its only power source, supplies one
ae2:certus_quartz_crystal, and turns the crank. Exits 0 only when that charger
holds ae2:charged_certus_quartz_crystal. Uses its own game/home under
target/smoke and closes its own client.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
CHARGED = "ae2:charged_certus_quartz_crystal"
RAW = "ae2:certus_quartz_crystal"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    args = parser.parse_args()
    parent = ROOT / "target/smoke"
    parent.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="charge-certus-", dir=parent))
    game_dir = work / "game"
    game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-charge-certus-{os.getpid()}.sock"
    env = {**os.environ, "HORIZON_HOME": str(work / "home"), "HORIZON_NO_AUDIO": "1"}
    transcript = []
    processes = []
    logs = []
    charger = (8, 73, 7)
    crank = (8, 74, 7)

    def ctl(*command):
        run = subprocess.run(
            [str(args.ctl.resolve()), "--socket", str(socket), "--timeout", "25", *map(str, command)],
            env=env, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout)
        transcript.append({"command": command, "exit": run.returncode, "result": data})
        (work / "commands.json").write_text(json.dumps(transcript, indent=2) + "\n")
        if run.returncode:
            raise RuntimeError(f"{command}: {data}")
        return data

    def launch():
        log = (work / "game.log").open("w")
        logs.append(log)
        process = subprocess.Popen([
            str(args.game.resolve()), "--world", "ChargeCertusSmoke", "--creative", "--seed", "42",
            "--offscreen", "--control-socket", str(socket), "--render-distance", "3",
            "--width", "960", "--height", "640", "--game-dir", str(game_dir),
            "--resources", str(args.resources.resolve()), "--dev-mod", str(ROOT / "target/content/ae2"),
        ], cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(process)
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"game exited: {log.name}")
            try:
                ctl("wait-ready")
                return
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(0.1)
        raise RuntimeError(f"game did not become ready: {log.name}")

    def slots():
        return ctl("container", *charger)["slots"]

    def nbt():
        return "\n".join(ctl("command", "data get block 8 73 7")["chat"])

    def charged_present():
        return slots() == [{"count": 1, "item": CHARGED}]

    try:
        launch()
        ctl("command", "fill 5 72 3 11 72 9 stone")
        ctl("command", "fill 5 73 3 11 78 9 air")
        ctl("tp", 8.5, 73, 4.5)
        ctl("command", "clear @s")
        ctl("setblock", *charger, "ae2:charger[facing=north]")
        ctl("setblock", *crank, "ae2:crank[facing=up]")
        census = ctl("blocks", 5, 72, 3, 11, 78, 9)["counts"]
        creative = ctl("find-block", "ae2:creative_energy_cell", 8)
        if "ae2:creative_energy_cell" in census or creative["count"] != 0 or census.get("ae2:energy_cell"):
            raise RuntimeError(f"power source must be the crank: {census} {creative}")
        ctl("give", RAW, 1)
        ctl("wait", 4)
        assert ctl("inventory")["counts"].get(RAW) == 1
        assert ctl("inventory")["counts"].get(CHARGED, 0) == 0
        ctl("select", RAW)
        ctl("wait", 3)
        ctl("interact", *charger)
        ctl("select", 9)
        ctl("wait", 3)
        assert slots() == [{"count": 1, "item": RAW}]
        assert ctl("inventory")["counts"].get(RAW, 0) == 0
        for turn in range(10):
            ctl("interact", *crank)
            ctl("wait", 20)
            if turn in (0, 8):
                assert f"internalCurrentPower: {(turn + 1) * 160}.0d" in nbt()
        if CHARGED not in nbt():
            assert "internalCurrentPower: 1600.0d" in nbt()
        for _ in range(100):
            if charged_present():
                break
            ctl("wait", 20)
        if not charged_present():
            print(f"FAIL: charged certus crystal is absent: {slots()}", flush=True)
            raise SystemExit(1)
        assert "internalCurrentPower: 0.0d" in nbt()
        assert ctl("inventory")["counts"].get(CHARGED, 0) == 0
        assert ctl("find-block", "ae2:creative_energy_cell", 8)["count"] == 0
        ctl("quit")
        report = {
            "passed": True,
            "supplied": RAW,
            "result": CHARGED,
            "power": "ae2:crank",
            "logs": str(work),
        }
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"PASS: crank-powered charger converted one certus crystal; logs: {work}", flush=True)
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
