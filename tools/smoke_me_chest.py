#!/usr/bin/env python3
"""Check the retained native ME Chest host in an isolated world.

This checks source inventories, own-cell input, energy, physical state models,
replacement and restart. It does not claim menus or external capabilities.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ctl', type=Path, required=True)
    parser.add_argument('--resources', type=Path, required=True)
    parser.add_argument('--game', type=Path, default=ROOT / 'target/horizon-cargo/fast/horizon-client')
    args = parser.parse_args()
    binary = args.game.resolve()
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    sources = {str(p.relative_to(ROOT)): digest(p) for p in sorted((ROOT / 'mixins/ae2').glob('*')) if p.is_file()}
    binary_hash = digest(binary)
    parent = ROOT / 'target/smoke'; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='me-chest-', dir=parent))
    game_dir = work / 'game'; game_dir.mkdir()
    socket = work / 'control.json'
    env = {**os.environ, 'HORIZON_HOME': str(work / 'home'), 'HORIZON_NO_AUDIO': '1'}
    transcript = []; processes = []; logs = []; checks = []
    chest = (8, 73, 7); drive = (8, 73, 8); power = (8, 73, 9)

    def ctl(*command):
        run = subprocess.run([str(args.ctl.resolve()), '--socket', str(socket), '--timeout', '25', *map(str, command)],
                             cwd=game_dir, env=env, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout)
        transcript.append({'command': command, 'exit': run.returncode, 'result': data})
        (work / 'commands.json').write_text(json.dumps(transcript, indent=2) + '\n')
        if run.returncode:
            raise RuntimeError(f'{command}: {data}')
        return data

    def command(text):
        result = ctl('command', text)
        assert not any('Unknown' in line or '<--[HERE]' in line for line in result.get('chat', [])), result
        return '\n'.join(result.get('chat', []))

    def launch(number):
        log = (work / f'game-{number}.log').open('w'); logs.append(log)
        process = subprocess.Popen([str(binary), '--world', 'MEChestHostSmoke', '--creative', '--seed', '42',
            '--offscreen', '--control-socket', str(socket), '--render-distance', '3', '--width', '960', '--height', '640',
            '--game-dir', str(game_dir), '--resources', str(args.resources.resolve()), '--dev-mod', str(ROOT / 'target/content/ae2')],
            cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(process)
        deadline = time.monotonic() + 100
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f'client exited: {log.name}')
            try:
                ctl('wait-ready')
                assert 'using the default resources' not in Path(log.name).read_text()
                return process
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(.1)
        raise RuntimeError(f'client did not become ready: {log.name}')

    def merge(pos, tag):
        command(f'data merge block {" ".join(map(str, pos))} {tag}')

    def nbt(pos):
        return command(f'data get block {" ".join(map(str, pos))}')

    def number(text, key):
        match = re.search(r'(?:"' + re.escape(key) + r'"|' + re.escape(key) + r'):\s*([-+0-9.eE]+)d', text)
        assert match, (key, text)
        return float(match.group(1))

    def sample():
        ctl('server-hold', 'on')
        try:
            text = nbt(chest)
            return {'tick': ctl('simulation')['game_time'], 'energy': number(text, 'internalCurrentPower'),
                    'nbt': text, 'slots': ctl('container', *chest)['slots'], 'block': ctl('getblock', *chest)['block']}
        finally:
            ctl('server-hold', 'off')

    def check(name, state=None):
        checks.append({'name': name, 'state': state})
        print(f'PASS: {name}', flush=True)

    def setup(energy, cell='ae2:item_storage_cell_1k', count=64):
        ctl('server-hold', 'on')
        try:
            for pos in (chest, drive, power):
                ctl('setblock', *pos, 'air')
            ctl('setblock', *chest, 'ae2:chest')
            merge(chest, '{internalCurrentPower:' + str(float(energy)) + 'd,inv:{item0:' +
                  ('{id:"minecraft:iron_ingot",count:' + str(count) + '}' if count else '{}') +
                  ',item1:{id:"' + cell + '"}},"test:unknown":123456789L}')
        finally:
            ctl('server-hold', 'off')

    def has_count(text, count):
        return bool(re.search(r'"?#"?:\s*' + str(count) + r'L\b', text))

    try:
        first = launch(1)
        ctl('tp', 8.5, 74, 3.5); ctl('wait', 10)
        command('fill 4 72 3 12 72 11 stone')
        command('fill 4 73 3 12 79 11 air')
        ctl('server-hold', 'on')
        try:
            for facing in ['north', 'east', 'south', 'west', 'up', 'down']:
                for lights in ['false', 'true']:
                    for spin in range(4):
                        wanted = f'ae2:chest[facing={facing},lights_on={lights},spin={spin}]'
                        ctl('setblock', *chest, wanted)
                        assert ctl('getblock', *chest)['block'] == wanted
        finally:
            ctl('server-hold', 'off')
        check('all 48 source physical states are registered without a waterlogged property')

        setup(0); ctl('wait', 12)
        zero = sample()
        assert zero['slots'] == [{'item': 'minecraft:iron_ingot', 'count': 64}, {'item': 'ae2:item_storage_cell_1k', 'count': 1}], zero
        assert 'ae2:storage_cell_inv' not in zero['nbt'] and zero['energy'] == 0, zero
        check('an unpowered own cell does not accept the waiting input', zero)
        merge(chest, '{internalCurrentPower:500.0d}'); ctl('wait', 15)
        charged = sample()
        assert charged['slots'] == [None, {'item': 'ae2:item_storage_cell_1k', 'count': 1}], charged
        assert has_count(charged['nbt'], 64) and 'minecraft:iron_ingot' in charged['nbt'], charged
        assert 0 < charged['energy'] < 436 and 'lights_on=true' in charged['block'], charged
        ctl('wait', 25); later = sample()
        ticks = later['tick'] - charged['tick']
        assert ticks > 0 and math.isclose(charged['energy'] - later['energy'], ticks * 1.5, abs_tol=1e-8), (charged, later)
        assert has_count(later['nbt'], 64), later
        check('source input uses the retained own cell and local energy drains at 1.5 AE/tick without stale refill', [charged, later])

        setup(0, 'ae2:fluid_storage_cell_1k')
        ctl('server-hold', 'on')
        try:
            ctl('setblock', *drive, 'ae2:drive[facing=west]')
            merge(drive, '{inv:{item0:{id:"ae2:item_storage_cell_1k"}}}')
            ctl('setblock', *power, 'ae2:creative_energy_cell')
        finally:
            ctl('server-hold', 'off')
        ctl('wait', 70); own = sample(); drive_data = nbt(drive)
        assert 'lights_on=true' in own['block'] and own['slots'][0]['count'] == 64, own
        assert 'ae2:storage_cell_inv' not in own['nbt'] and 'ae2:storage_cell_inv' not in drive_data, (own, drive_data)
        check('grid-connected Chest input targets its own fluid cell and cannot leak into an adjacent Drive item cell', {'chest': own, 'drive': drive_data})

        setup(200, count=9); ctl('wait', 12); before_bad = sample()
        assert has_count(before_bad['nbt'], 9), before_bad
        command('data modify block 8 73 7 inv set value {item0:{id:"minecraft:diamond",count:0},item1:{id:"ae2:item_storage_cell_1k"}}')
        ctl('wait', 15); bad = sample()
        assert re.search(r'count:\s*0', bad['nbt']), bad
        ctl('wait', 15); bad_later = sample()
        assert bad_later['energy'] == bad['energy'] and re.search(r'count:\s*0', bad_later['nbt']), (bad, bad_later)
        check('malformed original inventory is retained and prevents ticking or energy mutation', [bad, bad_later])

        setup(0, count=0); ctl('wait', 12); replacement = sample()
        assert replacement['energy'] == 0 and replacement['slots'][0] is None, replacement
        assert 'ae2:storage_cell_inv' not in replacement['nbt'], replacement
        check('a fresh physical Chest at the same position cannot inherit the previous cell or energy', replacement)

        setup(500, count=23); ctl('wait', 12); saved = sample()
        assert has_count(saved['nbt'], 23), saved
        ctl('save'); ctl('quit'); first.wait(timeout=25)
        launch(2); ctl('wait', 8); restarted = sample()
        assert has_count(restarted['nbt'], 23) and '123456789L' in restarted['nbt'], restarted
        assert restarted['slots'] == [None, {'item': 'ae2:item_storage_cell_1k', 'count': 1}], restarted
        assert 0 <= restarted['energy'] <= saved['energy'], (saved, restarted)
        check('native save/restart keeps both real slots, current cell contents, local power and unknown source NBT', [saved, restarted])
        ctl('tp', 8.5, 73, 4.5); ctl('look-at', 8.5, 73.5, 7.5); ctl('wait', 3)
        ctl('screenshot', str(work / 'chest.png'))
        assert digest(binary) == binary_hash, 'binary changed during smoke'
        assert sources == {str(p.relative_to(ROOT)): digest(p) for p in sorted((ROOT / 'mixins/ae2').glob('*')) if p.is_file()}, 'native source changed during smoke'
        report = {'schema': 1, 'result': 'passed', 'checks': checks, 'binary_sha256': binary_hash,
                  'source_sha256': sources, 'work': str(work),
                  'limitations': ['Command-initialized native host; menus and external capabilities are not exercised.',
                                  'Survival removal, source client cell rendering and external multiplayer remain open.']}
        (work / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report, indent=2))
    finally:
        for process in reversed(processes):
            if process.poll() is None:
                try:
                    ctl('quit'); process.wait(timeout=20)
                except Exception:
                    process.terminate()
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill(); process.wait(timeout=10)
        for log in logs:
            log.close()


if __name__ == '__main__':
    main()
