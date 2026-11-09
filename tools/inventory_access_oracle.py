#!/usr/bin/env python3
"""Compare source inventory mutation/notification order and retained stacks."""
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import random
import subprocess

import upstream
from storage_menu_oracle import method

ROOT = upstream.ROOT
WORK = ROOT / 'target/inventory-access-oracle'
HOST = r'''
package oracle;
import java.io.*;import java.util.*;
public class InventoryAccessOracle {
 static StringBuilder trace;static int nextId;static Ref f;
 static class ItemStack {
  static final ItemStack EMPTY=new ItemStack();
  final int id,key,max,component;int count;
  private ItemStack(){id=0;key=-1;max=64;component=0;}
  ItemStack(int k,int n,int m,int c){id=nextId++;key=k;count=n;max=m;component=c;}
  boolean isEmpty(){return key<0||count<=0;}int getCount(){return isEmpty()?0:count;}int getMaxStackSize(){return max;}
  ItemStack copy(){trace.append("COPY:").append(id).append(';');return isEmpty()?EMPTY:new ItemStack(key,count,max,component);}
  void setCount(int n){trace.append("COUNT:").append(id).append(':').append(n).append(';');count=n;}
  void shrink(int n){setCount(getCount()-n);}
  static boolean isSameItemSameComponents(ItemStack a,ItemStack b){trace.append("SAME:").append(a.id).append(':').append(b.id).append(';');return a.key==b.key&&a.component==b.component;}
  public String toString(){return id+":"+key+"/"+count+"/"+max+"/"+component;}
 }
 static class Preconditions {static void checkArgument(boolean valid,String ignored){if(!valid)throw new IllegalArgumentException();}}
 interface InternalInventory {
  int size();boolean isItemValid(int slot,ItemStack s);ItemStack getStackInSlot(int slot);int getSlotLimit(int slot);void setItemDirect(int slot,ItemStack s);
  __INSERT__
 }
 static class Filter {
  boolean allowExtract(Inv inv,int slot,int amount){trace.append("EXTRACT:").append(slot).append(':').append(amount).append(';');return f.extract;}
 }
 static class Host {
  boolean isClientSide(){return f.client!=0;}
  void onChangeInventory(Inv inv,int slot){
   trace.append("ON:").append(slot).append(':').append(inv.stacks.get(slot)).append(';');
   if((f.race&1)!=0)f.argument.count+=3;
   if((f.race&2)!=0)inv.stacks.set(slot,new ItemStack(2,7,64,2));
   if((f.race&4)!=0)f.original.count=1;
   if((f.race&8)!=0)inv.notifyContentsChanged(slot);
  }
  void saveChangedInventory(Inv inv){trace.append("SAVE:0;");}
 }
 static class Inv implements InternalInventory {
  List<ItemStack> stacks;Host host;Filter filter=new Filter();boolean notifyingChanges;
  Inv(ItemStack current){stacks=new ArrayList<>(List.of(current));host=f.client<0?null:new Host();}
  public int size(){return 1;}
  public boolean isItemValid(int slot,ItemStack s){trace.append("VALID:").append(slot).append(':').append(s.id).append(';');return f.valid;}
  public ItemStack getStackInSlot(int slot){trace.append("GET:").append(slot).append(';');return stacks.get(slot);}
  public int getSlotLimit(int slot){trace.append("LIMIT:").append(slot).append(';');return f.limit;}
  boolean isEnableClientEvents(){return f.enabled;}
  __INV__
 }
 interface ItemHandler {ItemStack extractItem(int slot,int amount,boolean simulate);}
 static class GenericStack {static boolean isWrapped(ItemStack s){return !s.isEmpty()&&s.key==3;}}
 static class Wrapper implements ItemHandler {
  Inv inventory;Wrapper(Inv inv){inventory=inv;}
  __WRAPPER__
 }
 static class Ref {boolean valid,extract,enabled;int client,race,limit;ItemStack original,argument;}
 public static void main(String[] args)throws Exception {
  var input=new BufferedReader(new InputStreamReader(System.in));String line;
  while((line=input.readLine())!=null){
   String[] w=line.split(" ");int[] n=new int[w.length];for(int i=1;i<w.length;i++)n[i]=Integer.parseInt(w[i]);
   trace=new StringBuilder();nextId=1;ItemStack.EMPTY.count=0;f=new Ref();
   f.valid=n[4]!=0;f.extract=n[5]!=0;f.limit=n[6];f.client=n[7];f.enabled=n[8]!=0;f.race=n[9];
   f.original=new ItemStack(n[10],n[11],n[12],n[13]);
   f.argument=n[18]!=0?f.original:new ItemStack(n[14],n[15],n[16],n[17]);
   var inv=new Inv(f.original);String result;
   try{result=""+switch(w[0]){case "I"->inv.insertItem(n[1],f.argument,n[3]!=0);case "E"->inv.extractItem(n[1],n[2],n[3]!=0);case "W"->new Wrapper(inv).extractItem(n[1],n[2],n[3]!=0);default->throw new AssertionError();};}
   catch(IllegalArgumentException e){result="SlotCheck";}catch(IndexOutOfBoundsException e){result="SlotRead";}
   System.out.println(result+"|"+inv.stacks.get(0)+"|"+f.argument+"|"+f.original+"|"+trace);
  }
 }
}
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home', type=Path, required=True)
    args = parser.parse_args()
    WORK.mkdir(parents=True, exist_ok=True)
    names = ['api/inventories/InternalInventory.java', 'util/inv/AppEngInternalInventory.java',
             'api/inventories/InternalInventoryItemHandler.java']
    source = {n: (upstream.SOURCE / 'src/main/java/appeng' / n).read_text() for n in names}
    insertion = method(source[names[0]], 'default ItemStack insertItem(')
    inventory = [method(source[names[1]], signature) for signature in
                 ['public ItemStack extractItem(', 'public void setItemDirect(', 'private void notifyContentsChanged(',
                  'protected void onContentsChanged(', 'protected boolean eventsEnabled(']]
    wrapper = method(source[names[2]], 'public ItemStack extractItem(')
    java = HOST.replace('__INSERT__', insertion).replace('__INV__', '\n'.join(inventory)).replace('__WRAPPER__', wrapper)
    path = WORK / 'InventoryAccessOracle.java'; path.write_text(java)
    subprocess.run([str(args.java_home / 'bin/javac'), '-proc:none', '-d', str(WORK), str(path)], check=True)
    rows = []
    for op, simulate, valid, extract, client, enabled, race, count in itertools.product(
            ['I', 'E', 'W'], range(2), range(2), range(2), [-1, 0, 1], range(2), [0, 1, 2, 4, 8, 15], [0, 1, 63, 64, 65]):
        rows.append([op, 0, 64, simulate, valid, extract, 64, client, enabled, race,
                     0, count, 64, 0, 0, 7, 64, 0, 0])
    rng = random.Random(904621)
    edges = [-2**31, -1, 0, 1, 2, 63, 64, 65, 99, 2**31 - 1]
    for _ in range(20000):
        rows.append([rng.choice(['I', 'E', 'W']), rng.choice([-1, 0, 0, 0, 1]), rng.choice(edges),
                     rng.randrange(2), rng.randrange(2), rng.randrange(2), rng.choice(edges), rng.choice([-1, 0, 1]),
                     rng.randrange(2), rng.randrange(16), rng.choice([-1, 0, 1, 3]), rng.choice(edges),
                     rng.choice([1, 16, 64, 99]), rng.randrange(3), rng.choice([-1, 0, 1, 3]), rng.choice(edges),
                     rng.choice([1, 16, 64, 99]), rng.randrange(3), rng.randrange(2)])
    lines = [' '.join(map(str, row)) for row in rows]
    data = ('\n'.join(lines) + '\n').encode(); (WORK / 'input.txt').write_bytes(data)
    expected = subprocess.run([str(args.java_home / 'bin/java'), '-cp', str(WORK), 'oracle.InventoryAccessOracle'],
                              input=data, capture_output=True)
    (WORK / 'java.stderr').write_bytes(expected.stderr); expected.check_returncode()
    actual = subprocess.run(['cargo', 'run', '--quiet', '--locked', '--example', 'inventory_access_oracle'],
                            cwd=ROOT, input=data, capture_output=True)
    (WORK / 'rust.stderr').write_bytes(actual.stderr); actual.check_returncode()
    (WORK / 'java.txt').write_bytes(expected.stdout); (WORK / 'rust.txt').write_bytes(actual.stdout)
    if expected.stdout != actual.stdout:
        for i, (a, b) in enumerate(itertools.zip_longest(expected.stdout.splitlines(), actual.stdout.splitlines())):
            if a != b:
                raise AssertionError(f'case {i}: {lines[i]}\nJava {a!r}\nRust {b!r}')
    sha = lambda b: hashlib.sha256(b).hexdigest()
    paths = ['mixins/ae2/inventory_access.rs', 'examples/inventory_access_oracle.rs',
             'examples/support/inventory_access_fixture.rs', 'tools/inventory_access_oracle.py']
    report = {'passed': True, 'cases': len(rows), 'source_sha256': {n: sha(v.encode()) for n, v in source.items()},
              'methods_sha256': {str(i): sha(v.encode()) for i, v in enumerate([insertion, *inventory, wrapper])},
              'binding_sha256': sha(java.encode()), 'rust_and_driver_sha256': {p: sha((ROOT / p).read_bytes()) for p in paths},
              'input_sha256': sha(data), 'output_sha256': sha(actual.stdout),
              'scope': 'Original insertion/extraction and guarded inventory notification methods; exact retained/copied stack identities, callback traces, wrapped gate and invalid-slot faults',
              'limits': ['Controlled stacks, components, filters, host and client/level bindings; no real Minecraft ItemStack or NeoForge capability implementation.',
                         'Native Chest automation, menus, ownership and world lifecycle are not activated by this standalone comparison.']}
    (ROOT / 'docs/inventory-access-oracle.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
