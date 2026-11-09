#!/usr/bin/env python3
"""Isolated CableBus item placement, dismantle settings, collision, drops and restart check.

Uses real held-item clicks for all 85 cable centers. Side parts, facades,
Color Applicator, source security and external multiplayer remain separate.
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
    work = Path(tempfile.mkdtemp(prefix='cable-items-', dir=parent))
    game_dir = work/'game'; game_dir.mkdir()
    socket = work/'control.json'
    env = {**os.environ, 'HORIZON_HOME': str(work/'home'), 'HORIZON_NO_AUDIO': '1'}
    transcript = []; processes = []; logs = []; checks = []
    def ctl(*command, allow_fail=False):
        run = subprocess.run([str(args.ctl.resolve()), '--socket', str(socket), '--timeout', '25', *map(str, command)], cwd=game_dir, env=env, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout); transcript.append({'command': command, 'exit': run.returncode, 'result': data})
        (work/'commands.json').write_text(json.dumps(transcript, indent=2)+'\n')
        if run.returncode and not allow_fail:
            raise RuntimeError(f'{command}: {data}')
        return data
    def launch(number):
        log = (work/f'game-{number}.log').open('w'); logs.append(log)
        process = subprocess.Popen([str(args.game.resolve()), '--world', 'CableItemsSmoke', '--creative', '--seed', '42', '--offscreen', '--control-socket', str(socket), '--render-distance', '3', '--width', '960', '--height', '640', '--game-dir', str(game_dir), '--resources', str(args.resources.resolve()), '--dev-mod', str(ROOT/'target/content/ae2')], cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
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

    def inventory():
        inv=ctl('inventory');return [slot for slot in inv['hotbar']+inv['main'] if slot.get('item')]
    def give(item,count=1,components=''):
        ctl('command','item replace entity @s hotbar.0 with '+item+('['+components+']' if components else '')+' '+str(count))
        ctl('select',1);ctl('wait',2)
    def count(item):return sum(s['count'] for s in inventory() if s['item']==item)
    placed=(6,73,7)
    try:
        first=launch(1);ctl('tp',8.5,82,4.5)
        for corner in [(3,71,2),(16,82,13)]:
            for _ in range(80):
                if ctl('block',*corner).get('loaded'):break
                ctl('wait',5)
            else:raise AssertionError(f'fixture chunk did not load: {corner}')
        ctl('command','fill 3 71 2 16 72 13 stone');ctl('command','fill 3 73 2 16 82 13 air')
        assert ctl('getblock',6,72,7)['block']=='minecraft:stone'
        ctl('tp',8.5,73,4.5);ctl('wait',8)
        colors=['white','light_gray','gray','black','lime','yellow','orange','brown','red','pink','magenta','purple','blue','light_blue','cyan','green','fluix']
        kinds=['glass_cable','covered_cable','smart_cable','covered_dense_cable','smart_dense_cable']
        owner_id=None
        for kind in kinds:
            for color in colors:
                item='ae2:'+color+'_'+kind
                give(item,2);ctl('place',*placed);ctl('wait',3)
                text=nbt(placed)
                assert item in text and 'gn:' in text and 'proxy:' not in text,text
                m=re.search(r'\bp:\s*(-?\d+)\b',text);assert m and int(m[1])>=0,text
                if owner_id is None:owner_id=int(m[1])
                assert int(m[1])==owner_id,text
                assert count(item)==2,inventory()
                block(placed,'air')
            print('PASS: 17 held-item placements for '+kind,file=__import__('sys').stderr,flush=True)
        checks.append({'case':'all 85 cable items place original centers in creative without shrinking','owner':owner_id})
        # Feed an actual Charger through cables placed with the real item path.
        ctl('tp',8.5,73,7.5);block((5,73,10),'ae2:creative_energy_cell')
        block((11,73,10),'ae2:charger[facing=north]');give('ae2:fluix_smart_cable',5)
        for x in range(6,11):ctl('place',x,73,10)
        merge((11,73,10),'{inv:{item0:{id:"ae2:certus_quartz_crystal"}}}')
        for _ in range(160):
            ctl('wait',10)
            if ctl('container',11,73,10)['slots']==[{'count':1,'item':'ae2:charged_certus_quartz_crystal'}]:break
        else:raise AssertionError('real item-placed cable chain did not power Charger')
        checks.append('real held-item cable chain carries ME power to a Charger recipe')
        for x in range(5,12):block((x,73,10),'air')
        ctl('command','gamemode survival')
        for i,kind in enumerate(kinds):
            item='ae2:'+colors[i*3]+'_'+kind
            ctl('command','clear @s');ctl('tp',8.5,73,4.5)
            give(item,3,'minecraft:custom_name={text:"Cable name",color:"gold"},minecraft:custom_data={keep_on_unplaced:99}')
            ctl('place',*placed);ctl('wait',5)
            text=nbt(placed);assert 'customName:' in text and 'Cable name' in text,text
            assert 'keep_on_unplaced' not in text,text
            assert count(item)==2,inventory()
            ctl('break',*placed);ctl('tp',6.5,73,7.5);ctl('wait',30)
            stacks=[s for s in inventory() if s['item']==item]
            assert sum(s['count'] for s in stacks)==3,stacks
            dropped=[s for s in stacks if 'keep_on_unplaced' not in s.get('components','')]
            assert sum(s['count'] for s in dropped)==1,stacks
            assert 'Cable name' in dropped[0].get('components',''),dropped
            assert 'stored_energy' not in dropped[0].get('components','') and 'gn:' not in dropped[0].get('components',''),dropped
            checks.append({'case':'survival placement/mining '+kind,'returned':stacks})
        # A new host rejected for entity obstruction is cleaned up and consumes nothing.
        ctl('command','clear @s');block(placed,'air');ctl('tp',6.5,73,7.5);ctl('wait',8)
        give('ae2:fluix_smart_cable',2)
        failed=ctl('place',*placed,allow_fail=True)
        assert ctl('getblock',*placed)['block']=='minecraft:air',failed
        assert count('ae2:fluix_smart_cable')==2,inventory()
        checks.append('player obstruction rolls back the new host and preserves held count')
        for kind in ['covered_dense_cable','smart_dense_cable']:
            item='ae2:fluix_'+kind
            for x, allowed in [(5.99,True),(6.02,False)]:
                ctl('command','clear @s');block(placed,'air')
                ctl('command',f'tp @s {x} 73 7.5');ctl('wait',8)
                give(item,2);result=ctl('place',*placed,allow_fail=True);ctl('wait',5)
                actual=ctl('getblock',*placed)['block']
                assert ('ae2:cable_bus' in actual)==allowed,(kind,x,result,actual)
                assert count(item)==(1 if allowed else 2),inventory()
            block(placed,'air')
        checks.append('both dense types admit a player just outside the 4.9-pixel physical boundary and refuse one just inside; only successful survival placement consumes')
        # Unsupported facade data must survive rejected admission and rollback.
        ctl('command','clear @s');ctl('tp',8.5,73,4.5)
        block(placed,'ae2:cable_bus');merge(placed,'{facadeNorth:{opaque_marker:123}}')
        give('ae2:fluix_glass_cable',2);before=nbt(placed)
        ctl('interact',6,72,7);ctl('wait',4);after=nbt(placed)
        assert after==before and 'opaque_marker: 123' in after and 'cable:' not in after,(before,after)
        assert count('ae2:fluix_glass_cable')==2,inventory()
        checks.append('unsupported facade host is preserved unchanged when central-item admission fails')
        block(placed,'air')
        # Source getDrops must also apply to a Level destruction, not just player mining.
        ctl('command','clear @s');ctl('tp',8.5,73,4.5);give('ae2:red_smart_cable',1)
        ctl('place',*placed);ctl('command','setblock 6 73 7 air destroy');ctl('tp',6.5,73,7.5);ctl('wait',30)
        assert count('ae2:red_smart_cable')==1,inventory()
        checks.append('Level command destruction returns the cable part, never a glass or CableBus item')
        # Water host state is source waterlogged=true, including fluid restoration on removal.
        ctl('command','clear @s');ctl('tp',8.5,73,4.5);block(placed,'water')
        give('ae2:fluix_covered_cable',1);ctl('place',*placed)
        assert 'waterlogged=true' in ctl('getblock',*placed)['block']
        give('minecraft:bucket');ctl('interact',*placed);ctl('wait',4)
        assert 'waterlogged=false' in ctl('getblock',*placed)['block']
        assert count('minecraft:water_bucket')==1,inventory()
        ctl('interact',*placed);ctl('wait',4)
        assert 'waterlogged=true' in ctl('getblock',*placed)['block']
        assert count('minecraft:bucket')==1,inventory()
        checks.append('real bucket drains and refills the CableBus without removing its part')
        broken=ctl('break',*placed)
        assert broken['drops']=={'ae2:fluix_covered_cable':1},broken
        assert ctl('getblock',*placed)['block'].startswith('minecraft:water'),ctl('getblock',*placed)
        checks.append('water placement records source waterlogged state and removal restores water')
        ctl('command','kill @e[type=minecraft:item]',allow_fail=True)
        ctl('command','fill 3 73 2 16 78 13 air')

        # Creative breaks and the block_drops rule suppress parts exactly once.
        ctl('command','gamemode creative');ctl('command','clear @s');block(placed,'air');ctl('tp',8.5,73,4.5)
        give('ae2:blue_glass_cable',1);ctl('place',*placed);ctl('command','clear @s');ctl('break',*placed)
        ctl('tp',6.5,73,7.5);ctl('wait',30);assert not inventory(),inventory()
        checks.append('creative mining produces no cable drop')
        ctl('command','gamemode survival');ctl('tp',8.5,73,4.5);give('ae2:lime_smart_dense_cable')
        ctl('place',*placed);ctl('command','gamerule block_drops false');ctl('break',*placed)
        ctl('tp',6.5,73,7.5);ctl('wait',30);assert not inventory(),inventory()
        ctl('command','gamerule block_drops true');checks.append('disabled block_drops suppresses the part drop')
        # The owner ID and custom name must be read from world data after a new process starts.
        ctl('tp',8.5,73,4.5);give('ae2:cyan_smart_cable',2,'minecraft:custom_name={text:"Restart cable"}')
        ctl('place',*placed);before=nbt(placed);ctl('save');ctl('quit');first.wait(timeout=25)
        launch(2);ctl('wait',20)
        after=nbt(placed);assert 'Restart cable' in after and 'ae2:cyan_smart_cable' in after,(before,after)
        m=re.search(r'\bp:\s*(-?\d+)\b',after);assert m and int(m[1])==owner_id,after
        ctl('tp',10.5,73,4.5);give('ae2:green_glass_cable');ctl('place',9,73,7)
        other=nbt((9,73,7));m=re.search(r'\bp:\s*(-?\d+)\b',other);assert m and int(m[1])==owner_id,other
        checks.append('source part/custom name and world-wide owner mapping survive restart and another placement')
        ctl('quit')
        report={'passed':True,'checks':checks,'logs':str(work),'binary_sha256':hashlib.sha256(args.game.read_bytes()).hexdigest(),'geometry_sha256':{name:hashlib.sha256((ROOT/'mixins/ae2'/name).read_bytes()).hexdigest() for name in ['part_geometry.rs','cable_view.rs','cable_items.rs']},'limits':[
            'Center-only placement/drop; unsupported side parts/facades and compatibility are not implemented',
            'Offhand Color Applicator, complete settings/security, source readiness events and external CTRL synchronization remain open',
            'Water host state/restoration checked; complete fluid simulation, lighting and all direct collision callers remain open',
            'Native game verification; no complete Java/NeoForge world oracle']}
        (work/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
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
