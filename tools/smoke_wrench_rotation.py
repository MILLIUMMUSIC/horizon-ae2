#!/usr/bin/env python3
"""Verify source Certus Quartz Wrench rotation on AE2 orientable blocks.

Uses actual player right-clicks with explicit faces in an isolated world and
reads the native block state back. Only closes its own game process.
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
    work = Path(tempfile.mkdtemp(prefix='wrench-rotation-', dir=parent))
    game_dir = work / 'game'
    game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f'ae2-wrench-rotation-{os.getpid()}.sock'
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
        log = (work / f'game-{len(processes) + 1}.log').open('w')
        logs.append(log)
        process = subprocess.Popen(
            [str(args.game.resolve()), '--world', 'WrenchRotationSmoke',
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

    def state(pos):
        return ctl('block', *pos).get('state')

    def swings():
        return ctl('state')['swings']

    def screen():
        return ctl('screen')['screen']

    def check(name, value):
        checks.append({'name': name, 'value': value})
        print('PASS:', name, file=sys.stderr, flush=True)

    def block(pos, state_name):
        ctl('setblock', *pos, state_name)

    def hold(item):
        ctl('command', f'item replace entity @s hotbar.0 with {item}')
        ctl('select', 1)
        ctl('wait', 2)

    def interact(pos, face):
        return ctl('interact', *pos, face)

    def rotates(name, pos, face, expected):
        before = swings()
        interact(pos, face)
        ctl('wait', 2)
        current = state(pos)
        assert current == expected, (name, current, expected)
        assert swings() == before + 1, (name, 'swing', before, swings())
        check(name, {'state': current, 'clicked_face': face})

    def refused(name, pos, face):
        interact(pos, face)
        ctl('wait', 2)
        current = state(pos)
        assert current == state_initial[pos], (name, current, state_initial[pos])
        check(name, {'state': current})

    state_initial = {}
    charger = (8, 73, 7)
    drive = (12, 73, 7)
    dispenser = (10, 73, 7)
    crank = (6, 73, 7)
    crank_back = (6, 73, 8)
    crank_west = (5, 73, 7)
    try:
        launch()
        ctl('tp', 8.5, 82, 4.5)
        for corner in [(3, 72, 2), (17, 72, 11)]:
            for _ in range(60):
                if ctl('block', *corner).get('loaded'):
                    break
                ctl('wait', 5)
            else:
                raise AssertionError(f'fixture chunk never loaded: {corner}')
        ctl('server-hold', 'on')
        try:
            ctl('command', 'fill 3 72 2 17 72 11 stone')
            ctl('command', 'fill 3 73 2 17 80 11 air')
            block(charger, 'ae2:charger[facing=north,spin=0]')
            block(drive, 'ae2:drive[facing=north,spin=0]')
            block(dispenser, 'minecraft:dispenser[facing=north]')
            block(crank_back, 'ae2:charger[facing=south,spin=0]')
            block(crank_west, 'ae2:charger[facing=south,spin=0]')
            block(crank, 'ae2:crank[facing=north]')
            ctl('tp', 8.5, 73, 4.5)
        finally:
            ctl('server-hold', 'off')
        ctl('wait', 4)
        assert ctl('getblock', 8, 72, 4)['block'] == 'minecraft:stone'
        state_initial = {pos: state(pos) for pos in
                         [charger, drive, dispenser, crank]}
        assert state_initial[charger] == 'ae2:charger[facing=north,spin=0]', state_initial
        assert state_initial[crank] == 'ae2:crank[facing=north]', state_initial
        ctl('command', 'data merge block 8 73 7 {internalCurrentPower:777.0d,"test:keep":12345L}')

        hold('ae2:certus_quartz_wrench')
        # Charger: four clockwise turns around the up face return to north;
        # every successful turn swings exactly once.
        rotates('wrench turns charger north to east around up',
                charger, 'up', 'ae2:charger[facing=east,spin=0]')
        rotates('wrench turns charger east to south around up',
                charger, 'up', 'ae2:charger[facing=south,spin=0]')
        rotates('wrench turns charger south to west around up',
                charger, 'up', 'ae2:charger[facing=west,spin=0]')
        rotates('wrench turns charger west to north around up',
                charger, 'up', 'ae2:charger[facing=north,spin=0]')
        # Clicking an east face uses the source X-axis turn and derives spin.
        ctl('tp', 11.5, 73, 7.5)
        ctl('wait', 2)
        rotates('wrench turns charger north to down around east',
                charger, 'east', 'ae2:charger[facing=down,spin=0]')
        rotates('wrench derives spin 2 from a second x-axis turn',
                charger, 'east', 'ae2:charger[facing=south,spin=2]')

        # Drive uses the same full-orientation rule and never opens its menu.
        ctl('tp', 12.5, 73, 4.5)
        ctl('wait', 2)
        rotates('wrench turns drive north to east around up',
                drive, 'up', 'ae2:drive[facing=east,spin=0]')
        assert screen() == 'playing', screen()
        check('wrench turn never opens the Drive menu', {'screen': 'playing'})

        # A vanilla orientable block has no AE2 orientation strategy.
        hold('ae2:certus_quartz_wrench')
        ctl('tp', 8.5, 73, 4.5)
        ctl('wait', 2)
        refused('wrench leaves a vanilla dispenser alone', dispenser, 'up')
        # Horizon's vanilla dispenser opens a container screen; close it so the
        # player-facing harness can continue. It is not an AE2 orientation action.
        if screen() != 'playing':
            ctl('close')

        # The common tag includes AE2's network tool, which the source excludes.
        hold('ae2:network_tool')
        state_initial[charger] = 'ae2:charger[facing=south,spin=2]'
        refused('network tool does not rotate the charger', charger, 'up')

        # A non-wrench item leaves the block alone.
        hold('minecraft:stick')
        refused('non-wrench item does not rotate the charger', charger, 'up')

        # The source hook handles only MAIN_HAND and rejects spectators.
        hold('minecraft:stick')
        ctl('command', 'item replace entity @s weapon.offhand with ae2:certus_quartz_wrench')
        refused('offhand wrench does not rotate the charger', charger, 'up')
        ctl('command', 'item replace entity @s weapon.offhand with minecraft:air')
        hold('ae2:certus_quartz_wrench')
        ctl('command', 'gamemode spectator')
        refused('spectator wrench does not rotate the charger', charger, 'up')
        ctl('command', 'gamemode creative')

        # The crank turns only while its new attached side is a crankable.
        hold('ae2:certus_quartz_wrench')
        ctl('tp', 6.5, 73, 4.5)
        ctl('wait', 2)
        state_initial[crank] = 'ae2:crank[facing=north]'
        rotates('wrench turns an attached crank north to east around up',
                crank, 'up', 'ae2:crank[facing=east]')
        state_initial[crank] = 'ae2:crank[facing=east]'
        refused('crank turn refuses a side without a crankable', crank, 'up')
        nbt = '\n'.join(ctl('command', 'data get block 8 73 7')['chat'])
        assert '12345' in nbt and '777' in nbt, nbt
        check('wrench turns preserve charger block entity data', {'nbt': nbt})
        ctl('save')
        ctl('quit')
        launch()
        ctl('wait', 20)
        assert state(charger) == 'ae2:charger[facing=south,spin=2]', state(charger)
        assert state(drive) == 'ae2:drive[facing=east,spin=0]', state(drive)
        assert state(crank) == 'ae2:crank[facing=east]', state(crank)
        nbt = '\n'.join(ctl('command', 'data get block 8 73 7')['chat'])
        assert '12345' in nbt and '777' in nbt, nbt
        check('rotated states and charger data survive restart',
              {'charger': state(charger), 'drive': state(drive), 'crank': state(crank), 'nbt': nbt})
        ctl('quit')
        report = {
            'passed': True,
            'logs': str(work),
            'checks': checks,
            'scope': 'Source Certus Quartz Wrench rotation for supported AE2 orientable blocks',
            'limits': [
                'Shift-disassembly, permissions/claims and the network tool UI remain unported',
                'Only Charger, Drive and Crank carry native orientation strategies today',
                'Rotation is server-authoritative; independent multiplayer is not covered',
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
