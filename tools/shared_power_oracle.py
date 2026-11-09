#!/usr/bin/env python3
"""Compare shared owned WRITE power across original energy service attachment.

Uses the original energy classes and the existing controlled infrastructure from
energy_oracle. The powered host persists independently of its current grid node.
"""
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import random
import subprocess
import energy_oracle
import upstream
ROOT=upstream.ROOT
WORK=ROOT/'target/shared-power-oracle'
JAVA=r'''
import java.util.*;
import appeng.api.config.*;import appeng.api.networking.energy.*;import appeng.api.networking.events.*;
import appeng.me.*;import appeng.me.energy.*;import appeng.me.service.*;import net.minecraft.nbt.*;
public class Main {
 static class State {
  Grid grid=new Grid();PathingService path=new PathingService();EnergyService service=new EnergyService(grid,path);GridNode node;
  State(){grid.energy=service;}
 }
 static class Host implements IAEPowerStorage {
  StoredEnergyAmount power;Grid grid;List<Integer> events=new ArrayList<>();int priority;
  Host(double amount,double max,int priority){this.priority=priority;power=new StoredEnergyAmount(amount,max,e->{events.add(e.ordinal());if(grid!=null&&e==GridPowerStorageStateChanged.PowerEventType.RECEIVE_POWER)grid.postEvent(new GridPowerStorageStateChanged(this,e));});}
  public double injectAEPower(double n,Actionable a){return n-power.insert(n,a==Actionable.MODULATE);}
  public double extractAEPower(double n,Actionable a,PowerMultiplier m){return power.extract(n,a==Actionable.MODULATE);}
  public double getAECurrentPower(){return power.getAmount();}public double getAEMaxPower(){return power.getMaximum();}public int getPriority(){return priority;}public boolean isAEPublicPowerStorage(){return true;}public AccessRestriction getPowerFlow(){return AccessRestriction.WRITE;}
 }
 public static void main(String[] args)throws Exception {
  Locale.setDefault(Locale.ROOT);var in=new Scanner(System.in);int cases=in.nextInt();var buffer=EnergyService.class.getDeclaredField("localStorage");buffer.setAccessible(true);
  for(int c=0;c<cases;c++){
   Host host=new Host(in.nextDouble(),in.nextDouble(),in.nextInt());State[] states={new State(),new State(),new State()};int attached=-1;double share=0;int count=in.nextInt();
   for(int i=0;i<count;i++){int op=in.nextInt();double result=0;
    switch(op){
     case 0->{int id=in.nextInt();State s=states[id];s.node=new GridNode();s.node.idle=1.;s.node.services.put(IAEPowerStorage.class,host);host.grid=s.grid;CompoundTag data=new CompoundTag();data.putDouble("e",share);s.service.addNode(s.node,data);share=0;attached=id;}
     case 1->{State s=states[attached];CompoundTag data=new CompoundTag();s.service.saveNodeData(s.node,data);share=data.getDouble("e");s.service.removeNode(s.node);s.node=null;host.grid=null;attached=-1;}
     case 2->{double amount=in.nextDouble();boolean commit=in.nextInt()!=0;result=host.power.extract(amount,commit);}
     case 3->{double amount=in.nextDouble();boolean commit=in.nextInt()!=0;result=host.power.insert(amount,commit);}
     case 4->host.power.setStored(in.nextDouble());
     case 5->{int id=in.nextInt();double amount=in.nextDouble();boolean commit=in.nextInt()!=0;result=states[id].service.injectPower(amount,commit?Actionable.MODULATE:Actionable.SIMULATE);}
     case 6->{int id=in.nextInt();double amount=in.nextDouble();boolean commit=in.nextInt()!=0;result=states[id].service.extractAEPower(amount,commit?Actionable.MODULATE:Actionable.SIMULATE,PowerMultiplier.ONE);}
     case 7->host.power.setMaximum(in.nextDouble());
     case 8->{int id=in.nextInt();State s=states[id];s.service.onServerStartTick();s.service.onServerEndTick();}
     default->throw new AssertionError();
    }
    System.out.print(result+" "+host.power.getAmount()+" "+host.power.getMaximum()+" "+share+" "+attached+" "+host.events.size());for(int e:host.events)System.out.print(" "+e);host.events.clear();
    for(State s:states){s.service.refreshPower();var b=(GridEnergyStorage)buffer.get(s.service);System.out.print(" "+b.getAECurrentPower()+" "+s.service.getMaxStoredPower()+" "+s.service.getStoredPower()+" "+(s.service.isNetworkPowered()?1:0));}System.out.println();
   }
  }
 }
}
'''

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--java-home',type=Path,required=True);a=p.parse_args();WORK.mkdir(parents=True,exist_ok=True)
    energy_oracle.WORK=WORK;sources=energy_oracle.generate();path=WORK/'java/Main.java';path.write_text(JAVA);classes=WORK/'classes';classes.mkdir(exist_ok=True)
    compiled=subprocess.run([str(a.java_home/'bin/javac'),'-d',str(classes),*map(str,sources)],capture_output=True);(WORK/'javac.stderr').write_bytes(compiled.stderr);compiled.check_returncode()
    rows=[];rng=random.Random(128902)
    for c in range(256):
        maximum=rng.choice([5.,500.,1000.,10000.]);amount=rng.choice([0.,.1,maximum]);priority=rng.choice([-17,0,37]);attached=-1
        ops=[[4,0],[0,0],[5,0,maximum+100,1],[2,2,1],[1],[2,1,1],[0,1],[6,1,1000,1],[1],[0,2]];attached=2
        for _ in range(120):
            op=rng.randrange(9)
            if op==0:
                if attached>=0:continue
                attached=rng.randrange(3);args=[attached]
            elif op==1:
                if attached<0:continue
                args=[];attached=-1
            elif op in [2,3]:args=[rng.choice([0.,.0000005,.000001,.1,1.,8.,5000.]),rng.randrange(2)]
            elif op==4:args=[rng.choice([0.,.1,1.,1000.])]
            elif op in [5,6]:args=[rng.randrange(3),rng.choice([0.,.000001,.1,1.,25.,125.,50000.]),rng.randrange(2)]
            elif op==7:args=[rng.choice([.05,1.,500.,1000.])]
            else:args=[rng.randrange(3)]
            ops.append([op,*args])
        rows.append((amount,maximum,priority,ops))
    lines=[str(len(rows))]
    for amount,maximum,priority,ops in rows:lines.append(f'{amount} {maximum} {priority} {len(ops)}');lines.extend(' '.join(map(str,op)) for op in ops)
    data=('\n'.join(lines)+'\n').encode();(WORK/'input.txt').write_bytes(data)
    java=subprocess.run([str(a.java_home/'bin/java'),'-cp',str(classes),'Main'],cwd=WORK,input=data,capture_output=True);(WORK/'java.stderr').write_bytes(java.stderr);java.check_returncode()
    rust=subprocess.run(['cargo','run','--quiet','--locked','--example','shared_power_oracle'],cwd=ROOT,input=data,capture_output=True);(WORK/'rust.stderr').write_bytes(rust.stderr);rust.check_returncode()
    (WORK/'java.txt').write_bytes(java.stdout);(WORK/'rust.txt').write_bytes(rust.stdout)
    import math
    for i,(x,y) in enumerate(itertools.zip_longest(java.stdout.splitlines(),rust.stdout.splitlines())):
        if x is None or y is None:raise AssertionError('length mismatch')
        xs=list(map(float,x.split()));ys=list(map(float,y.split()))
        if len(xs)!=len(ys) or any(not math.isclose(u,v,rel_tol=1e-11,abs_tol=1e-8) for u,v in zip(xs,ys)):raise AssertionError(f'operation {i}\nJava {x!r}\nRust {y!r}')
    sha=lambda b:hashlib.sha256(b).hexdigest()
    report={'passed':True,'cases':len(rows),'operations':sum(len(r[3]) for r in rows),'source_sha256':{name:sha((energy_oracle.SOURCE/name).read_bytes()) for name in energy_oracle.ORIGINALS},'binding_sha256':{str(p.relative_to(WORK)):sha(p.read_bytes()) for p in sources},'rust_sha256':{p:sha((ROOT/p).read_bytes()) for p in ['mixins/ae2/power.rs','examples/shared_power_oracle.rs']},'input_sha256':sha(data),'java_output_sha256':sha(java.stdout),'rust_output_sha256':sha(rust.stdout),'scope':'Original StoredEnergyAmount/EnergyService/GridEnergyStorage/overlay classes with one retained WRITE-only host, three grids, attachment/detachment, direct local work, injection/extraction, owner events, resizing, saved private buffer shares and ticks','limits':['Controlled grid/node/event infrastructure and collection/math bindings inherited from energy_oracle; this is not a whole-world or native Chest proof','Positive finite capacities, finite amounts and a fixed unique host priority; nonfinite power and dynamic mutable-priority tree behavior excluded','Owner events are delivered at the operation boundary in Rust; no concurrent/reentrant addon observer or native world-event ordering claim','Numerical comparison uses relative 1e-11 and absolute 1e-8 tolerances']}
    (ROOT/'docs/shared-power-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
