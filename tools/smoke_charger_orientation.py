#!/usr/bin/env python3
"""Verify source full-orientation placement for the AE2 Charger and Drive.

The world check places blocks as a player with a known yaw and pitch, then reads
back the native facing and spin state. Uses its own isolated world/home and only
closes its own process.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
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
    work = Path(tempfile.mkdtemp(prefix='charger-orientation-', dir=parent))
    game_dir = work / 'game'
    game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f'ae2-charger-orientation-{os.getpid()}.sock'
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
            [str(args.game.resolve()), '--world', 'ChargerOrientationSmoke',
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
                raise RuntimeError('game exited: ' + str(work / 'game.log'))
            try:
                ctl('wait-ready')
                return process
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(.1)
        raise RuntimeError('game did not become ready: ' + str(work / 'game.log'))

    def check(name, value):
        checks.append({'name': name, 'value': value})
        print('PASS:', name, file=sys.stderr, flush=True)

    def off(tick):
        ctl('wait', tick)

    def setup(support, target, player, item, target_state='air'):
        ctl('server-hold', 'on')
        try:
            ctl('setblock', *support, 'minecraft:stone')
            ctl('setblock', *target, target_state)
            ctl('command', f'item replace entity @s hotbar.0 with {item}')
            ctl('select', 1)
            ctl('tp', *player)
        finally:
            ctl('server-hold', 'off')
        off(2)

    def placed(target):
        try:
            ctl('place', *target)
        except RuntimeError:
            (work / 'placement-failure.json').write_text(json.dumps(ctl('state'), indent=2)+'\n')
            raise
        off(3)
        return ctl('block', *target).get('state')

    def case(name, support, target, player, item, expected, target_state='air'):
        setup(support, target, player, item, target_state)
        state = placed(target)
        assert state == expected, (name, state, expected)
        check(name, {'block': state, 'target': list(target), 'player': list(player)})

    try:
        launch()
        ctl('tp', 8.5, 82, 4.5)
        for corner in [(5, 72, 2), (16, 72, 10)]:
            for _ in range(60):
                if ctl('block', *corner).get('loaded'):
                    break
                off(5)
            else:
                raise AssertionError(f'fixture chunk never loaded: {corner}')
        ctl('server-hold', 'on')
        try:
            ctl('command', 'fill 5 72 2 16 72 10 stone')
            ctl('command', 'fill 5 73 2 16 80 10 air')
            ctl('command', 'setblock 10 71 7 stone')
            ctl('command', 'setblock 10 72 7 air')
            ctl('tp', 8.5, 73, 4.5)
        finally:
            ctl('server-hold', 'off')
        off(4)
        assert ctl('getblock', 11, 72, 7)['block'] == 'minecraft:stone'
        # Horizontal look: the source forward is the clicked direction toward
        # the player, so the placed block faces north.
        case('charger horizontal facing north spin 0',
             (8, 73, 7), (8, 73, 6), (8.5, 73, 4.5), 'ae2:charger',
             'ae2:charger[facing=north,spin=0]')
        # Looking down into a one-block hole at >65 degrees: facing up, with the
        # horizontal look supplying the spin reference.
        # Looking west makes the source up reference west, i.e. spin 3.
        case('charger steep look places facing up spin 3',
             (10, 71, 7), (10, 72, 7), (11.05, 73, 7.5), 'ae2:charger',
             'ae2:charger[facing=up,spin=3]')
        # Looking up at a ceiling block: facing down, spin 0 for the north look.
        case('charger steep upward look places facing down spin 0',
             (12, 76, 7), (12, 75, 7), (12.5, 73, 8.05), 'ae2:charger',
             'ae2:charger[facing=down,spin=0]')
        # A west horizontal look makes the down reference west, i.e. spin 1.
        case('charger steep upward look places facing down spin 1',
             (14, 76, 7), (14, 75, 7), (14.95, 73, 7.5), 'ae2:charger',
             'ae2:charger[facing=down,spin=1]')
        # The Drive uses the same full-orientation source strategy.
        case('drive horizontal facing north spin 0',
             (13, 73, 7), (13, 73, 6), (13.5, 73, 4.5), 'ae2:drive',
             'ae2:drive[facing=north,spin=0]')
        case('drive steep look places facing up spin 3',
             (11, 71, 9), (11, 72, 9), (12.05, 73, 9.5), 'ae2:drive',
             'ae2:drive[facing=up,spin=3]')
        case('inscriber horizontal orientation and dry state',
             (8,73,7), (8,73,6), (8.5,73,4.5), 'ae2:inscriber',
             'ae2:inscriber[facing=north,spin=0,waterlogged=false]')
        case('inscriber source steep placement spin',
             (10,71,7), (10,72,7), (11.05,73,7.5), 'ae2:inscriber',
             'ae2:inscriber[facing=up,spin=3,waterlogged=false]')
        case('inscriber retains the source fluid before placement',
             (8,73,7), (8,73,6), (8.5,73,4.5), 'ae2:inscriber',
             'ae2:inscriber[facing=north,spin=0,waterlogged=true]', 'minecraft:water')
        case('charger item state overrides placement orientation',
             (8,73,7), (8,73,6), (8.5,73,4.5),
             'ae2:charger[minecraft:block_state={facing:"east",spin:"2"}]',
             'ae2:charger[facing=east,spin=2]')
        case('inscriber item state overrides orientation and source water',
             (8,73,7), (8,73,6), (8.5,73,4.5),
             'ae2:inscriber[minecraft:block_state={facing:"east",spin:"2",waterlogged:"false"}]',
             'ae2:inscriber[facing=east,spin=2,waterlogged=false]', 'minecraft:water')
        ctl('quit')
        logs[-1].close()
        logs.pop()
        text = (work / 'game.log').read_text(errors='replace')
        for namespace in ('charger', 'drive', 'inscriber'):
            assert f'blockstate definition of ae2:{namespace} not loaded' not in text, \
                f'ae2:{namespace} blockstate did not load'
        check('24 charger/drive models and 48 inscriber models bound', {'log': 'clean'})
        report = {
            'passed': True,
            'logs': str(work),
            'checks': checks,
            'scope': 'Native source FacingWithSpinStrategy placement, item state precedence and Inscriber source water',
            'limits': [
                'Player-facing placement through horizon-ctl; not independent multiplayer',
                'The client prediction path still uses the engine template state until the server update',
                'Charger/Drive machine behavior, menu and model composition remain separate gaps',
            ],
        }
        (work / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
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


if __name__ == '__main__':
    main()
