#!/usr/bin/env python3
"""Compare original MEStorageMenu interaction and container-transfer callbacks.

Original methods execute with controlled stack object identity, player/slot,
container-item and storage/power bindings. No native menu/UI/permissions or real
platform container strategies are implied by these controlled-world traces.
"""
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import random
import struct
import subprocess
import minecraft_reference
import upstream
ROOT=upstream.ROOT
WORK=ROOT/'target/storage-menu-oracle'
HOST=r'''
package oracle;
import java.io.*;import java.util.*;import java.util.function.*;import java.util.stream.*;
import com.google.common.primitives.Ints;import com.google.common.collect.BiMap;import com.google.common.collect.HashBiMap;
import com.google.gson.*;
public class StorageMenuOracle {
 static Fixture f;
 @interface Nullable {}
 enum Actionable {SIMULATE,MODULATE}enum PowerMultiplier {CONFIG}
 __ACTIONS__
 static class AEKey {int id;AEKey(int id){this.id=id;}int getAmountPerOperation(){return id==2?125:1;}int getAmountPerUnit(){return id==2?1000:1;}public int hashCode(){return id;}public boolean equals(Object o){return o instanceof AEKey k&&id==k.id;}}
 static class AEItemKey extends AEKey {
  AEItemKey(int id){super(id);}static AEItemKey of(ItemStack s){return s.isEmpty()?null:(AEItemKey)KEYS[s.key];}static AEItemKey of(Item item){return (AEItemKey)KEYS[3];}
  int getMaxStackSize(){return maxCount(id);}boolean matches(ItemStack stack){return !stack.isEmpty()&&stack.key==id;}ItemStack toStack(){return toStack(1);}ItemStack toStack(int n){return new ItemStack(id,n);}ItemStack getReadOnlyStack(){return toStack();}
 }
 static class AEFluidKey extends AEKey {AEFluidKey(){super(2);}Fluid getFluid(){return new Fluid();}}
 static final AEKey[] KEYS={new AEItemKey(0),new AEItemKey(1),new AEFluidKey(),new AEItemKey(3),new AEItemKey(4)};
 static int maxCount(int k){return k==1||k==3?16:k==4?1:64;}static int itemId(int k){return k==0||k==1?0:k==3?1:k==4?2:-1;}
 static class Item {int id;Item(int id){this.id=id;}ItemStack getDefaultInstance(){return new ItemStack(3,1);}}
 static class Items {static final Item AIR=new Item(-1),IRON=new Item(0),BUCKET=new Item(1),WATER_BUCKET=new Item(2);}
 static class Fluid {Item getBucket(){return Items.WATER_BUCKET;}}
 static class ItemStack {
  static final ItemStack EMPTY=new ItemStack(-1,0);int key,count;ItemStack(int k,int n){key=k;count=n;}
  boolean isEmpty(){return key<0||count<=0;}int getCount(){return isEmpty()?0:count;}int getMaxStackSize(){return maxCount(key);}
  Item getItem(){int id=isEmpty()?-1:itemId(key);return id==0?Items.IRON:id==1?Items.BUCKET:id==2?Items.WATER_BUCKET:Items.AIR;}
  boolean is(Item item){return getItem()==item;}ItemStack copy(){return new ItemStack(key,count);}ItemStack copyWithCount(int n){var copy=copy();copy.setCount(n);return copy;}
  void setCount(int n){count=n;}void grow(int n){count+=n;}void shrink(int n){count-=n;}public String toString(){return isEmpty()?"-1/0":key+"/"+count;}
 }
 interface MEStorage {long extract(AEKey key,long amount,Actionable mode,IActionSource source);long insert(AEKey key,long amount,Actionable mode,IActionSource source);}
 interface IEnergySource {double extractAEPower(double amount,Actionable mode,PowerMultiplier multiplier);}
 interface IActionSource {Optional<ServerPlayer> player();}
 enum AeStats {ItemsExtracted,ItemsInserted;void addToPlayer(ServerPlayer p,int n){f.event("T:"+(this==ItemsExtracted?"Extracted":"Inserted")+":"+n);}}
 static class CompoundTag {}interface ICraftingRequester {}interface ICraftingLink {}static class CraftingLink implements ICraftingLink {CraftingLink(CompoundTag t,ICraftingRequester r){throw new UnsupportedOperationException();}}
 __HELPER__
 __UPDATES__
 static class Platform { __PLATFORM__ }
 static class Abilities {boolean instabuild;}
 static class ServerPlayer {Inventory getInventory(){return f.inv;}Abilities getAbilities(){var a=new Abilities();a.instabuild=f.creative;return a;}boolean addItem(ItemStack stack){return f.add(stack);}}
 static class Inventory {ServerPlayer player=new ServerPlayer();List<ItemStack> items=new ArrayList<>(List.of(ItemStack.EMPTY,ItemStack.EMPTY));}
 static class Slot {int index;Slot(int i){index=i;}boolean mayPlace(ItemStack stack){return true;}ItemStack getItem(){return f.inv.items.get(index);}int getMaxStackSize(ItemStack s){return s.getMaxStackSize();}void setByPlayer(ItemStack s){f.inv.items.set(index,s);f.event("SI:"+index+":"+s);}}
 interface ServerboundPacket {}record MEInteractionPacket(int containerId,long serial,InventoryAction action) implements ServerboundPacket {}
 static class PacketDistributor {static void sendToServer(ServerboundPacket p){throw new UnsupportedOperationException("server program only");}}
 static class CraftAmountMenu {static void open(ServerPlayer player,Object locator,AEKey key,int amount){f.event("O:"+key.id+":"+amount);}}
 static class AELog {
  static void warn(String message,Object... args){f.event("U:"+((InventoryAction)args[0]).ordinal());}
  static void error(String message,Object... args){String kind=message.startsWith("Unable to pull")?"CouldNotExtractFluid":message.startsWith("Fluid item")?"FluidExtractionChanged":"CouldNotInsertFluid";f.event("ERR:"+kind);}
 }
 record GenericStack(AEKey what,long amount) {}
 interface ContainerItemContext {
  long insert(AEKey key,long n,Actionable mode);long extract(AEKey key,long n,Actionable mode);GenericStack getExtractableContent();void playFillSound(ServerPlayer p,AEKey key);void playEmptySound(ServerPlayer p,AEKey key);
 }
 static class ContainerItemStrategies {
  static ContainerItemContext findCarriedContextForKey(AEKey key,ServerPlayer p,MEStorageMenu menu){f.event("F:"+(key==null?-1:key.id));return f.kind!=0&&!f.cursor.isEmpty()&&(key==null||key.id==2)?f.container:null;}
  static ContainerItemContext findCarriedContext(Object type,ServerPlayer p,MEStorageMenu menu){f.event("N");return f.kind!=0&&!f.cursor.isEmpty()?f.container:null;}
 }
 static class MEStorageMenu {
  int containerId=1;static final int MAX_CONTAINER_TRANSFER_ITERATIONS=256;IncrementalUpdateHelper updateHelper=new IncrementalUpdateHelper();
  MEStorage storage;IEnergySource energySource;MEStorageMenu(){for(AEKey k:KEYS)updateHelper.getOrAssignSerial(k);storage=f.storage;energySource=f.energySource;}
  boolean isClientSide(){return false;}boolean canInteractWithGrid(){f.event("L:"+(f.connected?1:0));return f.connected;}
  Inventory getPlayerInventory(){return f.inv;}ServerPlayer getPlayer(){return f.inv.player;}IActionSource getActionSource(){return f.source;}
  AEKey getStackBySerial(long n){return updateHelper.getBySerial(n);}Object getLocator(){return f.locator?this:null;}
  ItemStack getCarried(){return f.cursor;}void setCarried(ItemStack stack){f.cursor=stack;f.event("C:"+stack);}
  List<Slot> getQuickMoveDestinationSlots(ItemStack stack,boolean fromPlayer){f.event("DEST:"+stack);return List.of(new Slot(0),new Slot(1));}
  int getPlaceableAmount(Slot slot,AEItemKey key){int n=sourcePlaceable(slot,key);f.event("PLACE:"+slot.index+":"+key.id+":"+n);return n;}
  __PLACEABLE__
  interface FillingSource {long extract(long amount,Actionable mode);}interface EmptyingSink {long insert(AEKey what,long amount,Actionable mode);}
  __METHODS__
 }
 static class Fixture {
  StringBuilder trace=new StringBuilder();ItemStack cursor;Inventory inv=new Inventory();long[] counts=new long[5];double energy;boolean creative,connected,locator,player;int kind,race;long tank,cap;
  IActionSource source=()->player?Optional.of(inv.player):Optional.empty();
  void event(String text){trace.append(text).append(';');}
  boolean add(ItemStack stack){event("ADD:"+stack);for(boolean empty:List.of(false,true))for(int i=0;i<2;i++){
   ItemStack slot=inv.items.get(i);if(slot.isEmpty()!=empty)continue;if(slot.isEmpty()||slot.key==stack.key){int amount=Math.max(0,Math.min(stack.getCount(),stack.getMaxStackSize()-slot.getCount()));if(amount>0){if(slot.isEmpty())inv.items.set(i,new ItemStack(stack.key,amount));else slot.count+=amount;stack.count-=amount;}}
  }return stack.isEmpty();}
  MEStorage storage=new MEStorage(){
   public long extract(AEKey key,long n,Actionable mode,IActionSource s){if(s!=source)throw new AssertionError();long result=Math.min(Math.max(n,0),Math.max(counts[key.id],0));event("E:"+key.id+":"+n+":"+(mode==Actionable.MODULATE?1:0)+":"+result);if(mode==Actionable.MODULATE)counts[key.id]-=result;return result;}
   public long insert(AEKey key,long n,Actionable mode,IActionSource s){if(s!=source)throw new AssertionError();long result=Math.min(Math.max(n,0),Math.max(1000000-counts[key.id],0));event("I:"+key.id+":"+n+":"+(mode==Actionable.MODULATE?1:0)+":"+result);if(mode==Actionable.MODULATE)counts[key.id]+=result;return result;}
  };
  IEnergySource energySource=(n,mode,multiplier)->{
   if(multiplier!=PowerMultiplier.CONFIG)throw new AssertionError();event("P:"+Long.toUnsignedString(Double.doubleToRawLongBits(n),16)+":"+(mode==Actionable.MODULATE?1:0));double result=n<=0?0:Math.min(n,energy);
   if(mode==Actionable.MODULATE){energy-=result;if((race&1)!=0)cursor=new ItemStack(1,1);if((race&2)!=0)for(int i=0;i<5;i++)counts[i]/=2;if((race&4)!=0)inv.items.set(0,new ItemStack(1,1));}return result;
  };
  ContainerItemContext container=new ContainerItemContext(){
   public GenericStack getExtractableContent(){long amount=kind==1?(cursor.key==4?1000:0):tank;event("GET:"+amount);return new GenericStack(KEYS[2],amount);}
   public long insert(AEKey key,long n,Actionable mode){long result=key.id!=2?0:kind==1?(cursor.is(Items.BUCKET)&&n>=1000?1000:0):Math.min(Math.max(n,0),Math.max(cap-tank,0));event("CI:"+n+":"+(mode==Actionable.MODULATE?1:0)+":"+result);if(mode==Actionable.MODULATE&&result!=0){if(kind==1){cursor=new ItemStack(4,1);event("C:"+cursor);}else tank+=result;}return result;}
   public long extract(AEKey key,long n,Actionable mode){long available=kind==1?(cursor.key==4?1000:0):tank;long result=kind==1?(n>=1000?available:0):Math.min(Math.max(n,0),Math.max(available,0));if(mode==Actionable.MODULATE&&kind==3&&result>0)--result;event("CE:"+n+":"+(mode==Actionable.MODULATE?1:0)+":"+result);if(mode==Actionable.MODULATE&&result!=0){if(kind==1){cursor=new ItemStack(3,1);event("C:"+cursor);}else tank-=result;}return result;}
   public void playFillSound(ServerPlayer player,AEKey key){event("SF");}public void playEmptySound(ServerPlayer player,AEKey key){event("SE");}
  };
  Fixture(JsonObject v){for(int i=0;i<5;i++)counts[i]=v.getAsJsonArray("counts").get(i).getAsLong();energy=Double.longBitsToDouble(Long.parseUnsignedLong(v.get("energy").getAsString(),16));creative=v.get("creative").getAsBoolean();connected=v.get("connected").getAsBoolean();locator=v.get("locator").getAsBoolean();player=v.get("player").getAsBoolean();kind=v.get("kind").getAsInt();race=v.get("race").getAsInt();tank=v.get("tank").getAsLong();cap=v.get("cap").getAsLong();var c=v.getAsJsonArray("cursor");cursor=new ItemStack(c.get(0).getAsInt(),c.get(1).getAsInt());var slots=v.getAsJsonArray("slots");for(int i=0;i<2;i++)inv.items.set(i,new ItemStack(slots.get(i).getAsJsonArray().get(0).getAsInt(),slots.get(i).getAsJsonArray().get(1).getAsInt()));}
  String finish(boolean fault){return (fault?"NullFluidKey":"OK")+"|"+trace+"|"+cursor+"|"+inv.items.get(0)+","+inv.items.get(1)+"|"+Arrays.stream(counts).mapToObj(Long::toString).collect(Collectors.joining(","))+"|"+Long.toUnsignedString(Double.doubleToRawLongBits(energy),16)+"|"+tank;}
 }
 public static void main(String[] args)throws Exception{var in=new BufferedReader(new InputStreamReader(System.in));String line;while((line=in.readLine())!=null){var value=JsonParser.parseString(line).getAsJsonObject();f=new Fixture(value);var menu=new MEStorageMenu();boolean fault=false;try{menu.handleInteraction(value.get("serial").getAsLong(),InventoryAction.values()[value.get("action").getAsInt()]);}catch(NullPointerException ex){fault=true;}System.out.println(f.finish(fault));}}
}
'''

