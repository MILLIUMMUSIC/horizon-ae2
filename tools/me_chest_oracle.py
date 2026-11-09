#!/usr/bin/env python3
"""Compare original ME Chest handler/power/input/display/menu callbacks.

Real original StoredEnergyAmount and StorageHelper run with controlled cells,
stacks, grid, registry/menu/world bindings and guarded inventory notifications.
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
from storage_menu_oracle import method
ROOT=upstream.ROOT
WORK=ROOT/'target/me-chest-oracle'
HOST=r'''
package oracle;
import java.io.*;import java.lang.reflect.*;import java.util.*;import java.util.function.*;
import com.google.common.primitives.Ints;import com.google.gson.*;
import oracle.MEChestOracle.GridPowerStorageStateChanged.PowerEventType;
public class MEChestOracle {
 static Ref f;
 @interface Nullable{}
 enum FluidAction{SIMULATE,EXECUTE}
 enum Actionable{SIMULATE,MODULATE;static Actionable of(FluidAction a){return a==FluidAction.EXECUTE?MODULATE:SIMULATE;}}
 enum PowerMultiplier{ONE,CONFIG;double multiplier=1;double multiply(double n){return n*multiplier;}double divide(double n){return n/multiplier;}}
 static class GridPowerStorageStateChanged{enum PowerEventType{PROVIDE_POWER,RECEIVE_POWER}GridPowerStorageStateChanged(Object s,PowerEventType t){}}
 static class Mth{
  static final Method CLAMP;static{try{CLAMP=Class.forName("ayo").getMethod("a",double.class,double.class,double.class);}catch(Exception e){throw new ExceptionInInitializerError(e);}}
  static double clamp(double n,double min,double max){try{return (double)CLAMP.invoke(null,n,min,max);}catch(Exception e){throw new RuntimeException(e);}}
 }
 __ENERGY__
 __HELPER__
 enum CellState{ABSENT,EMPTY,NOT_EMPTY,TYPES_FULL,FULL}
 static class AEKey{int id;AEKey(int id){this.id=id;}int getAmountPerOperation(){return id==2?125:1;}}
 static class AEItemKey extends AEKey{AEItemKey(int id){super(id);}static AEItemKey of(ItemStack s){return s.isEmpty()?null:(AEItemKey)KEYS[s.key];}}
 static final AEKey[] KEYS={new AEItemKey(0),new AEItemKey(1),new AEFluidKey(2)};
 static class ItemStack{static final ItemStack EMPTY=new ItemStack(-1,0);int key,count;ItemStack(int k,int n){key=k;count=n;}boolean isEmpty(){return key<0||count<=0;}int getCount(){return isEmpty()?0:count;}void shrink(int n){count-=n;}public String toString(){return isEmpty()?"-1/0":key+"/"+count;}}
 static class AEFluidKey extends AEKey{AEFluidKey(int id){super(id);}static AEKey of(FluidStack s){return s.n>0&&s.key==2?KEYS[2]:null;}}
 static class FluidStack{static final FluidStack EMPTY=new FluidStack(-1,0);int key,n;FluidStack(int key,int n){this.key=key;this.n=n;}int getAmount(){return n;}}
 static class FluidType{static final int BUCKET_VOLUME=1000;}
 interface IFluidTank{FluidStack getFluid();int getFluidAmount();int getCapacity();boolean isFluidValid(FluidStack s);}
 interface IFluidHandler{int getTanks();FluidStack getFluidInTank(int tank);int getTankCapacity(int tank);boolean isFluidValid(int tank,FluidStack s);int fill(FluidStack s,FluidAction action);FluidStack drain(FluidStack s,FluidAction a);FluidStack drain(int n,FluidAction a);}
 static class Player{}interface IActionSource{Optional<Player> player();}interface IEnergySource{double extractAEPower(double n,Actionable mode,PowerMultiplier multiplier);}
 enum AeStats{ItemsInserted,ItemsExtracted;void addToPlayer(Player p,int n){throw new AssertionError("machine source has no player");}}
 static class CompoundTag{}interface ICraftingRequester{}interface ICraftingLink{}static class CraftingLink implements ICraftingLink{CraftingLink(CompoundTag t,ICraftingRequester r){throw new UnsupportedOperationException();}}
 interface MEStorage{long insert(AEKey k,long n,Actionable a,IActionSource s);long extract(AEKey k,long n,Actionable a,IActionSource s);}
 static class DelegatingMEInventory implements MEStorage{
  MEStorage delegate;DelegatingMEInventory(MEStorage d){delegate=d;}public long insert(AEKey k,long n,Actionable a,IActionSource s){return delegate.insert(k,n,a,s);}public long extract(AEKey k,long n,Actionable a,IActionSource s){return delegate.extract(k,n,a,s);}
 }
 interface InternalInventory{}
 interface IAEItemFilter{boolean allowExtract(InternalInventory inv,int slot,int amount);boolean allowInsert(InternalInventory inv,int slot,ItemStack stack);}
 static class AppEngInternalInventory implements InternalInventory{
  Ref host;int id;boolean notifyingChanges;List<ItemStack> stacks=new ArrayList<>(List.of(ItemStack.EMPTY));AppEngInternalInventory(Ref host,int id){this.host=host;this.id=id;}
  ItemStack getStackInSlot(int i){return stacks.get(i);}boolean isEmpty(){return stacks.get(0).isEmpty();}boolean isEnableClientEvents(){return false;}
  __INVENTORY__
 }
 static class StorageCell implements MEStorage{
  int id;Runnable changed;StorageCell(int id,Runnable cb){this.id=id;changed=cb;}double getIdleDrain(){return id==11?.5:2.;}
  CellState getStatus(){long total=Arrays.stream(f.counts[id==11?0:1]).sum();return total==0?CellState.EMPTY:total>=f.cap?CellState.FULL:CellState.NOT_EMPTY;}
  void persist(){f.event("PERSIST:"+id);}
  public long insert(AEKey key,long n,Actionable mode,IActionSource source){if(source!=f.mySrc)throw new AssertionError();long[] counts=f.counts[id==11?0:1];long amount=Math.min(Math.max(n,0),Math.max(f.cap-counts[key.id],0));f.event("I:"+id+":"+key.id+":"+n+":"+(mode==Actionable.MODULATE?1:0)+":"+amount);if(mode==Actionable.MODULATE&&amount>0){counts[key.id]+=amount;if((f.race&1)!=0)f.inputInventory.stacks.set(0,new ItemStack(1,1));if((f.race&2)!=0)f.cellInventory.setItemDirect(0,new ItemStack(f.getCell().key==11?13:11,1));changed.run();}return amount;}
  public long extract(AEKey key,long n,Actionable mode,IActionSource source){if(source!=f.mySrc)throw new AssertionError();long[] counts=f.counts[id==11?0:1];long amount=Math.min(Math.max(n,0),Math.max(counts[key.id],0));f.event("X:"+id+":"+key.id+":"+n+":"+(mode==Actionable.MODULATE?1:0)+":"+amount);if(mode==Actionable.MODULATE&&amount>0){counts[key.id]-=amount;changed.run();}return amount;}
 }
 static class StorageCells{
  static StorageCell getCellInventory(ItemStack stack,Runnable cb){f.event("LOAD:"+stack);return !stack.isEmpty()&&(stack.key==11||stack.key==13)?new StorageCell(stack.key,cb):null;}
  static Object getHandler(ItemStack stack){f.event("HANDLER:"+stack);return !stack.isEmpty()&&stack.key>=11&&stack.key<=13?new Object():null;}
 }
 static class Grid implements IEnergySource{
  Grid getEnergyService(){return this;}
  public double extractAEPower(double n,Actionable mode,PowerMultiplier m){f.event("G:"+bits(n)+":"+(mode==Actionable.MODULATE?1:0)+":"+m.ordinal());double result=Math.min(Math.max(n,0),f.gridPower);if(mode==Actionable.MODULATE){f.gridPower-=result;if((f.race&4)!=0)PowerMultiplier.CONFIG.multiplier=2;}return result;}
  boolean isNetworkPowered(){return f.gridPowered;}void postEvent(GridPowerStorageStateChanged event){f.event("RECEIVE");}
 }
 static class Node{Grid getGrid(){return f.gridPresent?f.grid:null;}boolean isPowered(){return f.nodePowered;}boolean isOnline(){return f.nodeOnline;}void setIdlePowerUsage(double idle){f.event("IDLE:"+bits(idle));if((f.race&8)!=0)f.cellInventory.setItemDirect(0,ItemStack.EMPTY);}void ifPresent(Consumer<Grid> cb){if(f.gridPresent)cb.accept(f.grid);}}
 static class IGridNodeListener{enum State{ANY}}
 static class IStorageProvider{static void requestUpdate(Node n){f.event("STORAGE");}}
 interface IStorageMounts{void mount(StorageCell cell,int priority);default void mount(DelegatingMEInventory cell,int priority){mount((StorageCell)cell.delegate,priority);}}
 record ILinkStatus(int value){static ILinkStatus ofConnected(){return new ILinkStatus(2);}static ILinkStatus ofDisconnected(String text){return new ILinkStatus(text.equals("CELL")?0:1);}}
 enum PlayerMessages{ChestCannotReadStorageCell;String text(){return "CELL";}}enum GuiText{OutOfPower;String text(){return "POWER";}}
 static class BasicCellChestMenu{static final int TYPE=0;}static class MEChestMenu{static final int TYPE=1;}static class MenuLocators{static Object forBlockEntity(Ref r){return r;}}
 static class MenuOpener{static void open(int kind,Player p,Object locator){f.event("OPEN:"+kind);}}
 static class Level{void blockEntityChanged(Object pos){f.event("BLOCK");}}
 static class Root implements IEnergySource{
  StoredEnergyAmount stored;
  protected void emitPowerStateEvent(GridPowerStorageStateChanged.PowerEventType event){throw new AssertionError();}
  __POWER_WRAPPER__
  __LOCAL_SOURCE__
  protected double extractAEPower(double amt,Actionable mode){f.event("L:"+bits(amt)+":"+(mode==Actionable.MODULATE?1:0));double result=sourceLocalExtract(amt,mode);if(mode==Actionable.MODULATE&&(f.race&4)!=0)PowerMultiplier.CONFIG.multiplier=2;return result;}
  double getAECurrentPower(){return stored.getAmount();}
 }
 static class Ref extends Root{
  AppEngInternalInventory inputInventory=new AppEngInternalInventory(this,0),cellInventory=new AppEngInternalInventory(this,1);
  IActionSource mySrc=Optional::empty;Node node=new Node();Grid grid=new Grid();boolean gridPresent,gridPowered,nodePowered,nodeOnline,client;Level level;Object worldPosition=new Object();
  long[][] counts=new long[2][3];long cap;int race;double gridPower;boolean isCached,wasOnline;ChestMonitorHandler cellHandler;FluidHandler fluidHandler;double idlePowerUsage;int priority;CellState clientCellState=CellState.ABSENT;boolean clientPowered;
  __FLUID__
  Node getMainNode(){return node;}boolean isClientSide(){return client;}void event(String s){trace.append(s).append(';');}StringBuilder trace=new StringBuilder();
  void markForUpdate(){event("MARK");}void invalidateCapabilities(){event("CAP");}void saveChangedInventory(AppEngInternalInventory inv){event("SAVE:"+inv.id);}
  __METHODS__
  __MONITOR__
  __FILTERS__
  Ref(JsonObject v){f=this;PowerMultiplier.CONFIG.multiplier=dbl(v.get("factor"));gridPresent=v.get("grid").getAsBoolean();gridPowered=v.get("grid_powered").getAsBoolean();nodePowered=v.get("node_powered").getAsBoolean();nodeOnline=v.get("online").getAsBoolean();client=v.get("client").getAsBoolean();level=v.get("level").getAsBoolean()?new Level():null;cap=v.get("cap").getAsLong();race=v.get("race").getAsInt();gridPower=dbl(v.get("grid_energy"));stored=new StoredEnergyAmount(dbl(v.get("local")),500*PowerMultiplier.CONFIG.multiplier,e->{event("EVENT:"+e.ordinal());emitPowerStateEvent(e);});var c=v.getAsJsonArray("counts");for(int i=0;i<2;i++)for(int k=0;k<3;k++)counts[i][k]=c.get(i).getAsJsonArray().get(k).getAsLong();cellInventory.stacks.set(0,stack(v.getAsJsonArray("cell")));inputInventory.stacks.set(0,stack(v.getAsJsonArray("input")));}
 }
 static String bits(double n){return Long.toUnsignedString(Double.doubleToLongBits(n),16);}static double dbl(JsonElement n){return Double.longBitsToDouble(Long.parseUnsignedLong(n.getAsString(),16));}static ItemStack stack(JsonArray a){return new ItemStack(a.get(0).getAsInt(),a.get(1).getAsInt());}
 public static void main(String[] args)throws Exception{var in=new BufferedReader(new InputStreamReader(System.in));String line;while((line=in.readLine())!=null){var v=JsonParser.parseString(line).getAsJsonObject();var r=new Ref(v);var results=new StringBuilder();for(var value:v.getAsJsonArray("ops")){var a=value.getAsJsonArray();String op=a.get(0).getAsString();String result="-";switch(op){
 case "U"->r.openCellInventoryMenu(new Player());case "V"->result=r.fluidHandler==null?"-":""+r.fluidHandler.getCapacity()+":"+(r.fluidHandler.isFluidValid(FluidStack.EMPTY)?1:0)+":"+r.fluidHandler.getTankCapacity(a.get(1).getAsInt())+":"+(r.fluidHandler.isFluidValid(a.get(1).getAsInt(),FluidStack.EMPTY)?1:0);case "Z"->result=""+(r.fluidHandler==null?0:r.fluidHandler.fill(new FluidStack(a.get(1).getAsInt(),a.get(2).getAsInt()),a.get(3).getAsBoolean()?FluidAction.EXECUTE:FluidAction.SIMULATE));case "H"->r.updateHandler();case "T"->r.serverTick();case "D"->r.recalculateDisplay();case "C"->r.cellInventory.setItemDirect(0,new ItemStack(a.get(1).getAsInt(),a.get(2).getAsInt()));case "I"->r.inputInventory.setItemDirect(0,new ItemStack(a.get(1).getAsInt(),a.get(2).getAsInt()));
 case "F"->result=r.new InputInventoryFilter().allowInsert(r.inputInventory,0,new ItemStack(a.get(1).getAsInt(),a.get(2).getAsInt()))?"1":"0";case "L"->result=""+r.getLinkStatus().value();case "Q"->r.setPriority(a.get(1).getAsInt());case "N"->{r.nodeOnline=a.get(1).getAsBoolean();r.onMainNodeStateChanged(IGridNodeListener.State.ANY);}case "M"->r.mountInventories((cell,p)->r.event("MOUNT:"+cell.id+":"+p));case "O"->result=r.openGui(new Player())?"1":"0";case "P"->r.stored.setStored(dbl(a.get(1)));
 case "E"->result=bits(r.extractAEPower(dbl(a.get(1)),a.get(2).getAsBoolean()?Actionable.MODULATE:Actionable.SIMULATE,PowerMultiplier.values()[a.get(3).getAsInt()]));
 case "X","Y"->{r.updateHandler();result=""+(r.cellHandler==null?0:op.equals("X")?r.cellHandler.extract(KEYS[a.get(1).getAsInt()],a.get(2).getAsLong(),a.get(3).getAsBoolean()?Actionable.MODULATE:Actionable.SIMULATE,r.mySrc):r.cellHandler.insert(KEYS[a.get(1).getAsInt()],a.get(2).getAsLong(),a.get(3).getAsBoolean()?Actionable.MODULATE:Actionable.SIMULATE,r.mySrc));}
 default->throw new AssertionError(op);}
 results.append(result).append(',');}
 System.out.println(results+"|"+r.trace+"|"+(r.cellHandler==null?-1:r.cellHandler.cellInventory.id)+":"+(r.isCached?1:0)+":"+(r.fluidHandler==null?0:1)+":"+bits(r.idlePowerUsage)+":"+r.priority+":"+r.clientCellState.ordinal()+":"+(r.clientPowered?1:0)+":"+(r.wasOnline?1:0)+"|"+r.getCell()+"|"+r.inputInventory.getStackInSlot(0)+"|"+Arrays.deepToString(r.counts)+"|"+bits(r.stored.getAmount())+":"+bits(r.gridPower)+":"+bits(PowerMultiplier.CONFIG.multiplier));
 }}
}
'''

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--java-home',type=Path,required=True);args=parser.parse_args();WORK.mkdir(parents=True,exist_ok=True)
    paths=['blockentity/storage/MEChestBlockEntity.java','api/storage/StorageHelper.java','me/energy/StoredEnergyAmount.java','util/inv/AppEngInternalInventory.java','blockentity/powersink/AEBasePoweredBlockEntity.java']
    originals={p:(upstream.SOURCE/'src/main/java/appeng'/p).read_text() for p in paths};chest=originals[paths[0]]
    signatures=['public ItemStack getCell(', 'protected void emitPowerStateEvent(', 'private void recalculateDisplay(', 'private void updateHandler(', 'private ChestMonitorHandler wrap(', 'public ILinkStatus getLinkStatus(', 'public CellState getCellStatus(', 'public boolean isPowered(', 'protected double extractAEPower(', 'public void serverTick(', 'public void onMainNodeStateChanged(', 'public void onChangeInventory(', 'private void tryToStoreContents(', 'public void mountInventories(', 'public void setPriority(', 'private void blinkCell(', 'public boolean openGui(', 'private void onCellContentChanged(', 'public void openCellInventoryMenu(']
    methods=[method(chest,n) for n in signatures]
    inv=[method(originals[paths[3]],n) for n in ['public void setItemDirect(', 'private void notifyContentsChanged(', 'protected void onContentsChanged(', 'protected boolean eventsEnabled(']]
    energy=originals[paths[2]];energy=energy[energy.index('public final class StoredEnergyAmount'):].replace('public final class StoredEnergyAmount','public static final class StoredEnergyAmount',1)
    helper=originals[paths[1]];helper=helper[helper.index('public final class StorageHelper'):].replace('public final class StorageHelper','public static final class StorageHelper',1)
    java=HOST.replace('__ENERGY__',energy).replace('__HELPER__',helper).replace('__INVENTORY__','\n'.join(inv)).replace('__METHODS__','\n'.join(methods)).replace('__MONITOR__',method(chest,'private class ChestMonitorHandler ')).replace('__FILTERS__',method(chest,'private class InputInventoryFilter '))
    fluid=method(chest,'private class FluidHandler ')
    java=java.replace('__FLUID__',fluid.replace('{','{FluidHandler(){f.event("FLUID");}',1))
    wrapper=method(originals[paths[4]],'public final double extractAEPower(')
    local_method=method(originals[paths[4]],'protected double extractAEPower(')
    java=java.replace('__POWER_WRAPPER__',wrapper).replace('__LOCAL_SOURCE__',local_method.replace('extractAEPower(', 'sourceLocalExtract(',1))
    java=java.replace('MEChestBlockEntity.this','Ref.this')
    path=WORK/'MEChestOracle.java';path.write_text(java);cp=minecraft_reference.prepare();subprocess.run([str(args.java_home/'bin/javac'),'-proc:none','-cp',cp,'-d',str(WORK),str(path)],cwd=WORK,check=True)
    bits=lambda n:struct.pack('>d',n).hex()
    base={'factor':bits(1.),'grid':False,'grid_powered':False,'node_powered':False,'online':False,'client':False,'level':True,'cap':128,'race':0,'grid_energy':bits(500.),'local':bits(100.),'counts':[[5,0,1000],[0,10,0]],'cell':[11,1],'input':[0,4]}
    program=[['V',0],['Z',2,100,True],['H'],['V',0],['L'],['F',0,3],['M'],['T'],['O'],['E',bits(200.),True,1],['I',1,7],['C',13,1],['Q',-17],['V',0],['Z',2,1000,False],['N',True],['M'],['Y',2,100,True],['X',1,10,True],['C',12,1],['V',0],['Z',2,1000,True],['H'],['H'],['O'],['L'],['C',-1,0],['T'],['D'],['U']]
    rows=[]
    for cell,local,grid,flags,race,factor in itertools.product([-1,10,11,12,13],[0.,1.,1.1,500.],[False,True],range(4),[0,1,2,4,8],[.5,1.,2.]):
        rows.append({**base,'cell':[cell,1],'local':bits(local if local<=500*factor else 500*factor),'grid':grid,'grid_powered':bool(flags&1),'node_powered':bool(flags&2),'online':bool(flags&2),'race':race,'factor':bits(factor),'ops':program})
    rng=random.Random(512887)
    for _ in range(1600):
        ops=[]
        for _ in range(rng.randrange(1,35)):
            op=rng.choice(['H','T','D','C','I','F','L','Q','N','M','O','P','E','X','Y','Z','V','U'])
            if op=='V':argsop=[rng.choice([-1,0,1,2147483647])]
            elif op=='Z':argsop=[rng.choice([-1,2]),rng.choice([0,1,125,1000,2147483647]),bool(rng.randrange(2))]
            elif op=='C':argsop=[rng.choice([-1,10,11,12,13]),rng.choice([0,1,2])]
            elif op in ['I','F']:argsop=[rng.randrange(2),rng.choice([0,1,2,16,64])]
            elif op=='Q':argsop=[rng.choice([-2147483648,-1,0,7,2147483647])]
            elif op=='N':argsop=[bool(rng.randrange(2))]
            elif op=='P':argsop=[bits(rng.choice([0.,1e-8,1e-6,.1,1.,1.1,250.,1000.]))]
            elif op=='E':argsop=[bits(rng.choice([-1.,0.,1e-8,1e-6,.1,1.,10.,500.,1000.])),bool(rng.randrange(2)),rng.randrange(2)]
            elif op in ['X','Y']:argsop=[rng.randrange(3),rng.choice([0,1,125,1000,9223372036854775807]),bool(rng.randrange(2))]
            else:argsop=[]
            ops.append([op,*argsop])
        factor=rng.choice([.5,1.,2.]);rows.append({**base,'factor':bits(factor),'grid':bool(rng.randrange(2)),'grid_powered':bool(rng.randrange(2)),'node_powered':bool(rng.randrange(2)),'online':bool(rng.randrange(2)),'client':rng.randrange(20)==0,'level':True,'race':rng.randrange(16),'cell':[rng.choice([-1,10,11,12,13]),1],'input':[rng.randrange(2),rng.choice([0,1,64])],'counts':[[rng.choice([0,1,127,128,1000]) for _ in range(3)] for _ in range(2)],'local':bits(rng.choice([0.,.1,1.,100.,250.])),'grid_energy':bits(rng.choice([0.,1.,10.,1000.])),'ops':ops})
    rows.extend({**base,'level':False,'ops':[['C',cell,1],['H'],['Q',7],['C',-1,0]]} for cell in [-1,10,11,12,13])
    data=('\n'.join(json.dumps(r,separators=(',',':')) for r in rows)+'\n').encode();(WORK/'input.txt').write_bytes(data)
    expected=subprocess.run([str(args.java_home/'bin/java'),'-cp',str(WORK)+':'+cp,'oracle.MEChestOracle'],cwd=WORK,input=data,capture_output=True);(WORK/'java.stderr').write_bytes(expected.stderr);expected.check_returncode()
    actual=subprocess.run(['cargo','run','--quiet','--locked','--example','me_chest_oracle'],cwd=ROOT,input=data,capture_output=True);(WORK/'rust.stderr').write_bytes(actual.stderr);actual.check_returncode()
    (WORK/'java.txt').write_bytes(expected.stdout);(WORK/'rust.txt').write_bytes(actual.stdout)
    if expected.stdout!=actual.stdout:
        for i,(a,b) in enumerate(itertools.zip_longest(expected.stdout.splitlines(),actual.stdout.splitlines())):
            if a!=b:raise AssertionError(f'case {i}: {rows[i]}\nJava {a!r}\nRust {b!r}')
    sha=lambda b:hashlib.sha256(b).hexdigest()
    report={'passed':True,'cases':len(rows),'operations':sum(len(r['ops']) for r in rows),'callbacks':sum(line.split(b'|')[1].count(b';') for line in actual.stdout.splitlines()),'source_sha256':{p:sha(s.encode()) for p,s in originals.items()},'method_sha256':dict(zip(signatures,map(lambda s:sha(s.encode()),methods)))|{'base_multiplier_wrapper':sha(wrapper.encode()),'base_local_extract':sha(local_method.encode()),'fluid_handler':sha(fluid.encode())},'reference_sha256':{str(Path(p).relative_to(ROOT)):sha(Path(p).read_bytes()) for p in cp.split(':')},'binding_sha256':sha(java.encode()),'rust_sha256':{p:sha((ROOT/p).read_bytes()) for p in ['mixins/ae2/me_chest.rs','examples/me_chest_oracle.rs','examples/support/me_chest_fixture.rs','mixins/ae2/storage_helper.rs','mixins/ae2/energy.rs']},'input_sha256':sha(data),'output_sha256':sha(actual.stdout),'scope':'Original ME Chest handler/cache, grid-first local fallback power, input/filter, monitor, fluid fill/capacity predicates, display/link, priority/node/mount/menu callback programs with original StorageHelper, StoredEnergyAmount/Mth and guarded AppEng inventory notification methods', 'limits':['Controlled registry/cell/grid/world/player/menu bindings and identity-preserving stacks; no native registration/UI/capability/lifecycle proof','Transfers run with an attached level; source pre-level content callback faults are excluded','Finite positive CONFIG factors and finite requests/local storage; source nonfinite energy edge behavior is not established by this comparison','StorageCell contents/capacity/status/idle/persistence are controlled; independent cell/backend proofs remain separate','Client stream/NBT/visual/color/config codecs, native fluid capability transport/lifetime and original block activation/orientation are not executed']}
    (ROOT/'docs/me-chest-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
