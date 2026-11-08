#!/usr/bin/env python3
"""Exercise both source Fluix recipes and item tick boundaries in a private world.

Materials are supplied; source client lightning, arbitrary fluid tags, explosion
recipes, full registry/component codecs and independent multiplayer are separate.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
import zipfile

ROOT=Path(__file__).resolve().parents[1]
CHARGED='ae2:charged_certus_quartz_crystal'
FLUIX='ae2:fluix_crystal'
REDSTONE='minecraft:redstone'
QUARTZ='minecraft:quartz'


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ctl',type=Path,required=True)
    parser.add_argument('--resources',type=Path,required=True)
    parser.add_argument('--game',type=Path,default=ROOT/'target/horizon-cargo/fast/horizon-client')
    parser.add_argument('--content',type=Path,default=ROOT/'target/content/ae2')
    parser.add_argument('--jar',type=Path,default=ROOT/'target/jars/appliedenergistics2-19.2.18.jar')
    args=parser.parse_args()
    lock=json.loads((ROOT/'upstream.lock.json').read_text())
    assert hashlib.sha512(args.jar.read_bytes()).hexdigest()==lock['jar']['sha512']
    remap=json.loads((args.content/'ae2-recipe-map.json').read_text())
    compared=[]
    with zipfile.ZipFile(args.jar) as jar:
        for name in ['fluix_crystals','fluix_crystal']:
            source_id=f'ae2:transform/{name}';source=json.loads(jar.read(f'data/ae2/recipe/transform/{name}.json'))
            carried=args.content/'data/ae2/recipe'/f'{remap[source_id].split(":",1)[1]}.json'
            assert json.loads(carried.read_text())==source
            assert 'circumstance' not in source
            compared.append(source_id)
    parent=ROOT/'target/smoke';parent.mkdir(parents=True,exist_ok=True)
    work=Path(tempfile.mkdtemp(prefix='fluix-source-',dir=parent));game_dir=work/'game';game_dir.mkdir()
    socket=Path(tempfile.gettempdir())/f'ae2-fluix-{os.getpid()}.sock'
    env={**os.environ,'HORIZON_HOME':str(work/'home'),'HORIZON_NO_AUDIO':'1'}
    transcript=[];checks=[];process=None;log=None

    def ctl(*command):
        run=subprocess.run([str(args.ctl.resolve()),'--socket',str(socket),'--timeout','30',*map(str,command)],
            cwd=game_dir,env=env,capture_output=True,text=True,timeout=40)
        data=json.loads(run.stdout)
        transcript.append({'command':command,'exit':run.returncode,'result':data})
        (work/'commands.json').write_text(json.dumps(transcript,indent=2)+'\n')
        if run.returncode:raise RuntimeError((command,data))
        return data

    def cmd(text):return ctl('command',text)
    def entities():return ctl('entities',48,'item')['entities']
    def counts():
        found={}
        for entity in entities():
            x,y,z=entity['pos']
            if not (6<=x<=12 and 72<=y<=84 and 6<=z<=12):continue
            stack=entity['item'];kind=stack['item']
            found[kind]=found.get(kind,0)+stack['count']
        return found
    def check(name,expected):
        found=counts()
        assert found==expected,(name,found,expected)
        checks.append({'name':name,'items':found})
    def clear():cmd('kill @e[type=minecraft:item]')
    def summon(item,count=1,age=0,pos=(8.5,74,8.5)):
        tag='{Item:'+json.dumps({'id':item,'count':count})+f',Age:{age}s,PickupDelay:32767s,Motion:[0.0d,0.0d,0.0d]'+'}'
        cmd('summon minecraft:item '+' '.join(map(str,pos))+' '+tag)
    def ingredients(count=1,age=0):
        summon(CHARGED,count,age);summon(REDSTONE,count);summon(QUARTZ,count)
    def step(n):
        before=ctl('simulation')['game_time'];cmd(f'tick step {n}')
        deadline=time.monotonic()+max(15,n/10)
        while time.monotonic()<deadline:
            current=ctl('simulation')['game_time']
            if current>=before+n:
                assert current==before+n,(before,n,current)
                # Let the authoritative entity snapshot reach this control client.
                ctl('wait',2)
                return
            time.sleep(.05)
        raise RuntimeError(f'tick step {n} timed out')
    def selector(item):return '@e[type=minecraft:item,nbt={Item:{id:"'+item+'"}},limit=1]'
    def pool():
        cmd('fill 6 72 6 12 72 12 minecraft:stone')
        cmd('fill 6 73 6 12 82 12 minecraft:glass')
        cmd('fill 7 73 7 11 82 11 minecraft:air')
        cmd('fill 7 73 7 11 81 11 minecraft:water')

    try:
        log=(work/'game.log').open('w')
        process=subprocess.Popen([str(args.game.resolve()),'--world','FluixSourceSmoke','--creative','--seed','42',
            '--offscreen','--control-socket',str(socket),'--render-distance','3','--width','960','--height','640',
            '--game-dir',str(game_dir),'--resources',str(args.resources.resolve()),'--dev-mod',str(args.content.resolve())],
            cwd=game_dir,env=env,stdout=log,stderr=subprocess.STDOUT)
        for _ in range(300):
            if process.poll() is not None:raise RuntimeError(f'game exited: {work}')
            try:ctl('wait-ready');break
            except (RuntimeError,subprocess.TimeoutExpired,json.JSONDecodeError):time.sleep(.1)
        else:raise RuntimeError('game did not become ready')
        text=(work/'game.log').read_text(errors='replace')
        assert 'content mods not loaded' not in text and 'using the default resources' not in text
        ctl('tp',8.5,84,8.5);ctl('wait-ready')
        for corner in [(6,72,6),(12,82,12)]:
            for _ in range(80):
                if ctl('block',*corner).get('loaded'):break
                ctl('wait',5)
            else:raise AssertionError(f'fixture chunk did not load: {corner}')
        pool();assert ctl('getblock',8,72,8)['block']=='minecraft:stone'
        ctl('fly',8.5,84,8.5,8.5,84,8.5,.1)
        assert ctl('state')['player']['flying']
        cmd('fill 7 73 7 9 82 9 minecraft:glass')
        cmd('fill 8 73 8 8 82 8 minecraft:air')
        cmd('fill 8 73 8 8 76 8 minecraft:water')
        clear()
        # Keep the original PR's actual player throws, then add deterministic
        # server-step boundaries. No test rewrites prepared content.
        cmd('clear @s')
        for slot,item in enumerate([CHARGED,REDSTONE,QUARTZ]):
            cmd(f'item replace entity @s hotbar.{slot} with {item} 1')
            ctl('select',slot+1);ctl('look-at',8.5,74,8.5);ctl('wait',1);ctl('drop');ctl('wait',3)
        ctl('tp',8.5,84,24.5);ctl('wait',200)
        ctl('tp',8.5,78,8.5);ctl('wait',40)
        found=ctl('inventory')['counts'].get(FLUIX,0)
        # The crystal may still be lower in the water column: verify exact
        # conservation across inventory and world before picking up the rest.
        assert found+counts().get(FLUIX,0)==2,(found,counts())
        for entity in entities():
            if entity['item']['item']==FLUIX:
                cmd(f'tp @s 8.5 {entity["pos"][1]} 8.5');ctl('wait',20)
        assert ctl('inventory')['counts'].get(FLUIX,0)==2
        checks.append({'name':'original player throw and pickup','fluix_total':2})
        ctl('tp',8.5,84,24.5);cmd('tick freeze');clear();pool()
        ingredients();step(60);check('strict 60-tick threshold',{CHARGED:1,REDSTONE:1,QUARTZ:1})
        step(1);check('tick 61 consumes one each and creates two Fluix',{FLUIX:2})
        chat='\n'.join(cmd('data get entity '+selector(FLUIX))['chat'])
        assert re.search(r'PickupDelay:\s*0s\b',chat),chat
        checks.append({'name':'fresh result pickup delay','pickup_delay':0})
        clear();summon(CHARGED);summon('ae2:fluix_dust');step(61)
        check('second original Fluix Dust recipe',{FLUIX:1})
        clear();ingredients(3);step(61);check('first of three sequential successes',{CHARGED:2,REDSTONE:2,QUARTZ:2,FLUIX:2})
        step(1);check('success retains clock for next tick',{CHARGED:1,REDSTONE:1,QUARTZ:1,FLUIX:4})
        step(1);check('third success does not wait another 61 ticks',{FLUIX:6})
        clear();ingredients(age=5939);step(60);check('last live tick before expiry',{CHARGED:1,REDSTONE:1,QUARTZ:1})
        step(1);check('expired trigger cannot transform at tick 61',{REDSTONE:1,QUARTZ:1})
        clear();summon(CHARGED);summon(QUARTZ);step(61)
        check('failed attempt leaves ingredients',{CHARGED:1,QUARTZ:1})
        trigger=next(e for e in entities() if e['item']['item']==CHARGED)
        summon(REDSTONE,pos=trigger['pos']);step(60);check('failure resets clock',{CHARGED:1,REDSTONE:1,QUARTZ:1})
        step(1);check('retry at new threshold',{FLUIX:2})
        clear();ingredients();step(60)
        # At y=74.874 the box center starts in air just below the water, but
        # its upper part still receives buoyancy. The surviving end position
        # enters water during this tick. A pre-movement hook misses the recipe.
        cmd('fill 7 73 7 11 74 11 minecraft:air')
        cmd('tp '+selector(CHARGED)+' 8.5 74.874 8.5')
        cmd('tp '+selector(REDSTONE)+' 8.5 75.3 8.5')
        cmd('tp '+selector(QUARTZ)+' 8.5 75.3 8.5')
        step(1);check('fluid sampled after movement',{FLUIX:2})
        clear();cmd('fill 7 73 7 11 81 11 minecraft:air');ingredients();step(80)
        check('dry ingredients never transform',{CHARGED:1,REDSTONE:1,QUARTZ:1})
        clear();summon('minecraft:dirt',1);summon('minecraft:dirt',2);step(40)
        check('vanilla merge conserves three dirt',{'minecraft:dirt':3})
        assert len(entities())==1,entities()
        clear();summon('minecraft:dirt',1,5999);step(1);check('vanilla item expiry unchanged',{})
        clear();cmd('clear @s');cmd('tick unfreeze');ctl('tp',8.5,74,8.5)
        # Ordinary pickup must still use the untouched native permission/stats path.
        cmd('summon minecraft:item 8.5 74 8.5 {Item:{id:"minecraft:dirt",count:3},PickupDelay:0s}')
        ctl('wait',8);assert ctl('inventory')['counts'].get('minecraft:dirt',0)==3
        checks.append({'name':'vanilla immediate pickup','dirt':3})
        report={'passed':True,'logs':str(work),'recipes_compared':compared,'checks':checks,
          'binary_sha256':hashlib.sha256(args.game.read_bytes()).hexdigest(),
          'adapter_sha256':hashlib.sha256((ROOT/'mixins/ae2/transform_throw_world.rs').read_bytes()).hexdigest(),
          'limits':['Supplied ingredients and command-controlled entity lifetime/steps',
                    'Source lightning, explosion transforms, arbitrary fluid tags/components, reload/identity lifecycle and independent multiplayer remain open']}
        (work/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
        print(f'PASS: {len(checks)} Fluix/item checks; logs: {work}',file=sys.stderr)
        ctl('quit');process.wait(timeout=20)
    finally:
        if process is not None and process.poll() is None:
            process.terminate()
            try:process.wait(timeout=10)
            except subprocess.TimeoutExpired:process.kill();process.wait()
        if log:log.close()
        socket.unlink(missing_ok=True)


if __name__=='__main__':main()
