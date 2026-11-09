#!/usr/bin/env python3
"""Execute the real 1.21.1 TagEntry for the cube adapter's reference contract.

Uses controlled vanilla-item tag members and empty conditions/functions. It does
not execute LootPool selection, world contexts or a full Java/native comparison.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import minecraft_reference
ROOT=Path(__file__).resolve().parents[1]
WORK=ROOT/'target/cube-loot-reference'
JAVA = r'''package oracle;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
import java.util.function.Consumer;
import com.google.gson.Gson;
public class TagEntryReference {
 public static void main(String[] args) throws Exception {
  Class.forName("ab").getMethod("a").invoke(null);
  Class.forName("akt").getMethod("a").invoke(null);
  Class<?> registryClass=Class.forName("jz"), mapped=Class.forName("ju"), location=Class.forName("akr"), tagClass=Class.forName("awu"), keyClass=Class.forName("akq"), context=Class.forName("err"), entryClass=Class.forName("esm"), poolEntry=Class.forName("esg"), stackClass=Class.forName("cuq");
  Object registry=Class.forName("lt").getField("g").get(null);
  Object key=registryClass.getMethod("d").invoke(registry);
  Object tag=tagClass.getMethod("a",keyClass,location).invoke(null,key,location.getMethod("a",String.class).invoke(null,"ae2_oracle:presses"));
  Constructor<?> ctor=entryClass.getDeclaredConstructor(tagClass,boolean.class,int.class,int.class,List.class,List.class);ctor.setAccessible(true);
  Method bind=mapped.getMethod("a",Map.class), holder=registryClass.getMethod("c",location), itemName=registryClass.getMethod("b",Object.class);
  Method expand=entryClass.getMethod("expand",context,Consumer.class), create=poolEntry.getMethod("a",Consumer.class,context), weight=poolEntry.getMethod("a",float.class), getItem=stackClass.getMethod("g"), getCount=stackClass.getMethod("H");
  List<Map<String,Object>> cases=new ArrayList<>();
  List<String> ids=List.of("minecraft:diamond","minecraft:emerald","minecraft:iron_ingot","minecraft:gold_ingot");
  for(int n=0;n<=4;n++)for(boolean reverse:List.of(false,true))for(boolean expanded:List.of(false,true)) {
   var members=new ArrayList<>(ids.subList(0,n));if(reverse)Collections.reverse(members);
   var holders=new ArrayList<>();for(String id:members){Object loc=location.getMethod("a",String.class).invoke(null,id);holders.add(((Optional<?>)holder.invoke(registry,loc)).orElseThrow());}
   bind.invoke(registry,Map.of(tag,holders));
   Object entry=ctor.newInstance(tag,expanded,1,0,List.of(),List.of());
   var choices=new ArrayList<>();boolean accepted=(boolean)expand.invoke(entry,null,(Consumer<Object>)choices::add);
   var outputs=new ArrayList<List<Map<String,Object>>>();var weights=new ArrayList<Integer>();
   for(Object choice:choices){weights.add((int)weight.invoke(choice,0f));var out=new ArrayList<Map<String,Object>>();create.invoke(choice,(Consumer<Object>)s->{try{out.add(Map.of("item",itemName.invoke(registry,getItem.invoke(s)).toString(),"count",getCount.invoke(s)));}catch(Exception e){throw new RuntimeException(e);}},null);outputs.add(out);}
   cases.add(Map.of("members",members,"expanded",expanded,"accepted",accepted,"choices",outputs,"weights",weights));
  }
  Files.writeString(Path.of(args[0]),new Gson().toJson(cases));
 }
}
'''

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home',type=Path,required=True)
    args=parser.parse_args();WORK.mkdir(parents=True,exist_ok=True)
    cp=minecraft_reference.prepare();source=WORK/'TagEntryReference.java';source.write_text(JAVA)
    subprocess.run([str(args.java_home/'bin/javac'),'-proc:none','-cp',cp,'-d',str(WORK),str(source)],cwd=WORK,check=True)
    result=subprocess.run([str(args.java_home/'bin/java'),'-cp',str(WORK)+os.pathsep+cp,'oracle.TagEntryReference',str(WORK/'reference.json')],cwd=WORK,capture_output=True,text=True)
    (WORK/'java.log').write_text(result.stdout+result.stderr)
    if result.returncode:raise RuntimeError((result.stdout+result.stderr)[-4000:])
    raw=(WORK/'reference.json').read_bytes();rows=json.loads(raw)
    for row in rows:
        expected=[[{'item':n,'count':1} for n in row['members']]] if not row['expanded'] else [[{'item':n,'count':1}] for n in row['members']]
        assert row['accepted'] and row['choices']==expected,row
        assert row['weights']==[1]*len(expected),row
    sha=lambda b:hashlib.sha256(b).hexdigest()
    report={'passed':True,'reference_cases':len(rows),'scope':'Actual Minecraft 1.21.1 TagEntry expansion/create, including ordered tags and empty members, expanded/unexpanded modes; default weight 1 and quality 0',
        'limits':['Controlled vanilla-item tag members in the isolated Java registry; empty conditions/functions and null unused context', 'Native adapter integration is tested separately by horizon-items tests and private-world cube mining; this is not a whole-pool differential comparison'],
        'minecraft_client_sha256':sha((minecraft_reference.WORK/'client.jar').read_bytes()),'mappings_sha256':sha((minecraft_reference.WORK/'client_mappings.txt').read_bytes()),'binding_sha256':sha(JAVA.encode()),'native_adapter_sha256':sha((ROOT/'mixins/ae2/cube_loot.rs').read_bytes()),'reference_output_sha256':sha(raw),'cases':rows}
    (ROOT/'docs/tag-entry-reference.json').write_text(json.dumps(report,indent=2)+'\n')
    print('PASS:',len(rows),'actual Minecraft TagEntry reference cases')
if __name__=='__main__':main()
