#!/usr/bin/env python3
"""Compare source IncrementalUpdateHelper's public lifecycle and key identities.

Original Guava BiMap/HashSet execute with controlled equal keys carrying distinct
object tokens. Pending sets are compared after sorting by key: cross-runtime
HashSet iteration order is deliberately not a claim of this comparison.
"""
import argparse
import hashlib
import json
from pathlib import Path
import random
import subprocess
import minecraft_reference
import upstream
ROOT=upstream.ROOT
WORK=ROOT/'target/incremental-update-oracle'
HOST=r'''
package oracle;
import java.io.*;
import java.util.*;
import java.util.function.Consumer;
import java.util.stream.Collectors;
import com.google.common.collect.BiMap;
import com.google.common.collect.HashBiMap;
public class IncrementalUpdateOracle {
 @interface Nullable {}
 static class ItemStack {}
 static class AEKey {
  int id,object;AEKey(int id,int object){this.id=id;this.object=object;}
  public int hashCode(){return id;}public boolean equals(Object other){return other instanceof AEKey k && k.id==id;}
  public String toString(){return id+"/"+object;}
 }
 __SOURCE__
 public static void main(String[] args) throws Exception {
  var input=new BufferedReader(new InputStreamReader(System.in));String line;var updates=new IncrementalUpdateHelper();
  while((line=input.readLine())!=null){
   var w=line.split(" ");AEKey key=w.length>1?new AEKey(Integer.parseInt(w[1]),w.length>2?Integer.parseInt(w[2]):0):null;
   String result="-";
   switch(w[0]){
    case "A"->result=Long.toString(updates.getOrAssignSerial(key));
    case "S"->result=String.valueOf(updates.getSerial(key));
    case "R"->updates.removeSerial(key);
    case "C"->updates.clear();
    case "F"->updates.reset();
    case "D"->updates.addChange(key);
    case "M"->updates.commitChanges();
    case "Q"->result=String.valueOf(updates.getBySerial(Long.parseLong(w[1])));
    default->throw new AssertionError();
   }
   var changes=new ArrayList<AEKey>();updates.forEach(changes::add);changes.sort(Comparator.comparingInt(k->k.id));
   System.out.println(result+"|"+updates.hasChanges()+"|"+updates.isFullUpdate()+"|"+changes.stream().map(Object::toString).collect(Collectors.joining(",")));
  }
 }
}
'''

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--java-home',type=Path,required=True);args=parser.parse_args()
    WORK.mkdir(parents=True,exist_ok=True)
    source=(upstream.SOURCE/'src/main/java/appeng/menu/me/common/IncrementalUpdateHelper.java').read_text()
    body=source[source.index('public class IncrementalUpdateHelper'):].replace('public class IncrementalUpdateHelper','public static class IncrementalUpdateHelper',1)
    java=HOST.replace('__SOURCE__',body);path=WORK/'IncrementalUpdateOracle.java';path.write_text(java);cp=minecraft_reference.prepare()
    subprocess.run([str(args.java_home/'bin/javac'),'-proc:none','-cp',cp,'-d',str(WORK),str(path)],cwd=WORK,check=True)
    rows=['A 7 1','A 7 2','D 7 1','D 7 2','Q 1','M','C','Q 1','R 7','D 7 3','A 7 4','Q 1','Q 2','F','Q 2']
    rng=random.Random(912818)
    for i in range(30000):
        action=rng.choices(['A','S','R','C','F','D','M','Q'],[25,15,10,2,1,30,4,13])[0]
        if action in ['C','F','M']:rows.append(action)
        elif action=='Q':rows.append(f'Q {rng.randrange(-2,1000)}')
        else:rows.append(f'{action} {rng.randrange(-32,33)} {i+100}')
    data=('\n'.join(rows)+'\n').encode();(WORK/'input.txt').write_bytes(data)
    expected=subprocess.run([str(args.java_home/'bin/java'),'-cp',str(WORK)+':'+cp,'oracle.IncrementalUpdateOracle'],cwd=WORK,input=data,capture_output=True)
    (WORK/'java.stderr').write_bytes(expected.stderr);expected.check_returncode()
    actual=subprocess.run(['cargo','run','--quiet','--locked','--example','incremental_update_oracle'],cwd=ROOT,input=data,capture_output=True)
    (WORK/'rust.stderr').write_bytes(actual.stderr);actual.check_returncode()
    (WORK/'java.txt').write_bytes(expected.stdout);(WORK/'rust.txt').write_bytes(actual.stdout)
    if expected.stdout!=actual.stdout:
        for i,(a,b) in enumerate(zip(expected.stdout.splitlines(),actual.stdout.splitlines())):
            if a!=b:raise AssertionError(f'operation {i}: {rows[i]}\nJava {a!r}\nRust {b!r}')
        raise AssertionError('trace lengths differ')
    sha=lambda b:hashlib.sha256(b).hexdigest()
    report={'passed':True,'operations':len(rows),'scope':'Original IncrementalUpdateHelper public operations, actual Guava BiMap and HashSet, canonical mapping-key versus latest pending-key object tokens, full/clear/commit/reset lifecycle and lookup/removal; pending sets sorted for comparison',
        'source_sha256':sha(source.encode()),'binding_sha256':sha(java.encode()),'rust_sha256':{name:sha((ROOT/name).read_bytes()) for name in ['mixins/ae2/incremental_update.rs','examples/incremental_update_oracle.rs']},'input_sha256':sha(data),'output_sha256':sha(actual.stdout),
        'limits':['Controlled key equality/hash/object metadata, non-null keys; Java hash iteration order is not reproduced','Native per-player menu lifetime, packets, unknown-key transaction gating and client repository remain unimplemented; serial-overflow collision behavior has separate Rust unit coverage']}
    (ROOT/'docs/incremental-update-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