def method(source,needle):
    start=source.index(needle);brace=source.index('{',start);depth=0;i=brace;state='code'
    while i<len(source):
        c=source[i];n=source[i:i+2]
        if state=='line':
            if c=='\n':state='code'
        elif state=='comment':
            if n=='*/':state='code';i+=1
        elif state in ['"',"'"]:
            if c=='\\':i+=1
            elif c==state:state='code'
        elif n=='//':state='line';i+=1
        elif n=='/*':state='comment';i+=1
        elif c in ['"',"'"]:state=c
        elif c=='{':depth+=1
        elif c=='}':
            depth-=1
            if depth==0:return source[start:i+1]
        i+=1
    raise ValueError('unclosed source method')

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--java-home',type=Path,required=True);args=parser.parse_args();WORK.mkdir(parents=True,exist_ok=True)
    paths=['menu/me/common/MEStorageMenu.java','menu/AEBaseMenu.java','helpers/InventoryAction.java','api/storage/StorageHelper.java','menu/me/common/IncrementalUpdateHelper.java','util/Platform.java']
    originals={n:(upstream.SOURCE/'src/main/java/appeng'/n).read_text() for n in paths}
    methods=[method(originals[paths[0]],needle) for needle in ['public final void handleInteraction(', 'protected void handleNetworkInteraction(', 'private void tryFillContainerItem(', 'protected void putCarriedItemIntoNetwork(', 'private boolean moveOneStackToPlayer(']]
    methods += [method(originals[paths[1]],needle) for needle in ['protected final void handleFillingHeldItem(', 'protected final void handleEmptyHeldItem(']]
    helper=originals[paths[3]];helper=helper[helper.index('public final class StorageHelper'):].replace('public final class StorageHelper','public static final class StorageHelper',1)
    updates=originals[paths[4]];updates=updates[updates.index('public class IncrementalUpdateHelper'):].replace('public class IncrementalUpdateHelper','public static class IncrementalUpdateHelper',1)
    actions=originals[paths[2]];actions=actions[actions.index('public enum InventoryAction'):]
    platform='\n'.join(method(originals[paths[5]],needle) for needle in ['public static ItemStack getInsertionRemainder(', 'public static ItemStack copyStackWithSize('])
    placeable=method(originals[paths[1]],'protected int getPlaceableAmount(').replace('protected int getPlaceableAmount(','protected int sourcePlaceable(',1)
    java=HOST.replace('__HELPER__',helper).replace('__UPDATES__',updates).replace('__ACTIONS__',actions).replace('__PLATFORM__',platform).replace('__PLACEABLE__',placeable).replace('__METHODS__','\n'.join(methods))
    path=WORK/'StorageMenuOracle.java';path.write_text(java);cp=minecraft_reference.prepare()
    subprocess.run([str(args.java_home/'bin/javac'),'-proc:none','-cp',cp,'-d',str(WORK),str(path)],cwd=WORK,check=True)
    bits=lambda n:struct.pack('>d',n).hex();rows=[]
    base={'counts':[100,30,10000,3,0],'energy':bits(10000.),'cursor':[-1,0],'slots':[[-1,0],[-1,0]],'connected':True,'creative':False,'locator':True,'player':True,'kind':0,'tank':0,'cap':5000,'race':0}
    for action,serial,cursor,kind,race in itertools.product(range(21),[-1,0,1,2,3,99],[[-1,0],[0,1],[0,64],[1,3],[3,1],[4,1]],range(4),[0,1,2,4]):
        row={**base,'action':action,'serial':serial,'cursor':cursor,'kind':kind,'race':race,'tank':2000 if kind>=2 else 0}
        rows.append(json.dumps(row,separators=(',',':')))
    rng=random.Random(912820)
    for _ in range(6000):
        row={**base,'action':rng.randrange(21),'serial':rng.choice([-1,0,1,2,3,4,5,99]),'connected':rng.randrange(10)!=0,'creative':bool(rng.randrange(2)),'locator':bool(rng.randrange(2)),'player':bool(rng.randrange(2)),
             'cursor':[rng.choice([-1,0,1,3,4]),rng.choice([0,1,2,15,16,63,64])],'counts':[rng.choice([0,1,2,16,64,1000,1000000]) for _ in range(5)],'slots':[[rng.choice([-1,0,1,3,4]),rng.choice([0,1,15,16,63,64])] for _ in range(2)],
             'kind':rng.randrange(4),'race':rng.randrange(8),'tank':rng.choice([0,1,999,1000,2000,1000000]),'cap':rng.choice([0,1,1000,5000,1000000]),'energy':bits(rng.choice([0.,.1,.11,1.,2.1,1000.,10000000.,float('inf'),float('nan')]))}
        if row['cursor'][0] in [3,4]:row['cursor'][1]=1
        rows.append(json.dumps(row,separators=(',',':')))
    data=('\n'.join(rows)+'\n').encode();(WORK/'input.txt').write_bytes(data)
    expected=subprocess.run([str(args.java_home/'bin/java'),'-cp',str(WORK)+':'+cp,'oracle.StorageMenuOracle'],cwd=WORK,input=data,capture_output=True)
    (WORK/'java.stderr').write_bytes(expected.stderr);expected.check_returncode()
    actual=subprocess.run(['cargo','run','--quiet','--locked','--example','storage_menu_oracle'],cwd=ROOT,input=data,capture_output=True)
    (WORK/'rust.stderr').write_bytes(actual.stderr);actual.check_returncode()
    (WORK/'java.txt').write_bytes(expected.stdout);(WORK/'rust.txt').write_bytes(actual.stdout)
    if actual.stdout!=expected.stdout:
        for i,(a,b) in enumerate(itertools.zip_longest(expected.stdout.splitlines(),actual.stdout.splitlines())):
            if a!=b:raise AssertionError(f'case {i}: {rows[i]}\nJava {a!r}\nRust {b!r}')
    sha=lambda b:hashlib.sha256(b).hexdigest()
    report={'passed':True,'cases':len(rows),'source_sha256':{n:sha(s.encode()) for n,s in originals.items()},'method_sha256':{str(i):sha(s.encode()) for i,s in enumerate(methods)},'binding_sha256':sha(java.encode()),'rust_sha256':{n:sha((ROOT/n).read_bytes()) for n in ['mixins/ae2/storage_menu.rs','mixins/ae2/storage_helper.rs','examples/storage_menu_oracle.rs','examples/support/storage_menu_fixture.rs']},'input_sha256':sha(data),'output_sha256':sha(actual.stdout),
        'scope':'Original server interaction/null/serial gates, item actions, carried remainder, quick-move program, bucket fallback, inherited fill/empty loops, actual StorageHelper and update helper; ordered callback traces and resulting cursor/slots/storage/power/container state',
        'limits':['Controlled key/stack/player/slot/container strategies, storage and energy callbacks; no actual native menus, platform capabilities, permission or whole-world behavior', 'Opaque stack handles preserve object identity under controlled reentrant callbacks; a source null-key filling fault maps to typed NullFluidKey rejection', 'Two controlled destination slots/player slots and controlled generic bucket/tank callbacks; full platform slot/semantic/capability implementations remain separate', 'Client dispatch branch, actual player stats/sounds/crafting UI and registry codecs are not executed']}
    (ROOT/'docs/storage-menu-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
