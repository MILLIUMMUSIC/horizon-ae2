#!/usr/bin/env python3
"""Compare original MEInteractionPacket with real MC FriendlyByteBuf/Netty.

Preserves source record write/decode/server dispatch and InventoryAction. Only
payload type/codec registration and player/menu interfaces are controlled.
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
ROOT=upstream.ROOT
WORK=ROOT/'target/me-interaction-oracle'
HOST=r'''
package oracle;
import java.io.*;
import java.lang.reflect.*;
import java.util.*;
import java.util.function.*;
import io.netty.buffer.*;
public class MEInteractionOracle {
 interface StreamCodec<B,T> {static <B,T> StreamCodec<B,T> ofMember(BiConsumer<T,B> write,Function<B,T> read){return new StreamCodec<>(){};}}
 interface CustomAppEngPayload {record Type<T>(String id){}static <T> Type<T> createType(String id){return new Type<>(id);}}
 interface ServerboundPacket extends CustomAppEngPayload {Type<?> type();void handleOnServer(ServerPlayer player);}
 interface IMEInteractionHandler {void handleInteraction(long serial,InventoryAction action);}
 static class AbstractContainerMenu {int containerId;AbstractContainerMenu(int n){containerId=n;}}
 static class Menu extends AbstractContainerMenu implements IMEInteractionHandler {
  String event="ignored";Menu(int n){super(n);}public void handleInteraction(long n,InventoryAction action){event=n+":"+action.ordinal();}
 }
 static class ServerPlayer {AbstractContainerMenu containerMenu;}
 static RuntimeException fail(Exception e){Throwable t=e;while(t instanceof InvocationTargetException)t=((InvocationTargetException)t).getCause();return t instanceof RuntimeException r?r:new RuntimeException(t);}
 static class RegistryFriendlyByteBuf {
  static final Class<?> C;static {try{C=Class.forName("vw");}catch(Exception e){throw new ExceptionInInitializerError(e);}}
  ByteBuf raw;Object actual;
  RegistryFriendlyByteBuf(byte[] bytes){try{raw=Unpooled.wrappedBuffer(bytes);actual=C.getConstructor(ByteBuf.class).newInstance(raw);}catch(Exception e){throw fail(e);}}
  RegistryFriendlyByteBuf(){try{raw=Unpooled.buffer();actual=C.getConstructor(ByteBuf.class).newInstance(raw);}catch(Exception e){throw fail(e);}}
  int readInt(){return raw.readInt();}
  long readVarLong(){try{return (long)C.getMethod("m").invoke(actual);}catch(Exception e){throw fail(e);}}
  <E extends Enum<E>> E readEnum(Class<E> cls){try{return cls.cast(C.getMethod("b",Class.class).invoke(actual,cls));}catch(Exception e){throw fail(e);}}
  void writeInt(int n){raw.writeInt(n);}
  void writeVarLong(long n){try{C.getMethod("a",long.class).invoke(actual,n);}catch(Exception e){throw fail(e);}}
  void writeEnum(Enum<?> value){try{C.getMethod("a",Enum.class).invoke(actual,value);}catch(Exception e){throw fail(e);}}
 }
 __ACTION__
 __PACKET__
 public static void main(String[] args) throws Exception {
  var input=new BufferedReader(new InputStreamReader(System.in));String line;
  while((line=input.readLine())!=null){String[] w=line.split(" ");
   switch(w[0]){
    case "P"->{var p=new MEInteractionPacket(Integer.parseInt(w[1]),Long.parseLong(w[2]),InventoryAction.values()[Integer.parseInt(w[3])]);var b=new RegistryFriendlyByteBuf();p.write(b);byte[] bytes=new byte[b.raw.writerIndex()];b.raw.getBytes(0,bytes);System.out.println(HexFormat.of().formatHex(bytes));b.raw.release();}
    case "D"->{var b=new RegistryFriendlyByteBuf(w.length>1?HexFormat.of().parseHex(w[1]):new byte[0]);try{var p=MEInteractionPacket.decode(b);System.out.println("OK:"+p.containerId()+":"+p.serial()+":"+p.action().ordinal()+":"+b.raw.readerIndex());}catch(Exception e){String kind=e instanceof ArrayIndexOutOfBoundsException?"InvalidAction":e instanceof IndexOutOfBoundsException?"Truncated":e.getMessage().equals("VarLong too big")?"VarLongTooBig":e.getMessage().equals("VarInt too big")?"VarIntTooBig":"UNEXPECTED_"+e;System.out.println("ERR:"+kind+":"+b.raw.readerIndex());}finally{b.raw.release();}}
    case "H"->{var p=new MEInteractionPacket(Integer.parseInt(w[1]),Long.parseLong(w[2]),InventoryAction.values()[Integer.parseInt(w[3])]);var menu=new Menu(Integer.parseInt(w[4]));var player=new ServerPlayer();player.containerMenu=w[5].equals("1")?menu:new AbstractContainerMenu(menu.containerId);p.handleOnServer(player);System.out.println(menu.event);}
    default->throw new AssertionError();
   }
  }
 }
}
'''

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--java-home',type=Path,required=True);args=parser.parse_args();WORK.mkdir(parents=True,exist_ok=True)
    paths=['core/network/serverbound/MEInteractionPacket.java','helpers/InventoryAction.java']
    originals={name:(upstream.SOURCE/'src/main/java/appeng'/name).read_text() for name in paths}
    packet=originals[paths[0]];packet=packet[packet.index('public record MEInteractionPacket'):]
    action=originals[paths[1]];action=action[action.index('public enum InventoryAction'):]
    java=HOST.replace('__ACTION__',action).replace('__PACKET__',packet);path=WORK/'MEInteractionOracle.java';path.write_text(java);cp=minecraft_reference.prepare()
    subprocess.run([str(args.java_home/'bin/javac'),'-proc:none','-cp',cp,'-d',str(WORK),str(path)],cwd=WORK,check=True)
    rows=[]
    for cid,serial,action in itertools.product([-2**31,-1,0,1,2**31-1],[-2**63,-1,0,1,127,128,2**31,2**63-1],range(21)):
        rows.append(f'P {cid} {serial} {action}')
        for compatible in [0,1]:
            for current in [cid,cid^1]:rows.append(f'H {cid} {serial} {action} {current} {compatible}')
    # Both noncanonical and overflow payloads: real MC consumes an extra byte
    # before a too-long error, but a missing extra byte stays buffer underflow.
    payloads=[b'',b'\0',b'\0'*3,b'\0'*6,b'\0'*4+b'\x80\0\x80\0\xff']
    for n in range(1,13):payloads.extend([b'\0'*4+b'\x80'*n,b'\0'*4+b'\x80'*n+b'\0',b'\0'*5+b'\x80'*n,b'\0'*5+b'\x80'*n+b'\0'])
    rng=random.Random(912819)
    for _ in range(40000):
        payloads.append(rng.randbytes(rng.randrange(33)))
    rows.extend('D '+p.hex() for p in payloads)
    data=('\n'.join(rows)+'\n').encode();(WORK/'input.txt').write_bytes(data)
    expected=subprocess.run([str(args.java_home/'bin/java'),'-cp',str(WORK)+':'+cp,'oracle.MEInteractionOracle'],cwd=WORK,input=data,capture_output=True)
    (WORK/'java.stderr').write_bytes(expected.stderr);expected.check_returncode()
    actual=subprocess.run(['cargo','run','--quiet','--locked','--example','me_interaction_oracle'],cwd=ROOT,input=data,capture_output=True)
    (WORK/'rust.stderr').write_bytes(actual.stderr);actual.check_returncode()
    (WORK/'java.txt').write_bytes(expected.stdout);(WORK/'rust.txt').write_bytes(actual.stdout)
    if actual.stdout!=expected.stdout:
        for i,(a,b) in enumerate(itertools.zip_longest(expected.stdout.splitlines(),actual.stdout.splitlines())):
            if a!=b:raise AssertionError(f'case {i}: {rows[i]}\nJava {a!r}\nRust {b!r}')
    sha=lambda b:hashlib.sha256(b).hexdigest()
    report={'passed':True,'cases':len(rows),'source_sha256':{n:sha(s.encode()) for n,s in originals.items()},'binding_sha256':sha(java.encode()),'rust_sha256':{n:sha((ROOT/n).read_bytes()) for n in ['mixins/ae2/me_interaction.rs','examples/me_interaction_oracle.rs']},'minecraft_client_sha256':sha((minecraft_reference.WORK/'client.jar').read_bytes()),'input_sha256':sha(data),'output_sha256':sha(actual.stdout),
        'scope':'Original packet body encode/decode and server current-menu/id dispatch, original InventoryAction enum, actual Minecraft FriendlyByteBuf/VarLong/VarInt and Netty indices; malformed/noncanonical input and signed bounds',
        'limits':['Controlled codec/type registration and player/menu interfaces; no native network authentication or real menu lifetime/storage transaction','This is packet payload behavior, not complete native framing or the MEStorageMenu link/serial/action gate']}
    (ROOT/'docs/me-interaction-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
