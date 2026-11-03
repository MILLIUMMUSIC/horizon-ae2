#!/usr/bin/env python3
"""Verify the Wooden Crank's source facing states and per-facing shape.

The player aims at points inside and outside the source crank box; the native
crosshair target must include the machine only where the shaped face covers it.
Only this smoke's own game process is closed.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ctl', type=Path, required=True)
    parser.add_argument('--resources', type=Path, required=True)
    parser.add_argument('--game', type=Path,
                        default=ROOT / 'target/horizon-cargo/fast/horizon-client')
    args = parser.parse_args()
    parent = ROOT / 'target/smoke'
    parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='crank-states-', dir=parent))
    game_dir = work / 'game'
    game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f'ae2-crank-states-{os.getpid()}.sock'
    env = {**os.environ, 'HORIZON_HOME': str(work / 'home'), 'HORIZON_NO_AUDIO': '1'}
    transcript = []
    processes = []
    logs = []
    checks = []

    def ctl(*command):
        run = subprocess.run(
            [str(args.ctl.resolve()), '--socket', str(socket), '--timeout', '25',
             *map(str, command)],
            env=env, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout)
        transcript.append({'command': command, 'exit': run.returncode, 'result': data})
        (work / 'commands.json').write_text(json.dumps(transcript, indent=2) + '\n')
        if run.returncode:
            raise RuntimeError(f'{command}: {data}')
        return data

    def launch():
        log = (work / 'game.log').open('w')
        logs.append(log)
        process = subprocess.Popen(
            [str(args.game.resolve()), '--world', 'CrankStatesSmoke',
             '--creative', '--seed', '42', '--offscreen',
             '--control-socket', str(socket), '--render-distance', '3',
             '--width', '960', '--height', '640', '--game-dir', str(game_dir),
             '--resources', str(args.resources.resolve()),
             '--dev-mod', str(ROOT / 'target/content/ae2')],
            cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(process)
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError('game exited: ' + log.name)
            try:
                ctl('wait-ready')
                return process
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(.1)
        raise RuntimeError('game did not become ready: ' + log.name)

    def check(name, value):
        checks.append({'name': name, 'value': value})
        print('PASS:', name, flush=True)

    def target(point):
        ctl('look-at', *point)
        ctl('wait', 1)
        return ctl('looking-at')

    def block(point):
        hit = target(point)
        return hit['block']['block'] if hit.get('kind') == 'block' else None

    try:
        launch()
        # Load the working chunks before the setup commands read them.
        ctl('tp', 6.5, 73, 4.5)
        ctl('wait-ready')
        ctl('wait', 2)
        charger = (6, 73, 8)
        charger_west = (5, 73, 7)
        crank = (6, 73, 7)
        ctl('server-hold', 'on')
        try:
            ctl('command', 'fill 3 72 2 17 72 11 stone')
            ctl('command', 'fill 3 73 2 17 80 11 air')
            ctl('setblock', *charger, 'ae2:charger[facing=south,spin=0]')
            ctl('setblock', *charger_west, 'ae2:charger[facing=south,spin=0]')
            ctl('setblock', *crank, 'ae2:crank[facing=north]')
            ctl('tp', 6.5, 73, 4.5)
        finally:
            ctl('server-hold', 'off')
        ctl('wait', 4)
        assert ctl('block', *crank)['state'] == 'ae2:crank[facing=north]'
        # North facing box: x 0.15..0.85, y 0.15..0.85, z 0.30..1.00.
        assert block((6.75, 73.5, 7.5)) == 'ae2:crank'
        check('crank north shape covers its offset box', {'point': [6.75, 73.5, 7.5]})
        # A ray passing below the box bottom (y 0.15) misses the crank.
        assert block((6.5, 73.0, 7.5)) != 'ae2:crank'
        check('crank north shape leaves the source gap below it', {'point': [6.5, 73.0, 7.5]})
        ctl('command', 'item replace entity @s hotbar.0 with ae2:certus_quartz_wrench')
        ctl('select', 1)
        ctl('wait', 2)
        ctl('interact', *crank, 'up')
        ctl('wait', 2)
        assert ctl('block', *crank)['state'] == 'ae2:crank[facing=east]'
        # East facing box: x 0.00..0.70, y 0.15..0.85, z 0.15..0.85.
        assert block((6.8, 73.5, 7.5)) != 'ae2:crank'
        check('crank east shape no longer covers the north-only x range',
              {'point': [6.8, 73.5, 7.5]})
        assert block((6.35, 73.5, 7.5)) == 'ae2:crank'
        check('crank east shape covers its rotated box', {'point': [6.35, 73.5, 7.5]})
        ctl('quit')
        report = {
            'passed': True,
            'logs': str(work),
            'checks': checks,
            'scope': 'Wooden Crank source facing-only state and per-facing selection shape',
            'limits': [
                'Placement orientation/canSurvive, animation/TESR, synchronization, statistics and fake players remain unported',
                'The selection check exercises the effective gameplay/collision tables; exact collision boxes are also asserted by the native library test',
                'The inherited dispenser behavior chain and independent multiplayer are not covered',
            ],
        }
        (work / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
        print('PASS: crank facing states and shape; logs:', work, flush=True)
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


if __name__ == '__main__':
    main()
