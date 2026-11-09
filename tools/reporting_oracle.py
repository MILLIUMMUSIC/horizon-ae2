#!/usr/bin/env python3
"""Original Reporting/AEBasePart callbacks with verified MC floor and NBT codecs.

Player/world/node/host/model interfaces are controlled bindings, not a running
Minecraft world or a full managed-node/input/packet lifecycle comparison.
"""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import random
import struct
import subprocess

import minecraft_reference
import upstream
from inscriber_oracle import method

ROOT=upstream.ROOT
WORK=ROOT/'target/reporting-oracle'
BASE=upstream.SOURCE/'src/main/java/appeng'
JAVA=r'''
package oracle;
import java.io.*;import java.util.*;import java.lang.reflect.*;
public class ReportingOracle {
 enum Direction {
  DOWN(0,-1,0),UP(0,1,0),NORTH(0,0,-1),SOUTH(0,0,1),WEST(-1,0,0),EAST(1,0,0);
  int x,y,z;Direction(int x,int y,int z){this.x=x;this.y=y;this.z=z;}
 }
 record BlockPos(int x,int y,int z){BlockPos relative(Direction d){return new BlockPos(x+d.x,y+d.y,z+d.z);}}
 static class BlockGetter {}
 static class BlockState {Level level;BlockState(Level l){level=l;}int getLightBlock(Level l,BlockPos p){l.queries++;return l.opacity;}}
 static class Level extends BlockGetter {int opacity,queries;BlockState getBlockState(BlockPos p){return new BlockState(this);}}
 static class BlockEntity {Level level=new Level();BlockPos pos=new BlockPos(Integer.MAX_VALUE,2,3);Level getLevel(){return level;}BlockPos getBlockPos(){return pos;}}
 static class Host {BlockEntity be=new BlockEntity();int updates,saves;void markForUpdate(){updates++;}void markForSave(){saves++;}BlockEntity getBlockEntity(){return be;}}
 static class Inventory {Object getSelected(){return null;}}
 static class Player {float yaw;float getYRot(){return yaw;}Inventory getInventory(){return new Inventory();}}
 static class Vec3 {}
 static class InteractionUtil {static boolean allowed;static boolean canWrenchRotate(Object stack){return allowed;}}
 static class HolderLookup {static class Provider {}}
 interface IPartModel {}
 record Model(int id) implements IPartModel {}
 static class RegistryFriendlyByteBuf {byte[] bytes=new byte[2];int cursor;void writeByte(int b){bytes[cursor++]=(byte)b;}byte readByte(){return bytes[cursor++];}}
 static class Mth {
  static int floor(double value){try{return (int)Class.forName("ayo").getMethod("a",double.class).invoke(null,value);}catch(Exception e){throw new RuntimeException(e);}}
 }
 static class CompoundTag {
  Object actual;
  CompoundTag(){try{actual=Class.forName("ub").getConstructor().newInstance();}catch(Exception e){throw new RuntimeException(e);}}
  void set(int type,String value){try{
   if(type==0)return;
   String cls=switch(type){case 1->"tz";case 2->"ut";case 3->"ug";case 4->"uj";case 5->"ue";case 6->"uc";default->"uw";};
   Class<?> arg=switch(type){case 1->byte.class;case 2->short.class;case 3->int.class;case 4->long.class;case 5->float.class;case 6->double.class;default->String.class;};
   Object number=switch(type){case 1->Byte.valueOf(value);case 2->Short.valueOf(value);case 3->Integer.valueOf(value);case 4->Long.valueOf(value);case 5->Float.intBitsToFloat(Integer.parseUnsignedInt(value,16));case 6->Double.longBitsToDouble(Long.parseUnsignedLong(value,16));default->"invalid";};
   Object tag=Class.forName(cls).getMethod("a",arg).invoke(null,number);
   Class.forName("ub").getMethod("a",String.class,Class.forName("uy")).invoke(actual,"spin",tag);
  }catch(Exception e){throw new RuntimeException(e);}}
  byte getByte(String name){try{return (byte)Class.forName("ub").getMethod("f",String.class).invoke(actual,name);}catch(Exception e){throw new RuntimeException(e);}}
  void putByte(String name,byte value){set(1,Byte.toString(value));}
 }
 static class Base {
  boolean clientSidePowered,clientSideMissingChannel,sendPower=true,sendMissing=true,client,light;
  Host host=new Host();Direction side=Direction.NORTH;
  boolean isPowered(){return clientSidePowered;}boolean isMissingChannel(){return clientSideMissingChannel;}
  boolean isActive(){return isPowered()&&!isMissingChannel();}boolean isLightSource(){return light;}
  boolean shouldSendPowerStateToClient(){return sendPower;}boolean shouldSendMissingChannelStateToClient(){return sendMissing;}
  Host getHost(){return host;}Direction getSide(){return side;}boolean isClientSide(){return client;}
  void onPlacement(Player p){}boolean onUseWithoutItem(Player p,Vec3 pos){return false;}
  void readFromNBT(CompoundTag tag,HolderLookup.Provider registries){}void writeToNBT(CompoundTag tag,HolderLookup.Provider registries){}
  // BASE_METHODS
 }
 static class Part extends Base {
  byte spin=0;int opacity=-1;
  // REPORTING_METHODS
 }
 static int n(StringTokenizer t){return Integer.parseInt(t.nextToken());}
 public static void main(String[] args)throws Exception {
  var input=new BufferedReader(new InputStreamReader(System.in));String line;var out=new StringBuilder();
  while((line=input.readLine())!=null){var t=new StringTokenizer(line);String op=t.nextToken();var p=new Part();
   switch(op){
    case "P"->{p.side=Direction.values()[n(t)];p.spin=(byte)n(t);var player=new Player();player.yaw=Float.intBitsToFloat(Integer.parseUnsignedInt(t.nextToken(),16));p.onPlacement(player);out.append(p.getSpin());}
    case "W"->{p.spin=(byte)n(t);p.client=n(t)!=0;InteractionUtil.allowed=n(t)!=0;boolean result=p.onUseWithoutItem(new Player(),new Vec3());out.append(p.spin).append(' ').append(result?1:0).append(' ').append(p.host.updates).append(' ').append(p.host.saves);}
    case "F"->{int old=n(t);p.clientSidePowered=(old&1)!=0;p.clientSideMissingChannel=(old&2)!=0;p.spin=(byte)n(t);var packet=new RegistryFriendlyByteBuf();packet.bytes=new byte[]{(byte)n(t),(byte)n(t)};p.sendPower=n(t)!=0;p.sendMissing=n(t)!=0;boolean changed=p.readFromStream(packet);var model=(Model)p.selectModel(new Model(0),new Model(1),new Model(2));var wire=new RegistryFriendlyByteBuf();p.writeToStream(wire);out.append(changed?1:0).append(' ').append((p.isPowered()?1:0)|(p.isMissingChannel()?2:0)).append(' ').append(p.spin).append(' ').append(model.id).append(' ').append(wire.bytes[0]&255).append(' ').append(wire.bytes[1]&255);}
    case "N"->{var tag=new CompoundTag();tag.set(n(t),t.nextToken());p.readFromNBT(tag,null);out.append(p.spin);p.writeToNBT(tag,null);if(tag.getByte("spin")!=p.spin)throw new AssertionError("saved spin");}
    case "L"->{p.side=Direction.values()[n(t)];p.host.be.level.opacity=n(t);int second=n(t);boolean powered=n(t)!=0;p.light=n(t)!=0;var neighbor=new BlockPos(n(t),n(t),n(t));int off=p.getLightLevel();p.clientSidePowered=powered;p.host.be.level.opacity=second;int light=p.getLightLevel();p.onNeighborChanged(p.host.be.level,p.host.be.pos,neighbor);int after=p.getLightLevel();out.append(off).append(' ').append(light).append(' ').append(after).append(' ').append(p.host.be.level.queries).append(' ').append(p.host.updates);}
    default->throw new AssertionError(op);
   }
   out.append('\n');
  }
  System.out.print(out);
 }
}
'''


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home',type=Path,default=Path('/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home'))
    args=parser.parse_args();upstream.verify(upstream.SOURCE);WORK.mkdir(parents=True,exist_ok=True)
    cp=minecraft_reference.prepare()
    source=(BASE/'parts/reporting/AbstractReportingPart.java').read_text()
    signatures=['public void onNeighborChanged(', 'public void readFromNBT(', 'public void writeToNBT(', 'public void writeToStream(', 'public boolean readFromStream(', 'public final int getLightLevel(', 'public boolean onUseWithoutItem(', 'public final void onPlacement(', 'private int blockLight(', 'protected IPartModel selectModel(', 'public final byte getSpin(']
    methods=[method(source,s) for s in signatures]
    base=(BASE/'parts/AEBasePart.java').read_text()
    base_methods=[method(base,s) for s in ['public void writeToStream(', 'public boolean readFromStream(']]
    java=JAVA.replace('// BASE_METHODS','\n'.join(base_methods)).replace('// REPORTING_METHODS','\n'.join(methods))
    src=WORK/'ReportingOracle.java';src.write_text(java)
    compiled=subprocess.run([str(args.java_home/'bin/javac'),'-proc:none','-cp',cp,'-d',str(WORK/'classes'),str(src)],capture_output=True)
    (WORK/'javac.stderr').write_bytes(compiled.stderr)
    if compiled.returncode:raise RuntimeError(compiled.stderr.decode()[-6000:])
    rng=random.Random(0xae219218)
    fbits=lambda n:struct.pack('>f',n).hex()
    dbits=lambda n:struct.pack('>d',n).hex()
    yaws={fbits(v) for v in range(-720,721)}
    for v in range(-16,17):
        for d in range(-4,5):yaws.add(f'{(int(fbits(v*90-45),16)+d)&0xffffffff:08x}')
    yaws.update(f'{rng.getrandbits(32):08x}' for _ in range(4000))
    rows=[f'P {side} {spin} {yaw}' for side in range(6) for spin in [-128,-3,0,3,127] for yaw in sorted(yaws)]
    placement=len(rows)
    rows += [f'W {spin} {client} {allowed}' for spin,client,allowed in itertools.product(range(-128,128),range(2),range(2))]
    wrench=len(rows)-placement
    rows += [f'F {old} {spin} {new} {next_spin} {sp} {sm}' for old,spin,new,next_spin,sp,sm in itertools.product(range(4),[-128,0,3,127],range(256),[-128,0,3,127],range(2),range(2))]
    flags=len(rows)-placement-wrench
    for kind in range(8):
        values={0:['0'],1:list(map(str,range(-128,128))),2:list(map(str,[-32768,-257,-256,-255,-1,0,1,255,256,257,32767])),3:list(map(str,[-2147483648,-257,-256,-255,-1,0,1,255,256,257,2147483647])),4:list(map(str,[-2**63,-2**31,-257,-1,0,1,257,2**31,2**63-1])),5:[fbits(v) for v in [-float('inf'),float('inf'),float('nan'),-0.1,-1.1,-0.,0.,0.1,255.9,256.,-2.**31,2.**31]],6:[dbits(v) for v in [-float('inf'),float('inf'),float('nan'),-0.1,-1.1,-0.,0.,0.1,255.9,256.,-2.**31,2.**31]],7:['0']}[kind]
        if kind in [5,6]:values += [f'{rng.getrandbits(32 if kind==5 else 64):x}' for _ in range(4000)]
        rows += [f'N {kind} {value}' for value in values]
    nbt=len(rows)-placement-wrench-flags
    offsets=[(0,-1,0),(0,1,0),(0,0,-1),(0,0,1),(-1,0,0),(1,0,0)]
    signed=lambda n:(n+2**31)%2**32-2**31
    for side,a,b,power,light,match in itertools.product(range(6),[-2147483648,-1,0,1,4,9,15,16,2147483647],[-2147483648,-1,0,4,15,2147483647],range(2),range(2),range(2)):
        neighbor=[signed(2147483647+offsets[side][0]),2+offsets[side][1],3+offsets[side][2]] if match else [0,0,0]
        rows.append(f'L {side} {a} {b} {power} {light} '+' '.join(map(str,neighbor)))
    data=('\n'.join(rows)+'\n').encode();(WORK/'input.txt').write_bytes(data)
    expected=subprocess.run([str(args.java_home/'bin/java'),'-cp',str(WORK/'classes')+os.pathsep+cp,'oracle.ReportingOracle'],cwd=WORK,input=data,capture_output=True)
    (WORK/'java.stderr').write_bytes(expected.stderr)
    if expected.returncode:raise RuntimeError(expected.stderr.decode()[-6000:])
    actual=subprocess.run(['cargo','run','--quiet','--locked','--example','reporting_oracle'],cwd=ROOT,input=data,capture_output=True)
    (WORK/'rust.stderr').write_bytes(actual.stderr);actual.check_returncode()
    (WORK/'java.txt').write_bytes(expected.stdout);(WORK/'rust.txt').write_bytes(actual.stdout)
    if expected.stdout!=actual.stdout:
        for i,(a,b) in enumerate(itertools.zip_longest(expected.stdout.splitlines(),actual.stdout.splitlines())):
            if a!=b:raise AssertionError(f'case {i}: {rows[i]}\nJava {a}\nRust {b}')
    sha=lambda b:hashlib.sha256(b).hexdigest()
    report={'passed':True,'cases':len(rows),'placement_cases':placement,'wrench_cases':wrench,'stream_flag_model_cases':flags,'nbt_cases':nbt,'light_cache_cases':len(rows)-placement-wrench-flags-nbt,
      'source_sha256':{name:sha((BASE/name).read_bytes()) for name in ['parts/reporting/AbstractReportingPart.java','parts/AEBasePart.java']},
      'method_sha256':{sig:sha(body.encode()) for sig,body in zip(signatures,methods)},'base_method_sha256':[sha(m.encode()) for m in base_methods],
      'binding_sha256':sha(java.encode()),'rust_sha256':{name:sha((ROOT/name).read_bytes()) for name in ['mixins/ae2/reporting.rs','examples/reporting_oracle.rs']},
      'minecraft_client_sha256':sha((minecraft_reference.WORK/'client.jar').read_bytes()),'minecraft_mappings_sha256':sha((minecraft_reference.WORK/'client_mappings.txt').read_bytes()),'input_sha256':sha(data),'output_sha256':sha(actual.stdout),
      'scope':'Verbatim Reporting placement/wrench/NBT/stream/light/cache/model callbacks and AEBasePart flags; real pinned Minecraft Mth.floor, CompoundTag and numeric tags',
      'limits':['Player/world/node/host/model/base-callback bindings are controlled; no native placement, complete managed-node/input/packet ordering or world-light proof','Native saved-part bridge and node profile consume these rules, but side-part world activation/rendering/menu remains separate']}
    (ROOT/'docs/reporting-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))


if __name__=='__main__':main()
