#!/usr/bin/env python3
"""Isolated center-only CableBus native state/render/power/restart check.

Creates parts through source-format commands. Survival placement, complete
multipart behavior, external multiplayer and all collision paths remain open.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--ctl', type=Path, required=True)
    p.add_argument('--resources', type=Path, required=True)
    p.add_argument('--game', type=Path, default=ROOT/'target/horizon-cargo/fast/horizon-client')
    args = p.parse_args()
    parent = ROOT/'target/smoke'; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='cable-world-', dir=parent))
    game_dir = work/'game'; game_dir.mkdir()
    socket = work/'control.json'
    env = {**os.environ, 'HORIZON_HOME': str(work/'home'), 'HORIZON_NO_AUDIO': '1'}
    transcript = []; processes = []; logs = []; checks = []
    def ctl(*command):
        run = subprocess.run([str(args.ctl.resolve()), '--socket', str(socket), '--timeout', '25', *map(str, command)], cwd=game_dir, env=env, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout); transcript.append({'command': command, 'exit': run.returncode, 'result': data})
        (work/'commands.json').write_text(json.dumps(transcript, indent=2)+'\n')
        if run.returncode:
            raise RuntimeError(f'{command}: {data}')
        return data
    def launch(number):
        log = (work/f'game-{number}.log').open('w'); logs.append(log)
        process = subprocess.Popen([str(args.game.resolve()), '--world', 'CableWorldSmoke', '--creative', '--seed', '42', '--offscreen', '--control-socket', str(socket), '--render-distance', '3', '--width', '960', '--height', '640', '--game-dir', str(game_dir), '--resources', str(args.resources.resolve()), '--dev-mod', str(ROOT/'target/content/ae2')], cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(process); deadline = time.monotonic()+100
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f'client exited: {log.name}')
            try:
                ctl('wait-ready')
                if 'using the default resources' in Path(log.name).read_text():
                    raise AssertionError('mod content fallback')
                return process
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(.1)
        raise RuntimeError('client not ready')
    def block(pos, name):
        result = ctl('setblock', *pos, name)
        expected = name.split('[')[0]
        if ':' not in expected:
            expected = 'minecraft:'+expected
        assert result['block'].startswith(expected), result
    def merge(pos, tag):
        ctl('command', f'data merge block {" ".join(map(str,pos))} {tag}')
    def nbt(pos):
        return '\n'.join(ctl('command', f'data get block {" ".join(map(str,pos))}')['chat'])
    def cable(pos, item):
        block(pos, 'ae2:cable_bus')
        merge(pos, '{hasRedstone:2,cable:{id:"ae2:'+item+'"}}')
    try:
        first = launch(1)
        ctl('tp', 15.5, 82, 1.5)
        for corner in [(4,71,-8),(29,72,18)]:
            for _ in range(80):
                if ctl('block',*corner).get('loaded'):break
                ctl('wait',5)
            else:raise AssertionError(f'fixture chunk did not load: {corner}')
        ctl('command', 'fill 4 71 -8 29 72 18 stone')
        ctl('command', 'fill 4 73 -8 29 90 18 air')
        assert ctl('getblock',8,72,8)['block']=='minecraft:stone'
        test = (6, 74, 8)
        ctl('server-hold', 'on')
        try:
            for light in range(16):
                for wet in ['false', 'true']:
                    block(test, f'ae2:cable_bus[light_level={light},waterlogged={wet}]')
                    actual = ctl('getblock', *test)['block']
                    assert f'light_level={light}' in actual and f'waterlogged={wet}' in actual, actual
            block(test, 'air')
            colors = ['white','light_gray','gray','black','lime','yellow','orange','brown','red','pink','magenta','purple','blue','light_blue','cyan','green','fluix']
            kinds = ['glass_cable','covered_cable','smart_cable','covered_dense_cable','smart_dense_cable']
            for row, kind in enumerate(kinds):
                for col, color in enumerate(colors):
                    cable((8+col, 74+row, 8), color+'_'+kind)
        finally:
            ctl('server-hold', 'off')
        ctl('wait', 40)
        checks.append('all 32 source light/waterlogged states registered')
        for row, kind in enumerate(kinds):
            for col, color in enumerate(colors):
                text = nbt((8+col,74+row,8))
                assert f'ae2:{color}_{kind}' in text and 'gn:' in text, text
                assert 'proxy:' not in text, text
        checks.append('all 85 source cable center IDs participate and save gn instead of proxy')
        ctl('tp', 8.5, 73, 5.5); ctl('wait', 10); ctl('look-at', 8.5, 74.5, 8.5); ctl('wait', 3)
        picked = ctl('looking-at')
        assert 'ae2:cable_bus' in json.dumps(picked), picked
        checks.append({'center_outline_pick': picked})
        ctl('tp', 16.5, 73, -4.5); ctl('wait', 10); ctl('look-at', 16.5, 76, 8.5); ctl('wait', 8)
        ctl('key', 'f1', 'down'); ctl('key', 'f1', 'up'); ctl('wait', 3)
        ctl('screenshot', str(work/'cable-types-colors.png'))
        source = (7,74,14); charger = (13,74,14)
        block(source, 'ae2:creative_energy_cell')
        for x in range(8,13):
            cable((x,74,14), 'fluix_smart_cable')
        block(charger, 'ae2:charger[facing=north]')
        merge(charger, '{inv:{item0:{id:"ae2:certus_quartz_crystal"}}}')
        for _ in range(160):
            ctl('wait', 10)
            if ctl('container', *charger)['slots'] == [{'count': 1, 'item': 'ae2:charged_certus_quartz_crystal'}]:
                break
        else:
            raise AssertionError('cable-connected Charger did not finish its real recipe')
        checks.append('actual ME power crosses five CableBus nodes to a real Charger recipe')
        cable((10,74,14), 'red_smart_cable')
        cable((11,74,14), 'blue_smart_cable')
        block(charger, 'air'); block(charger, 'ae2:charger[facing=north]')
        merge(charger, '{inv:{item0:{id:"ae2:certus_quartz_crystal"}}}')
        ctl('wait', 100)
        assert ctl('container', *charger)['slots'] == [{'count': 1, 'item': 'ae2:certus_quartz_crystal'}]
        checks.append('unlike nontransparent cable colors split the live power network')
        cable((11,74,14), 'fluix_smart_cable')
        for _ in range(160):
            ctl('wait', 10)
            if ctl('container', *charger)['slots'] == [{'count': 1, 'item': 'ae2:charged_certus_quartz_crystal'}]:
                break
        else:
            raise AssertionError('transparent cable did not reconnect power')
        checks.append('transparent cable reconnects the colored network')
        persisted = nbt((10,74,14))
        ctl('save'); ctl('quit'); first.wait(timeout=25)
        launch(2); ctl('wait', 40)
        text = nbt((10,74,14))
        assert 'ae2:red_smart_cable' in text and 'gn:' in text and 'proxy:' not in text, (persisted,text)
        assert ctl('container', *charger)['slots'] == [{'count': 1, 'item': 'ae2:charged_certus_quartz_crystal'}]
        checks.append('source center identity, node NBT and downstream inventory survive restart')
        ctl('quit')
        report = {'passed': True, 'checks': checks, 'logs': str(work), 'binary_sha256':hashlib.sha256(args.game.read_bytes()).hexdigest(), 'network_sha256':hashlib.sha256((ROOT/'mixins/ae2/network.rs').read_bytes()).hexdigest(), 'limits': ['parts created by source-format commands, survival placement/drop not exercised here', 'smooth AO/full lighting and all direct collision callers require further coverage', 'facades/side parts/full source lifecycle and external multiplayer remain open', 'GPU screenshot requires visual review; no Java/NeoForge world oracle']}
        (work/'result.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report, indent=2))
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill(); process.wait(timeout=10)
        for log in logs:
            log.close()
        socket.unlink(missing_ok=True)


if __name__ == '__main__':
    main()
