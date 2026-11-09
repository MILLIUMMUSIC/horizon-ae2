#!/usr/bin/env python3
"""Compare both complete original StorageHelper powered-transfer methods.

Uses controlled key/storage/energy/action-source/statistic callbacks. This does
not implement or validate native menu transactions, permissions or full energy
configuration; callbacks record the original CONFIG argument.
"""
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import random
import re
import struct
import subprocess
import minecraft_reference
import upstream
ROOT=upstream.ROOT
WORK=ROOT/'target/storage-helper-oracle'
HOST=r'''
package oracle;
import java.io.*;
import java.util.*;
import com.google.common.primitives.Ints;
public class StorageHelperOracle {
 static StringBuilder trace;
 enum Actionable {SIMULATE,MODULATE}
 enum PowerMultiplier {CONFIG}
 static class AEKey {int factor; AEKey(int n){factor=n;}int getAmountPerOperation(){return factor;}}
 static class AEItemKey extends AEKey {AEItemKey(int n){super(n);}}
 interface MEStorage {long extract(AEKey key,long amount,Actionable mode,IActionSource source);long insert(AEKey key,long amount,Actionable mode,IActionSource source);}
 interface IEnergySource {double extractAEPower(double amount,Actionable mode,PowerMultiplier multiplier);}
 static class Player {}
 interface IActionSource {Optional<Player> player();}
 enum AeStats {ItemsExtracted,ItemsInserted;void addToPlayer(Player player,int amount){trace.append("T:").append(name()).append(':').append(amount).append(';');}}
 static class CompoundTag {}
 interface ICraftingRequester {}
 interface ICraftingLink {}
 static class CraftingLink implements ICraftingLink {CraftingLink(CompoundTag tag,ICraftingRequester requester){throw new UnsupportedOperationException("crafting link excluded");}}
 __SOURCE__
 public static void main(String[] args) throws Exception {
  var in=new BufferedReader(new InputStreamReader(System.in));String line;
  while((line=in.readLine())!=null){
   String[] w=line.split(" ");Actionable mode=w[1].equals("1")?Actionable.MODULATE:Actionable.SIMULATE;
   int factor=Integer.parseInt(w[2]);AEKey key=w[3].equals("1")?new AEItemKey(factor):new AEKey(factor);
   boolean player=w[4].equals("1");long amount=Long.parseLong(w[5]),simulated=Long.parseLong(w[6]),committed=Long.parseLong(w[7]);
   double available=Double.longBitsToDouble(Long.parseUnsignedLong(w[8],16));trace=new StringBuilder();
   IActionSource source=()->player?Optional.of(new Player()):Optional.empty();
   MEStorage storage=new MEStorage(){
    public long extract(AEKey k,long n,Actionable action,IActionSource s){if(k!=key||s!=source)throw new AssertionError();trace.append("E:").append(n).append(':').append(action==Actionable.MODULATE).append(';');return action==Actionable.SIMULATE?simulated:committed;}
    public long insert(AEKey k,long n,Actionable action,IActionSource s){if(k!=key||s!=source)throw new AssertionError();trace.append("I:").append(n).append(':').append(action==Actionable.MODULATE).append(';');return action==Actionable.SIMULATE?simulated:committed;}
   };
   IEnergySource energy=(n,action,multiplier)->{trace.append("P:").append(Long.toUnsignedString(Double.doubleToLongBits(n),16)).append(':').append(action==Actionable.MODULATE).append(':').append(multiplier).append(';');return available;};
   long result=w[0].equals("E")?StorageHelper.poweredExtraction(energy,storage,key,amount,source,mode):StorageHelper.poweredInsert(energy,storage,key,amount,source,mode);
   System.out.println(result+"|"+trace);
  }
 }
}
'''

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home',type=Path,required=True)
    args=parser.parse_args();WORK.mkdir(parents=True,exist_ok=True)
    source_path=upstream.SOURCE/'src/main/java/appeng/api/storage/StorageHelper.java'
    source=source_path.read_text();body=source[source.index('public final class StorageHelper'):]
    body=body.replace('public final class StorageHelper','public static final class StorageHelper',1)
    java=HOST.replace('__SOURCE__',body);path=WORK/'StorageHelperOracle.java';path.write_text(java)
    cp=minecraft_reference.prepare()
    subprocess.run([str(args.java_home/'bin/javac'),'-proc:none','-cp',cp,'-d',str(WORK),str(path)],cwd=WORK,check=True)
    bits=lambda n:struct.pack('>d',n).hex()
    rows=[]
    amounts=[-(2**63),-1,0,1,2,63,64,2**31-1,2**31,2**53-1,2**53,2**63-1]
    energies=[-float('inf'),-1.,-0.,0.,.09999999999999998,.1,.10000000000000002,.11,1.,2.1,125.,float('inf'),float('nan')]
    for op,mode,factor,item,player,index in itertools.product(['E','I'],[0,1],[-1,0,1,125,2**31-1],[0,1],[0,1],range(len(amounts))):
        rows.append(f'{op} {mode} {factor} {item} {player} {amounts[index]} {amounts[(index+3)%len(amounts)]} {amounts[(index+7)%len(amounts)]} {bits(energies[index%len(energies)])}')
    rng=random.Random(912817)
    for _ in range(12000):
        op=rng.choice(['E','I']);mode=rng.randrange(2);factor=rng.choice([1,125,rng.randrange(-2**31,2**31)])
        amount=rng.randrange(-2**63,2**63);sim=rng.randrange(-2**63,2**63);commit=rng.randrange(-2**63,2**63)
        energy=rng.getrandbits(64)
        rows.append(f'{op} {mode} {factor} {rng.randrange(2)} {rng.randrange(2)} {amount} {sim} {commit} {energy:016x}')
    data=('\n'.join(rows)+'\n').encode();(WORK/'input.txt').write_bytes(data)
    expected=subprocess.run([str(args.java_home/'bin/java'),'-cp',str(WORK)+':'+cp,'oracle.StorageHelperOracle'],input=data,capture_output=True,cwd=WORK)
    (WORK/'java.stderr').write_bytes(expected.stderr);expected.check_returncode()
    actual=subprocess.run(['cargo','run','--quiet','--locked','--example','storage_helper_oracle'],input=data,capture_output=True,cwd=ROOT)
    (WORK/'rust.stderr').write_bytes(actual.stderr);actual.check_returncode()
    (WORK/'java.txt').write_bytes(expected.stdout);(WORK/'rust.txt').write_bytes(actual.stdout)
    if expected.stdout!=actual.stdout:
        for i,(a,b) in enumerate(itertools.zip_longest(expected.stdout.splitlines(),actual.stdout.splitlines())):
            if a!=b:raise AssertionError(f'case {i}: {rows[i]}\nJava {a!r}\nRust {b!r}')
    sha=lambda b:hashlib.sha256(b).hexdigest()
    report={'passed':True,'cases':len(rows),'source_sha256':sha(source.encode()),'binding_sha256':sha(java.encode()),'rust_sha256':{name:sha((ROOT/name).read_bytes()) for name in ['mixins/ae2/storage_helper.rs','examples/storage_helper_oracle.rs']},'input_sha256':sha(data),'output_sha256':sha(actual.stdout),
        'scope':'Both original powered transfer overloads with explicit mode; complete ordered storage/power/statistics callback traces, source CONFIG argument, signed long counts, saturation and nonfinite energy',
        'limits':['Controlled key classes and inventory/power/action-source/statistics callbacks; crafting-link loader is compiled but excluded', 'No native menu, actual permission check, real player statistic lifecycle or CONFIG implementation is proved']}
    (ROOT/'docs/storage-helper-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
