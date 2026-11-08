#!/usr/bin/env python3
"""Compare placement and wrench rules with pinned Java orientation references."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import shutil
import struct
import subprocess
import minecraft_reference as reference
import upstream
ROOT = reference.ROOT
WORK = ROOT / 'target/orientation-oracle'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home', type=Path, default=Path(os.environ.get('JAVA_HOME', '/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home')))
    args = parser.parse_args()
    upstream.verify(upstream.SOURCE)
    cp = reference.prepare()
    source = WORK / 'src'; classes = WORK / 'classes'
    source.mkdir(parents=True, exist_ok=True); classes.mkdir(exist_ok=True)
    files = {
      'net/minecraft/core/Direction.java': '''package net.minecraft.core;
public enum Direction { DOWN,UP,NORTH,SOUTH,WEST,EAST;
 static final Class<?> MC; static final Object[] VALUES;
 static {try {MC=Class.forName("ji");VALUES=MC.getEnumConstants();}catch(Exception e){throw new RuntimeException(e);}}
 public static Direction fromYaw(float yaw) {try{return values()[((Enum<?>)MC.getMethod("a",double.class).invoke(null,(double)yaw)).ordinal()];}catch(Exception e){throw new RuntimeException(e);}}
 public Direction getOpposite(){return values()[ordinal() ^ 1];}
 public Direction around(Direction face){try{
 Object click=VALUES[face.ordinal()];Object axis=MC.getMethod("o").invoke(click);
 int sign=((Enum<?>)MC.getMethod("f").invoke(click)).ordinal();
 return values()[((Enum<?>)MC.getMethod(sign==0?"a":"b",Class.forName("ji$a")).invoke(VALUES[ordinal()],axis)).ordinal()];
 }catch(Exception e){throw new RuntimeException(e);}}
}''',
      'net/minecraft/world/level/block/state/properties/Property.java': 'package net.minecraft.world.level.block.state.properties; public class Property<T> {public final String name;public Property(String n){name=n;}}',
      'net/minecraft/world/level/block/state/properties/BlockStateProperties.java': 'package net.minecraft.world.level.block.state.properties; public class BlockStateProperties {public static final Property<net.minecraft.core.Direction> FACING=new Property<>("facing");}',
      'net/minecraft/world/level/block/state/BlockState.java': '''package net.minecraft.world.level.block.state;
import net.minecraft.core.Direction;import net.minecraft.world.level.block.state.properties.Property;
public record BlockState(Direction facing,int spin) {
 @SuppressWarnings("unchecked") public <T> T getValue(Property<T> p){return (T)(p.name.equals("facing")?facing:Integer.valueOf(spin));}
 public <T> BlockState setValue(Property<T> p,T v){return p.name.equals("facing")?new BlockState((Direction)v,spin):new BlockState(facing,(Integer)v);}
}''',
      'net/minecraft/world/item/context/BlockPlaceContext.java': '''package net.minecraft.world.item.context;
import net.minecraft.core.Direction;
public record BlockPlaceContext(float yaw,float pitch) {
 public Direction getHorizontalDirection(){return Direction.fromYaw(yaw);}
 public BlockPlaceContext getPlayer(){return this;} public float getXRot(){return pitch;}
}''',
      'appeng/api/orientation/IOrientationStrategy.java': '''package appeng.api.orientation;
import java.util.Collection;import appeng.block.orientation.SpinMapping;import net.minecraft.core.Direction;
import net.minecraft.world.level.block.state.BlockState;import net.minecraft.world.level.block.state.properties.Property;
import net.minecraft.world.item.context.BlockPlaceContext;
public interface IOrientationStrategy {
 Property<Integer> SPIN=new Property<>("spin");
 Direction getFacing(BlockState s);int getSpin(BlockState s);BlockState setFacing(BlockState s,Direction f);BlockState setSpin(BlockState s,int spin);
 BlockState getStateForPlacement(BlockState s,BlockPlaceContext c);Collection<Property<?>> getProperties();
 default BlockState setOrientation(BlockState s,Direction f,Direction up){return setSpin(setFacing(s,f),SpinMapping.getSpinFromUp(f,up));}
}''',
      'ae2reference/OrientationReference.java': '''package ae2reference;
import java.io.*;import java.util.*;import net.minecraft.core.Direction;import net.minecraft.world.item.context.BlockPlaceContext;
import net.minecraft.world.level.block.state.BlockState;import appeng.api.orientation.*;import appeng.block.orientation.SpinMapping;
public class OrientationReference {
 public static void main(String[] args)throws Exception {
 var ctor=FacingWithSpinStrategy.class.getDeclaredConstructor();ctor.setAccessible(true);var strategy=ctor.newInstance();
 var reader=new BufferedReader(new InputStreamReader(System.in));String line;
 while((line=reader.readLine())!=null){var a=line.split(" ");Direction facing;int spin;
 if(a[0].equals("Y")){float yaw=Float.intBitsToFloat(Integer.parseUnsignedInt(a[1],16)),pitch=Float.intBitsToFloat(Integer.parseUnsignedInt(a[2],16));
 var result=strategy.getStateForPlacement(new BlockState(Direction.NORTH,0),new BlockPlaceContext(yaw,pitch));facing=result.facing();spin=result.spin();
 }else{var old=Direction.values()[Integer.parseInt(a[1])];var up=SpinMapping.getUpFromSpin(old,Integer.parseInt(a[2]));var click=Direction.values()[Integer.parseInt(a[3])];
 facing=old.around(click);spin=SpinMapping.getSpinFromUp(facing,up.around(click));}
 System.out.println(facing.name().toLowerCase(Locale.ROOT)+" "+spin);
 }
 }
}'''}
    hashes = {}
    for relative in ['api/orientation/FacingWithSpinStrategy.java','block/orientation/SpinMapping.java']:
        original = upstream.SOURCE / 'src/main/java/appeng' / relative
        target = source / 'appeng' / relative; target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(original, target); hashes[relative] = hashlib.sha256(original.read_bytes()).hexdigest()
    for name, text in files.items():
        path = source / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_text(text+'\n')
    def bits(value):return struct.unpack('>I', struct.pack('>f', value))[0]
    yaws = {bits(n) for n in range(-720, 721)}
    for n in range(-16, 17):
        for delta in range(-4, 5):yaws.add((bits(n*45.0)+delta)&0xffffffff)
    rng = random.Random(0xae2)
    yaws.update(rng.getrandbits(32) for _ in range(10000))
    pitches = [bits(v) for v in [0,-65,65,-90,90]]+[bits(v)+d for v in [-65,65] for d in [-1,1]]
    rows = [f'Y {yaw:x} {pitch:x}' for yaw in sorted(yaws) for pitch in pitches]
    placement = len(rows)
    rows.extend(f'R {face} {spin} {click}' for face in range(6) for spin in range(4) for click in range(6))
    data = ('\n'.join(rows)+'\n').encode()
    java = args.java_home / 'bin/java'; javac = args.java_home / 'bin/javac'
    subprocess.run([str(javac),'-proc:none','-cp',cp,'-d',str(classes),*map(str,source.rglob('*.java'))],check=True,capture_output=True)
    original = subprocess.run([str(java),'-cp',cp+os.pathsep+str(classes),'ae2reference.OrientationReference'],cwd=WORK,input=data,capture_output=True,check=True)
    port = subprocess.run(['cargo','run','--quiet','--locked','--example','orientation_oracle'],cwd=ROOT,input=data,capture_output=True,check=True)
    want=original.stdout.splitlines();got=port.stdout.splitlines();assert len(want)==len(got)==len(rows)
    failures=[{'case':r,'java':a.decode(),'rust':b.decode()} for r,a,b in zip(rows,want,got) if a!=b]
    (WORK/'failures.json').write_text(json.dumps(failures[:100],indent=2)+'\n')
    if failures:raise AssertionError(f'{len(failures)} orientation mismatches; see {WORK}/failures.json')
    report={'passed':True,'cases':len(rows),'placement_cases':placement,'rotation_cases':len(rows)-placement,'source_sha256':hashes,'port_sha256':hashlib.sha256((ROOT/'mixins/ae2/orientation.rs').read_bytes()).hexdigest(),'input_sha256':hashlib.sha256(data).hexdigest(),
      'scope':'Verbatim FacingWithSpinStrategy and SpinMapping with original Minecraft Direction.fromYRot and clockwise/counterclockwise direction methods',
      'limits':['Placement context/property/state/interface are minimal hosts, not a complete Java world','Rotation composes original direction turns with SpinMapping as in BlockOrientation; the full BlockOrientation renderer/quaternion class is not executed','Native hand/spectator gating, placement hooks, structure tables and actual client/server behavior require separate checks']}
    (ROOT/'docs/orientation-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))


if __name__ == '__main__':main()
