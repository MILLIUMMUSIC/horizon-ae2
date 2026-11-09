#!/usr/bin/env python3
"""Isolated game verification of real finite/creative ME-cell power and restart.

No player saves/settings are read or modified. Stores exact commands/replies.
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

ROOT=Path(__file__).resolve().parents[1]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl",type=Path,required=True)
    parser.add_argument("--resources",type=Path,required=True)
    parser.add_argument("--game",type=Path,default=ROOT/"target/horizon-cargo/fast/horizon-client")
    args=parser.parse_args();parent=ROOT/"target/smoke";parent.mkdir(exist_ok=True)
    work=Path(tempfile.mkdtemp(prefix="power-",dir=parent));game_dir=work/"game";game_dir.mkdir()
    socket=Path(tempfile.gettempdir())/f"ae2-power-{os.getpid()}.sock"
    env={**os.environ,"HORIZON_HOME":str(work/"home"),"HORIZON_NO_AUDIO":"1"}
    transcript=[];processes=[];logs=[]

    def ctl(*command):
        run=subprocess.run([str(args.ctl.resolve()),"--socket",str(socket),"--timeout","25",*map(str,command)],env=env,text=True,capture_output=True,timeout=30)
        data=json.loads(run.stdout);transcript.append({"command":command,"exit":run.returncode,"result":data})
        (work/"commands.json").write_text(json.dumps(transcript,indent=2)+"\n")
        if run.returncode:raise RuntimeError(f"{command}: {data}")
        return data

    def launch(number):
        log=(work/f"game-{number}.log").open("w");logs.append(log)
        process=subprocess.Popen([str(args.game.resolve()),"--world","PowerSmoke","--creative","--seed","42","--offscreen","--control-socket",str(socket),"--render-distance","3","--width","960","--height","640","--game-dir",str(game_dir),"--resources",str(args.resources.resolve()),"--dev-mod",str(ROOT/"target/content/ae2")],cwd=game_dir,env=env,stdout=log,stderr=subprocess.STDOUT)
        processes.append(process);deadline=time.monotonic()+30
        while time.monotonic()<deadline:
            if process.poll() is not None:raise RuntimeError(f"game exited: {log.name}")
            try:ctl("wait-ready");return process
            except (RuntimeError,subprocess.TimeoutExpired):time.sleep(0.1)
        raise RuntimeError(f"game did not become ready: {log.name}")

    def block(pos,name):ctl("setblock",*pos,name)
    def nbt(pos):return "\n".join(ctl("command",f"data get block {' '.join(map(str,pos))}")["chat"])
    def power(pos):
        match=re.search(r"internalCurrentPower:\s*([-+0-9.eE]+)d",nbt(pos));return float(match.group(1)) if match else 0.0
    charger=(8,73,7);back=(8,73,8);front=(8,73,6)
    def slots():return ctl("container",*charger)["slots"]
    def insert():
        ctl("select","ae2:certus_quartz_crystal");ctl("wait",3);ctl("interact",*charger);ctl("select",9);ctl("wait",3)
    def wait_charged():
        for _ in range(100):
            if slots()==[{"count":1,"item":"ae2:charged_certus_quartz_crystal"}]:return
            ctl("wait",20)
        raise AssertionError("network-powered charger did not finish")
    def finite(pos,amount):
        block(pos,"ae2:energy_cell");ctl("command",f"data merge block {' '.join(map(str,pos))} {{internalCurrentPower:{amount}.0d}}")

    try:
        first=launch(1);ctl("tp",8.5,82,6.5)
        for corner in [(5,72,3),(11,78,9)]:
            for _ in range(80):
                if ctl("block",*corner).get("loaded"):break
                ctl("wait",5)
            else:raise AssertionError(f"fixture chunk did not load: {corner}")
        ctl("command","fill 5 72 3 11 72 9 stone");ctl("command","fill 5 73 3 11 78 9 air")
        assert ctl("getblock",8,72,7)["block"]=="minecraft:stone"
        ctl("tp",8.5,73,4.5);ctl("command","clear @s")
        block(charger,"ae2:charger[facing=north]");finite(back,2000);ctl("give","ae2:certus_quartz_crystal",3);ctl("wait",4);insert();wait_charged();ctl("wait",20)
        assert power(back)+power(charger)==400.0,"finite cell must lose exactly the 1600 AE consumed by one recipe"
        print("PASS: finite ME cell powers Charger; total remaining energy = 400 AE",flush=True,file=sys.stderr)

        # New empty Charger; putting the power source at its FRONT must not connect.
        block(back,"air");block(charger,"air");block(charger,"ae2:charger[facing=north]");finite(front,2000);insert();ctl("wait",100)
        assert slots()==[{"count":1,"item":"ae2:certus_quartz_crystal"}]
        assert power(front)==2000.0 and power(charger)==0.0
        print("PASS: front face does not connect or supply power",flush=True,file=sys.stderr)

        block(front,"air");block(back,"ae2:creative_energy_cell");wait_charged()
        print("PASS: creative ME cell powers the same recipe without a crank",flush=True,file=sys.stderr)

        # Charge an EMPTY Charger, then restart with nonzero finite storage on
        # both sides. It must not manufacture power during network re-creation.
        block(back,"air");block(charger,"air");block(charger,"ae2:charger[facing=north]");finite(back,2000);ctl("wait",60)
        assert power(charger)==1600.0 and power(back)==400.0
        ctl("save");ctl("quit");first.wait(timeout=20);launch(2);ctl("wait",20)
        assert power(charger)==1600.0 and power(back)==400.0
        insert();wait_charged();ctl("wait",20)
        assert power(charger)+power(back)==400.0
        ctl("look-at",8.5,73.5,7.5);ctl("key","f1","down");ctl("key","f1","up");ctl("screenshot",str(work/"powered-charger.png"));ctl("quit")
        report={"passed":True,"scope":["finite_cell_power","front_face_restriction","creative_cell_power","restart_energy_conservation"],"logs":str(work),"binary_sha256":hashlib.sha256(args.game.read_bytes()).hexdigest(),"network_sha256":hashlib.sha256((ROOT/"mixins/ae2/network.rs").read_bytes()).hexdigest()}
        (work/"result.json").write_text(json.dumps(report,indent=2)+"\n");print(json.dumps(report,indent=2));print(f"PASS: restart conserves finite cell + Charger energy; logs: {work}",flush=True,file=sys.stderr)
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
                try:process.wait(timeout=10)
                except subprocess.TimeoutExpired:process.kill();process.wait(timeout=10)
        for log in logs:log.close()
        socket.unlink(missing_ok=True)


if __name__=="__main__":main()
