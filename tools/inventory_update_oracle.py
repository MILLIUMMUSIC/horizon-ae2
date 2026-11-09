#!/usr/bin/env python3
"""Compare original inventory update builder/wire and Repo entry mutation rules.

AEKey's registry/component wire codec is controlled; MC buffer varints are real.
Original HashSet keys use unique small integer hashes, with ascending iteration.
"""
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import random
import subprocess
import minecraft_reference
import upstream
from storage_menu_oracle import method
ROOT=upstream.ROOT
WORK=ROOT/'target/inventory-update-oracle'
HOST=r'''
package oracle;
import java.io.*;import java.lang.reflect.*;import java.util.*;import java.util.function.*;
import com.google.common.collect.*;import com.google.gson.*;import io.netty.buffer.*;
public class InventoryUpdateOracle {
 @interface Nullable {} @interface NotNull {}enum Dist{CLIENT}@interface OnlyIn{Dist value();}
 interface StreamCodec<B,T>{static <B,T> StreamCodec<B,T> ofMember(BiConsumer<T,B> write,Function<B,T> read){return new StreamCodec<>(){};}}
 interface CustomAppEngPayload{record Type<T>(String id){}static <T> Type<T> createType(String id){return new Type<>(id);}}
 interface ClientboundPacket extends CustomAppEngPayload{Type<?> type();void handleOnClient(Player player);}
 static class RegistryAccess{}enum ConnectionType{NEOFORGE}
 static RuntimeException fail(Exception e){Throwable t=e;while(t instanceof InvocationTargetException)t=((InvocationTargetException)t).getCause();return t instanceof RuntimeException r?r:new RuntimeException(t);}
 static class RegistryFriendlyByteBuf {
  static final Class<?> C;static final Method RI,RL,WI,WL;static{try{C=Class.forName("vw");RI=C.getMethod("l");RL=C.getMethod("m");WI=C.getMethod("c",int.class);WL=C.getMethod("a",long.class);}catch(Exception e){throw new ExceptionInInitializerError(e);}}
  ByteBuf raw;Object actual;RegistryFriendlyByteBuf(ByteBuf raw,RegistryAccess r,ConnectionType c){this.raw=raw;try{actual=C.getConstructor(ByteBuf.class).newInstance(raw);}catch(Exception e){throw fail(e);}}
  RegistryFriendlyByteBuf(byte[] bytes){this(Unpooled.wrappedBuffer(bytes),null,ConnectionType.NEOFORGE);}
  int readVarInt(){try{return (int)RI.invoke(actual);}catch(Exception e){throw fail(e);}}long readVarLong(){try{return (long)RL.invoke(actual);}catch(Exception e){throw fail(e);}}
  void writeVarInt(int n){try{WI.invoke(actual,n);}catch(Exception e){throw fail(e);}}void writeVarLong(long n){try{WL.invoke(actual,n);}catch(Exception e){throw fail(e);}}
  boolean readBoolean(){return raw.readBoolean();}void writeBoolean(boolean v){raw.writeBoolean(v);}int writerIndex(){return raw.writerIndex();}int readerIndex(){return raw.readerIndex();}int readableBytes(){return raw.readableBytes();}
  void ensureWritable(int n){raw.ensureWritable(n);}void getBytes(int at,RegistryFriendlyByteBuf dst,int n){raw.getBytes(at,dst.raw,n);}
  byte[] bytes(){byte[] out=new byte[raw.writerIndex()];raw.getBytes(0,out);return out;}
 }
 static int padding,failKey;static List<Long> warnings;
 static class AEKey {
  int id;AEKey(int n){id=n;}public int hashCode(){return id;}public boolean equals(Object o){return o instanceof AEKey k&&k.id==id;}
  static void writeOptionalKey(RegistryFriendlyByteBuf b,AEKey k){b.writeBoolean(k!=null);if(k!=null){b.writeVarInt(k.id);if(k.id==failKey)throw new UnsupportedOperationException();b.writeVarInt(padding);for(int i=0;i<padding;i++)b.raw.writeByte((k.id*31+i)&255);}}
  static AEKey readOptionalKey(RegistryFriendlyByteBuf b){if(!b.readBoolean())return null;int id=b.readVarInt(),n=b.readVarInt();if(id<0||id>=8||n<0)throw new IllegalArgumentException();b.raw.skipBytes(n);return KEYS[id];}
 }
 static final AEKey[] KEYS=new AEKey[8];static{for(int i=0;i<8;i++)KEYS[i]=new AEKey(i);}
 interface AEKeyFilter{boolean matches(AEKey key);}
 static class KeyCounter{Map<AEKey,Long> counts=new HashMap<>();KeyCounter(JsonArray values){for(int i=0;i<8;i++){long n=values.get(i).getAsLong();if(n!=0)counts.put(KEYS[i],n);}}Set<AEKey> keySet(){return counts.keySet();}long get(AEKey k){return counts.getOrDefault(k,0L);}}
 __UPDATES__
 __ENTRY__
 static class AELog{static void warn(String s,Object... args){warnings.add((long)args[0]);}static void info(String s){}}
 static class AbstractContainerMenu{int containerId;AbstractContainerMenu(int id){containerId=id;}}
 static class MEStorageMenu extends AbstractContainerMenu{Repo repo;MEStorageMenu(int id,Repo r){super(id);repo=r;}Repo getClientRepo(){return repo;}}
 static class Player{AbstractContainerMenu containerMenu;}
 static class Repo{
  Map<Long,GridInventoryEntry> entries=new TreeMap<>();List<Object> view=new ArrayList<>(),pinnedRow=new ArrayList<>();Map<Integer,Object> entriesByItemId=new HashMap<>();boolean entriesByItemIdNeedsUpdate;int viewCalls;
  void updateView(){viewCalls++;}
  __REPO__
 }
 __PACKET__
 static String entry(GridInventoryEntry e){return e.getSerial()+":"+(e.getWhat()==null?-1:e.getWhat().id)+":"+e.getStoredAmount()+":"+e.getRequestableAmount()+":"+(e.isCraftable()?1:0);}
 static long[] values(JsonObject o,String name){var a=o.getAsJsonArray(name);long[] out=new long[a.size()];for(int i=0;i<out.length;i++)out[i]=a.get(i).getAsLong();return out;}
 public static void main(String[] args)throws Exception{
  var in=new BufferedReader(new InputStreamReader(System.in));String line;while((line=in.readLine())!=null){
   var row=JsonParser.parseString(line).getAsJsonObject();var updates=new IncrementalUpdateHelper();var repo=new Repo();warnings=new ArrayList<>();var out=new StringBuilder();
   for(var value:row.getAsJsonArray("ops")){var op=value.getAsJsonObject();String mode=op.get("mode").getAsString();
    if(mode.equals("clear")){updates.clear();continue;}if(mode.equals("reset")){updates.reset();continue;}if(mode.equals("remove")){updates.removeSerial(KEYS[op.get("key").getAsInt()]);continue;}
    int id=op.get("id").getAsInt();padding=op.get("padding").getAsInt();failKey=op.get("fail").getAsInt();var b=MEInventoryUpdatePacket.builder(id,updates.isFullUpdate(),new RegistryAccess());boolean fault=false;
    try{if(mode.equals("raw")){int n=op.get("n").getAsInt();for(int i=0;i<n;i++)b.add(new GridInventoryEntry(i,KEYS[i%8],i,-i,(i&1)==0));}
    else{for(var k:op.getAsJsonArray("changes"))updates.addChange(KEYS[k.getAsInt()]);int mask=op.get("filter").getAsInt();b.setFilter(k->(mask&(1<<k.id))!=0);var stored=new KeyCounter(op.getAsJsonArray("stored"));var requestable=new KeyCounter(op.getAsJsonArray("req"));Set<AEKey> craft=new HashSet<>();int bits=op.get("craft").getAsInt();for(int i=0;i<8;i++)if((bits&(1<<i))!=0)craft.add(KEYS[i]);if(mode.equals("full"))b.addFull(updates,stored,craft,requestable);else b.addChanges(updates,stored,craft,requestable);}
    }catch(UnsupportedOperationException ex){fault=true;}
    var packets=b.build();if(b.build()!=packets)throw new AssertionError();out.append(fault?"FAULT|":"OK|");
    for(var p:packets){var data=new RegistryFriendlyByteBuf(Unpooled.buffer(),new RegistryAccess(),ConnectionType.NEOFORGE);p.write(data);out.append(p.encodedEntryCount()).append(':').append(p.fullUpdate()?1:0).append(':').append(HexFormat.of().formatHex(data.bytes())).append(';');
     if(!fault){var actual=MEInventoryUpdatePacket.decode(new RegistryFriendlyByteBuf(data.bytes()));var player=new Player();player.containerMenu=new MEStorageMenu(id,repo);actual.handleOnClient(player);}
    }
    out.append('|');for(int i=0;i<8;i++)out.append(updates.getSerial(KEYS[i])).append(',');out.append(updates.isFullUpdate()?1:0).append(':');var pending=new TreeSet<Integer>();for(AEKey k:updates)pending.add(k.id);for(int k:pending)out.append(k).append(',');out.append('|');for(var e:repo.entries.values())out.append(entry(e)).append(',');out.append('|');for(long s:warnings)out.append(s).append(',');out.append('#');
   }System.out.println(out);
  }
 }
}
'''

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--java-home',type=Path,required=True);args=parser.parse_args();WORK.mkdir(parents=True,exist_ok=True)
    paths=['core/network/clientbound/MEInventoryUpdatePacket.java','menu/me/common/IncrementalUpdateHelper.java','menu/me/common/GridInventoryEntry.java','client/gui/me/common/Repo.java']
    originals={p:(upstream.SOURCE/'src/main/java/appeng'/p).read_text() for p in paths}
    packet=originals[paths[0]];packet=packet[packet.index('public record MEInventoryUpdatePacket'):]
    updates=originals[paths[1]];updates=updates[updates.index('public class IncrementalUpdateHelper'):].replace('public class IncrementalUpdateHelper','public static class IncrementalUpdateHelper',1)
    entry=originals[paths[2]];entry=entry[entry.index('public class GridInventoryEntry'):].replace('public class GridInventoryEntry','public static class GridInventoryEntry',1)
    methods=[method(originals[paths[3]],n) for n in ['public final void handleUpdate(', 'private void handleUpdate(', 'public final void clear(']]
    java=HOST.replace('__UPDATES__',updates).replace('__ENTRY__',entry).replace('__REPO__','\n'.join(methods)).replace('__PACKET__',packet)
    path=WORK/'InventoryUpdateOracle.java';path.write_text(java);cp=minecraft_reference.prepare()
    subprocess.run([str(args.java_home/'bin/javac'),'-proc:none','-cp',cp,'-d',str(WORK),str(path)],cwd=WORK,check=True)
    base={'mode':'changes','id':7,'padding':0,'fail':-1,'changes':list(range(8)),'stored':[1]*8,'req':[0]*8,'craft':0,'filter':255}
    rows=[]
    for mask,fail,padding in itertools.product([0,1,85,255],[-1,0,3,7],[0,1,127]):
        ops=[{**base,'filter':mask,'fail':fail,'padding':padding},{**base,'stored':[0,2,0,-1,4,0,0,8]}, {'mode':'clear'}, {**base,'changes':[],'stored':[0]*8}, {**base,'mode':'full'}, {'mode':'reset'}, {**base,'stored':[0]*8,'req':[-1,0,2,0,3,0,0,0],'craft':13}]
        rows.append({'ops':ops})
    rng=random.Random(119817)
    for _ in range(1200):
        ops=[]
        for _ in range(rng.randrange(1,15)):
            mode=rng.choice(['full','changes','changes','clear','reset','remove'])
            if mode in ['clear','reset','remove']:ops.append({'mode':mode,'key':rng.randrange(8)});continue
            ops.append({**base,'mode':mode,'id':rng.choice([-2147483648,-1,0,7,2147483647]),'changes':[rng.randrange(8) for _ in range(rng.randrange(12))],
                        'stored':[rng.choice([-9223372036854775808,-1,0,1,127,128,1000,9223372036854775807]) for _ in range(8)],'req':[rng.choice([-1,0,1,9223372036854775807]) for _ in range(8)],'craft':rng.randrange(256),'filter':rng.randrange(256),'padding':rng.choice([0,3,127,128]),'fail':rng.choice([-1,-1,-1,0,7])})
        rows.append({'ops':ops})
    # Entry count flush boundary and byte-limit post-write boundary (no preflush).
    for n,padding in [(32766,0),(32767,0),(32768,0),(65535,0),(1,524278),(1,524279),(1,524280),(1,524288),(3,262144),(3,524288)]:
        rows.append({'ops':[{**base,'mode':'raw','n':n,'padding':padding}]})
    data=('\n'.join(json.dumps(r,separators=(',',':')) for r in rows)+'\n').encode();(WORK/'input.txt').write_bytes(data)
    expected=subprocess.run([str(args.java_home/'bin/java'),'-cp',str(WORK)+':'+cp,'oracle.InventoryUpdateOracle'],cwd=WORK,input=data,capture_output=True)
    (WORK/'java.stderr').write_bytes(expected.stderr);expected.check_returncode()
    actual=subprocess.run(['cargo','run','--quiet','--locked','--example','inventory_update_oracle'],cwd=ROOT,input=data,capture_output=True)
    (WORK/'rust.stderr').write_bytes(actual.stderr);actual.check_returncode()
    (WORK/'java.txt').write_bytes(expected.stdout);(WORK/'rust.txt').write_bytes(actual.stdout)
    if expected.stdout!=actual.stdout:
        for i,(a,b) in enumerate(itertools.zip_longest(expected.stdout.splitlines(),actual.stdout.splitlines())):
            if a!=b:raise AssertionError(f'case {i}: {rows[i]}\nJava {(a or b"")[:1500]!r}\nRust {(b or b"")[:1500]!r}')
    sha=lambda b:hashlib.sha256(b).hexdigest()
    report={'passed':True,'cases':len(rows),'source_sha256':{p:sha(s.encode()) for p,s in originals.items()},'binding_sha256':sha(java.encode()),'rust_sha256':{p:sha((ROOT/p).read_bytes()) for p in ['mixins/ae2/inventory_update.rs','mixins/ae2/incremental_update.rs','mixins/ae2/me_interaction.rs','examples/inventory_update_oracle.rs']},'input_sha256':sha(data),'output_sha256':sha(actual.stdout),
            'scope':'Original whole packet builder/encode/decode and client dispatch, original GridInventoryEntry/IncrementalUpdateHelper and extracted Repo update/clear methods; exact packet bytes, serial/pending state and resulting client entries',
            'limits':['Controlled optional-key registry/component codec and player/menu/Repo view bindings; real MC FriendlyByteBuf/Netty varint encoding','Eight controlled keys use collision-free small integer hashes matching sorted iteration; arbitrary Java HashSet ordering excluded','No native session/transport/UI/search/sort/view-cell/permission or actual registry component codec proof','Partial key-write failures retain source buffered bytes and pending/serial mutations; caller must discard failed transmissions']}
    (ROOT/'docs/inventory-update-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
