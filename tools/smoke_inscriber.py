#!/usr/bin/env python3
"""Exercise original Inscriber recipes, energy and persistence in a private world.

Inputs are supplied. This does not claim full survival progression, the source
menu/render packets, name-press components or independent multiplayer coverage.
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
import zipfile

ROOT = Path(__file__).resolve().parents[1]
TAG_ITEMS = {
    'c:gems/certus_quartz': 'ae2:certus_quartz_crystal',
    'c:gems/fluix': 'ae2:fluix_crystal',
    'c:gems/diamond': 'minecraft:diamond',
    'c:ingots/gold': 'minecraft:gold_ingot',
    'c:silicon': 'ae2:silicon',
    'c:ender_pearls': 'minecraft:ender_pearl',
    'c:dusts/redstone': 'minecraft:redstone',
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ctl', type=Path, required=True)
    parser.add_argument('--resources', type=Path, required=True)
    parser.add_argument('--content', type=Path, default=ROOT / 'target/content/ae2')
    parser.add_argument('--jar', type=Path, default=ROOT / 'target/jars/appliedenergistics2-19.2.18.jar')
    parser.add_argument('--game', type=Path, default=ROOT / 'target/horizon-cargo/fast/horizon-client')
    args = parser.parse_args()
    lock = json.loads((ROOT / 'upstream.lock.json').read_text())
    assert hashlib.sha512(args.jar.read_bytes()).hexdigest() == lock['jar']['sha512']
    remap = json.loads((args.content / 'ae2-recipe-map.json').read_text())
    recipes = []
    with zipfile.ZipFile(args.jar) as archive:
        for name in sorted(archive.namelist()):
            if not name.startswith('data/ae2/recipe/inscriber/') or not name.endswith('.json'):
                continue
            source = json.loads(archive.read(name))
            source_id = 'ae2:' + name.removeprefix('data/ae2/recipe/').removesuffix('.json')
            carried_id = remap[source_id]
            carried = json.loads((args.content / 'data/ae2/recipe' / (carried_id.split(':',1)[1] + '.json')).read_text())
            assert source == carried, (source_id, source, carried)
            recipes.append((source_id, source))
    assert recipes
    parent = ROOT / 'target/smoke'; parent.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='inscriber-', dir=parent))
    game_dir = work / 'game'; game_dir.mkdir()
    env = {**os.environ, 'HORIZON_HOME': str(work / 'home'), 'HORIZON_NO_AUDIO': '1'}
    socket = Path(tempfile.gettempdir()) / f'ae2-inscriber-{os.getpid()}.sock'
    processes, logs, transcript, checks = [], [], [], []
    machine, cell = (8,73,8), (9,73,8)

    def ctl(*command):
        run = subprocess.run([str(args.ctl.resolve()), '--socket', str(socket), '--timeout', '35', *map(str, command)],
                             cwd=game_dir, env=env, capture_output=True, text=True, timeout=45)
        data = json.loads(run.stdout)
        transcript.append({'command': command, 'exit': run.returncode, 'result': data})
        (work / 'commands.json').write_text(json.dumps(transcript, indent=2) + '\n')
        if run.returncode: raise RuntimeError((command, data))
        return data

    def launch():
        log = (work / f'game-{len(processes)+1}.log').open('w'); logs.append(log)
        p = subprocess.Popen([str(args.game.resolve()), '--world', 'InscriberSmoke', '--creative', '--seed', '42',
            '--offscreen', '--control-socket', str(socket), '--render-distance', '3', '--width', '960', '--height', '640',
            '--game-dir', str(game_dir), '--resources', str(args.resources.resolve()), '--dev-mod', str(args.content.resolve())],
            cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(p)
        for _ in range(120):
            if p.poll() is not None: raise RuntimeError(f'game exited: {log.name}')
            try:
                ctl('wait-ready')
                loaded_log=Path(log.name).read_text(errors='replace')
                if 'content mods not loaded' in loaded_log or 'using the default resources' in loaded_log:
                    raise AssertionError(f'AE2 content failed to load: {log.name}')
                return p
            except (RuntimeError, subprocess.TimeoutExpired): time.sleep(.1)
        raise RuntimeError('game did not become ready')

    def shutdown(p):
        ctl('quit'); p.wait(timeout=20); socket.unlink(missing_ok=True)

    def merge(pos, tag):
        ctl('command', f'data merge block {" ".join(map(str,pos))} ' + json.dumps(tag))

    def stacks(pos=machine):
        return [(s['item'], s['count']) if s and s.get('item') else None for s in ctl('container', *pos)['slots']]

    def power(pos, field):
        chat = ctl('command', f'data get block {" ".join(map(str,pos))} {field}').get('chat', [])
        values = re.findall(r'(-?\d+(?:\.\d+)?(?:[Ee][+-]?\d+)?)d\b', '\n'.join(chat))
        return float(values[-1]) if values else 0.0

    def total():
        return sum(power(pos, field) for pos, field in [(machine,'internalCurrentPower'), (machine,'proxy.e'),
                                                       (cell,'internalCurrentPower'), (cell,'proxy.e')])

    def item(value):
        if value is None: return None
        if 'item' in value: return value['item']
        if 'tag' in value: return TAG_ITEMS[value['tag']]
        raise AssertionError(f'unsupported source ingredient in fixture: {value}')

    def fill(recipe, pos=machine, reverse=False):
        ingredients = recipe['ingredients']
        top, bottom = item(ingredients.get('top')), item(ingredients.get('bottom'))
        if reverse: top, bottom = bottom, top
        names = [top, bottom, item(ingredients['middle']), None]
        inv = {f'item{i}': {'id': name, 'count': 1 if i==2 else 2} if name else {} for i,name in enumerate(names)}
        ctl('command', f'data modify block {" ".join(map(str,pos))} inv set value ' + json.dumps(inv))
        return names

    def wait_result(recipe, names, pos=machine):
        expected = [(name, 1 if recipe['mode']=='press' else 2) if name else None for name in names[:2]]
        expected += [None, (recipe['result']['id'], recipe['result'].get('count',1))]
        for _ in range(60):
            ctl('wait', 10)
            if stacks(pos) == expected:
                ctl('wait', 20)  # Finish the remaining smash phase before editing another task.
                assert stacks(pos) == expected
                return expected
        raise AssertionError((recipe, expected, stacks(pos)))

    try:
        p = launch(); ctl('tp', 11.5,82,10.5)
        for corner in [(4,72,3),(18,72,18)]:
            for _ in range(60):
                if ctl('block', *corner).get('loaded'): break
                ctl('wait',5)
            else: raise AssertionError(f'fixture chunk did not load: {corner}')
        ctl('command','fill 4 72 3 18 72 18 stone')
        ctl('command','fill 4 73 3 18 80 18 air')
        assert ctl('getblock',8,72,8)['block']=='minecraft:stone'
        ctl('tp',8.5,73,5.5); ctl('command','clear @s')
        ctl('setblock',*machine,'ae2:inscriber[facing=north,spin=0,waterlogged=false]')
        ctl('setblock',*cell,'ae2:energy_cell'); merge(cell,{'internalCurrentPower':100000.0})
        ctl('wait',30)
        for index,(source_id,recipe) in enumerate(recipes):
            before = total(); names=fill(recipe,reverse=index%2==1)
            expected=wait_result(recipe,names); spent=before-total()
            assert abs(spent-2020.0)<1e-5, (source_id,'expected 101 callbacks * 20 AE',spent)
            checks.append({'recipe':source_id,'mode':recipe['mode'],'reversed_plates':index%2==1,
                           'slots':expected,'spent_AE':spent})
        # Saved original inv and public/private power must survive restart;
        # processing/smash progress itself is transient in the source.
        saved=stacks(); energy=total(); shutdown(p); p=launch(); ctl('wait',30)
        assert stacks()==saved; assert abs(total()-energy)<1e-5
        checks.append({'restart':'original inventories and total AE retained'})
        copy=next(r for name,r in recipes if name=='ae2:inscriber/calculation_processor_press')
        isolated=(12,73,8); front=(12,73,7)
        ctl('setblock',*isolated,'ae2:inscriber[facing=north,spin=0,waterlogged=false]')
        ctl('setblock',*front,'ae2:creative_energy_cell'); fill(copy,isolated); ctl('wait',250)
        assert stacks(isolated)[2]==('minecraft:iron_block',1) and stacks(isolated)[3] is None
        checks.append({'front':'front-facing adjacent power does not run a task'})
        cranked=(8,73,12); crank=(8,74,12)
        ctl('setblock',*cranked,'ae2:inscriber[facing=north,spin=0,waterlogged=false]')
        ctl('setblock',*crank,'ae2:crank[facing=up]'); fill(copy,cranked)
        ctl('tp',8.5,73,10.5); ctl('select',9); ctl('wait',3)
        for _ in range(13): ctl('interact',*crank); ctl('wait',20)
        names=[item(copy['ingredients'].get('top')),None,'minecraft:iron_block',None]
        wait_result(copy,names,cranked)
        assert abs(power(cranked,'internalCurrentPower')-60)<1e-5
        checks.append({'crank_turns':13,'supplied_AE':2080,'remaining_AE':60})
        # Enable auto-export on a sleeping, finished machine without changing
        # its inventory; source onConfigChanged must wake it.
        ctl('setblock',7,73,8,'minecraft:chest')
        merge(machine,{'auto_export':'YES'}); ctl('wait',80)
        assert stacks()[3] is None
        assert any(s and s.get('item')==saved[3][0] and s.get('count')==saved[3][1]
                   for s in ctl('container',7,73,8)['slots'])
        checks.append({'auto_export':'setting wakes an idle machine and exports its actual result'})
        merge(machine,{'auto_export':'NO','upgrades':[
            {'Slot':i,'id':'ae2:speed_card','count':1} for i in range(4)]})
        before=total(); names=fill(copy); wait_result(copy,names)
        spent=before-total()
        assert abs(spent-2500)<1e-5, ('four cards use five 500 AE callbacks',spent)
        checks.append({'speed_cards':4,'spent_AE':spent})
        # Real hopper admission and per-side views. The isolated host has no
        # power, so these assertions observe automation without recipe work.
        automated=(14,73,12); above=(14,74,12); north=(14,73,11); below=(14,72,12)
        ctl('setblock',*automated,'ae2:inscriber[facing=north,spin=0,waterlogged=false]')
        merge(automated,{'inscriber_separate_sides':'YES','inscriber_input_capacity':'ONE'})
        ctl('setblock',*above,'minecraft:hopper[facing=down]')
        def hopper_items(pos, name, count=2):
            ctl('command', f'data merge block {" ".join(map(str,pos))} '
                + '{Items:[{Slot:0b,id:' + json.dumps(name) + ',count:' + str(count) + '}]}')
        hopper_items(above,'minecraft:stone'); ctl('wait',30)
        assert stacks(above)[0]==('minecraft:stone',2) and stacks(automated)==[None]*4
        hopper_items(above,'ae2:calculation_processor_press'); ctl('wait',30)
        assert stacks(above)[0]==('ae2:calculation_processor_press',1)
        assert stacks(automated)[0]==('ae2:calculation_processor_press',1)
        ctl('setblock',*north,'minecraft:hopper[facing=south]')
        hopper_items(north,'minecraft:iron_block'); ctl('wait',30)
        assert stacks(north)[0]==('minecraft:iron_block',1)
        assert stacks(automated)[2]==('minecraft:iron_block',1)
        ctl('setblock',*below,'minecraft:hopper[facing=east]'); ctl('wait',20)
        assert all(s is None for s in stacks(below)), stacks(below)
        merge(automated,{'inv':{'item1':{'id':'ae2:silicon_press','count':2}}})
        ctl('wait',30)
        assert stacks(automated)[1] is None and stacks(below)[0]==('ae2:silicon_press',2)
        merge(below,{'Items':[]}); merge(automated,{'inscriber_separate_sides':'NO'})
        ctl('wait',30)
        assert all(s is None for s in stacks(below))
        assert stacks(automated)[0]==('ae2:calculation_processor_press',1)
        assert stacks(automated)[2]==('minecraft:iron_block',1)
        merge(automated,{'inv':{'item3':{'id':'ae2:fluix_crystal','count':1}}}); ctl('wait',30)
        assert stacks(automated)[3] is None and stacks(below)[0]==('ae2:fluix_crystal',1)
        checks.append({'automation':'invalid input refused; ONE capacity; top/middle admission; separated plate extraction and default output-only extraction'})
        signal_host=(5,73,15); comparator=(6,73,15); dust=(7,73,15)
        ctl('setblock',*signal_host,'ae2:inscriber[facing=north,spin=0,waterlogged=false]')
        signals=[]
        for capacity,counts,expected in [
            ('SIXTY_FOUR',[1,0,0,0],1), ('FOUR',[1,0,0,0],1),
            ('ONE',[1,0,0,0],4), ('FOUR',[4,4,4,4],15),
            ('SIXTY_FOUR',[64,64,64,64],15), ('ONE',[0,0,0,64],15),
        ]:
            ctl('setblock',*comparator,'air')
            inv={f'item{i}':{'id':'minecraft:stone','count':n} if n else {} for i,n in enumerate(counts)}
            ctl('command',f'data modify block {" ".join(map(str,signal_host))} inv set value ' + json.dumps(inv))
            merge(signal_host,{'inscriber_input_capacity':capacity}); ctl('wait',3)
            ctl('setblock',*comparator,'minecraft:comparator[facing=west]')
            ctl('setblock',*dust,'minecraft:redstone_wire'); ctl('wait',5)
            value=ctl('getblock',*dust)['block']
            match=re.search(r'\bpower=(\d+)',value)
            assert match and int(match[1])==expected, (capacity,counts,expected,value)
            signals.append({'capacity':capacity,'counts':counts,'dust':expected})
        checks.append({'comparator':signals})
        before=stacks()
        ctl('command','item replace entity @s hotbar.0 with ae2:certus_quartz_wrench')
        ctl('select',1); ctl('wait',2); ctl('interact',*machine,'up'); ctl('wait',3)
        assert ctl('block',*machine)['state']=='ae2:inscriber[facing=east,spin=0,waterlogged=false]'
        assert stacks()==before
        checks.append({'wrench':'source full orientation turns and retains all four inventory slots'})
        shutdown(p)
        report={'passed':True,'recipes':len(recipes),'checks':checks,'logs':str(work),
                'recipe_jar_sha512':lock['jar']['sha512'],
                'binary_sha256':hashlib.sha256(args.game.read_bytes()).hexdigest(),
                'binding_sha256':hashlib.sha256((ROOT/'mixins/ae2/inscriber_glue.rs').read_bytes()).hexdigest(),
                'state_adapter_sha256':hashlib.sha256((ROOT/'mixins/ae2/block_states.rs').read_bytes()).hexdigest(),
                'limits':['inputs are supplied; this is not complete survival progression',
                          'native integrated server only; external multiplayer remains open',
                          'name-press components, complete menu/render packets and full node lifecycle remain open']}
        (work/'result.json').write_text(json.dumps(report,indent=2)+'\n'); print(json.dumps(report,indent=2))
    finally:
        for p in processes:
            if p.poll() is None:
                p.terminate()
                try: p.wait(timeout=10)
                except subprocess.TimeoutExpired: p.kill(); p.wait(timeout=10)
        for log in logs: log.close()
        socket.unlink(missing_ok=True)


if __name__=='__main__': main()
