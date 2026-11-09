#!/usr/bin/env python3
"""Execute original BusCollisionHelper and source host selection with real MC AABB.

Direction vectors, part storage and facade absence are controlled bindings.
Minecraft's verified 1.21.1 AABB constructor, inflation and containment execute
through reflection; no game/world or GPU rendering is launched.
"""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import random
import shutil
import struct
import subprocess

import cable_oracle
import minecraft_reference
import upstream
from inscriber_oracle import method

ROOT = upstream.ROOT
BASE = upstream.SOURCE / 'src/main/java/appeng'
WORK = ROOT / 'target/part-geometry-oracle'

JAVA = r'''
package oracle;
import java.io.*;
import java.util.*;
import appeng.parts.BusCollisionHelper;
import appeng.api.parts.IPartCollisionHelper;
import net.minecraft.core.Direction;
import net.minecraft.world.phys.*;
public class PartGeometryOracle {
 static abstract class Part {public abstract void getBoxes(IPartCollisionHelper bch);}
 static class Reporting extends Part {
  // REPORTING_BOXES
 }
 static class Center extends Part {
  CableOracle.CablePart cable;
  Center(CableOracle.CablePart cable){this.cable=cable;}
  public void getBoxes(IPartCollisionHelper bch){
   var boxes=new CableOracle.Boxes();boxes.bb=bch.isBBCollision();cable.getBoxes(boxes,d->true);
   for(var b:boxes.calls)bch.addBox(b[0],b[1],b[2],b[3],b[4],b[5]);
  }
 }
 static class Facade {void getBoxes(IPartCollisionHelper bch,boolean item){throw new AssertionError("facades excluded");}}
 static class Facades {Facade getFacade(Direction d){return null;}}
 static class Mode {boolean opaqueFacades=true;}
 static class PartHelper {static Mode getCableRenderMode(){return new Mode();}}
 static class Platform {static final Direction[] DIRECTIONS_WITH_NULL={Direction.DOWN,Direction.UP,Direction.NORTH,Direction.SOUTH,Direction.WEST,Direction.EAST,null};}
 static class SelectedPart {
  Direction side;Object part;
  SelectedPart(){} SelectedPart(Object part,Direction side){this.part=part;this.side=side;}
 }
 static class Host {
  Part center;Part[] parts=new Part[6];Part getPart(Direction d){return d==null?center:parts[d.ordinal()];}
  Facades getFacadeContainer(){return new Facades();}
  // SELECT
 }
 static double read(StringTokenizer t){return Double.longBitsToDouble(Long.parseUnsignedLong(t.nextToken(),16));}
 static int integer(StringTokenizer t){return Integer.parseInt(t.nextToken());}
 static void bounds(StringBuilder out,AABB b){for(double v:b.values())out.append(' ').append(Long.toUnsignedString(Double.doubleToLongBits(v),16));}
 public static void main(String[] args)throws Exception {
  var input=new BufferedReader(new InputStreamReader(System.in));String line;var out=new StringBuilder();
  while((line=input.readLine())!=null){var t=new StringTokenizer(line);String op=t.nextToken();
   if(op.equals("B")||op.equals("A")){
    var boxes=new ArrayList<AABB>();BusCollisionHelper helper;
    if(op.equals("B")){int side=integer(t);helper=new BusCollisionHelper(boxes,side==6?null:Direction.values()[side],integer(t)!=0);}
    else {var x=Direction.values()[integer(t)];var y=Direction.values()[integer(t)];var z=Direction.values()[integer(t)];helper=new BusCollisionHelper(boxes,x,y,z,integer(t)!=0);}
    helper.addBox(read(t),read(t),read(t),read(t),read(t),read(t));var point=new Vec3(read(t),read(t),read(t));
    out.append(helper.getWorldX().ordinal()).append(' ').append(helper.getWorldY().ordinal()).append(' ').append(helper.getWorldZ().ordinal()).append(' ').append(helper.isBBCollision()?1:0);
    bounds(out,boxes.get(0));out.append(' ').append(boxes.get(0).inflate(.002,.002,.002).contains(point)?1:0).append('\n');
   }else{
    int kind=integer(t),parts=integer(t),conn=integer(t),dense=integer(t);boolean visual=integer(t)!=0;var point=new Vec3(read(t),read(t),read(t));
    Host host=new Host();var cable=CableOracle.cable(kind);cable.connections=CableOracle.mask(conn);cable.dense=dense;
    if(kind!=0)host.center=new Center(cable);
    for(var side:Direction.values())if((parts&(1<<side.ordinal()))!=0){host.parts[side.ordinal()]=new Reporting();var p=new CableOracle.Part();p.length=3.0;cable.host.parts[side.ordinal()]=p;}
    var boxes=new ArrayList<AABB>();var slots=new ArrayList<Integer>();
    for(var side:Platform.DIRECTIONS_WITH_NULL){var part=host.getPart(side);if(part==null)continue;int before=boxes.size();part.getBoxes(new BusCollisionHelper(boxes,side,visual));while(slots.size()<boxes.size())slots.add(side==null?6:side.ordinal());}
    out.append(boxes.size());for(int i=0;i<boxes.size();i++){out.append(' ').append(slots.get(i));bounds(out,boxes.get(i));}
    var selected=host.selectPartLocal(point);out.append(' ').append(selected.part==null?-1:selected.side==null?6:selected.side.ordinal()).append('\n');
   }
  }
  System.out.print(out);
 }
}
'''

AABB = r'''
package net.minecraft.world.phys;
import java.lang.reflect.*;
public class AABB {
 private static final Class<?> TYPE;
 private static final Constructor<?> CTOR;
 private static final Method INFLATE,CONTAINS;
 private static final Field[] FIELDS=new Field[6];
 static {try{
  TYPE=Class.forName("ewx");CTOR=TYPE.getConstructor(double.class,double.class,double.class,double.class,double.class,double.class);
  INFLATE=TYPE.getMethod("c",double.class,double.class,double.class);CONTAINS=TYPE.getMethod("e",double.class,double.class,double.class);
  for(int i=0;i<6;i++)FIELDS[i]=TYPE.getField(String.valueOf((char)('a'+i)));
 }catch(Exception e){throw new ExceptionInInitializerError(e);}}
 private final Object value;
 private AABB(Object value){this.value=value;}
 public AABB(double a,double b,double c,double d,double e,double f){try{value=CTOR.newInstance(a,b,c,d,e,f);}catch(Exception ex){throw new RuntimeException(ex);}}
 public AABB inflate(double a,double b,double c){try{return new AABB(INFLATE.invoke(value,a,b,c));}catch(Exception ex){throw new RuntimeException(ex);}}
 public boolean contains(Vec3 p){try{return (boolean)CONTAINS.invoke(value,p.x(),p.y(),p.z());}catch(Exception ex){throw new RuntimeException(ex);}}
 public double[] values(){try{double[] result=new double[6];for(int i=0;i<6;i++)result[i]=FIELDS[i].getDouble(value);return result;}catch(Exception ex){throw new RuntimeException(ex);}}
}
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home', type=Path, default=Path('/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home'))
    args = parser.parse_args()
    upstream.verify(upstream.SOURCE)
    cp = minecraft_reference.prepare()
    cable_oracle.WORK = WORK
    src, cable_methods, source_files = cable_oracle.generate()
    mapping = (minecraft_reference.WORK / 'client_mappings.txt').read_text()
    assert 'net.minecraft.world.phys.AABB -> ewx:' in mapping
    aabb_mapping = mapping.split('net.minecraft.world.phys.AABB -> ewx:\n')[1].split('\nnet.')[0]
    for signature in ['net.minecraft.world.phys.AABB inflate(double,double,double) -> c', 'boolean contains(double,double,double) -> e']:
        assert signature in aabb_mapping
    reporting = method((BASE / 'parts/reporting/AbstractReportingPart.java').read_text(), 'public final void getBoxes(')
    selection = method((BASE / 'parts/CableBusContainer.java').read_text(), 'public SelectedPart selectPartLocal(')
    for name in ['parts/BusCollisionHelper.java', 'api/parts/IPartCollisionHelper.java']:
        dest = src / 'appeng' / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(BASE / name, dest)
    bindings = {
        'net/minecraft/core/Direction.java': '''package net.minecraft.core;
public enum Direction { DOWN(0,-1,0),UP(0,1,0),NORTH(0,0,-1),SOUTH(0,0,1),WEST(-1,0,0),EAST(1,0,0);
private final int x,y,z;Direction(int x,int y,int z){this.x=x;this.y=y;this.z=z;}public int getStepX(){return x;}public int getStepY(){return y;}public int getStepZ(){return z;} }''',
        'net/minecraft/world/phys/Vec3.java': 'package net.minecraft.world.phys;public record Vec3(double x,double y,double z) {}',
        'net/minecraft/world/phys/AABB.java': AABB,
        'oracle/PartGeometryOracle.java': JAVA.replace('// REPORTING_BOXES', reporting).replace('// SELECT', selection),
    }
    for name, body in bindings.items():
        path = src / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body + '\n')
    classes = WORK / 'classes'
    compiled = subprocess.run([str(args.java_home / 'bin/javac'), '-proc:none', '-cp', cp, '-d', str(classes), *map(str, src.rglob('*.java'))], capture_output=True)
    (WORK / 'javac.stderr').write_bytes(compiled.stderr)
    if compiled.returncode:
        raise RuntimeError(compiled.stderr.decode()[-6000:])
    bits = lambda n: struct.pack('>d', n).hex().lstrip('0') or '0'
    row_numbers = lambda numbers: ' '.join(bits(n) for n in numbers)
    rng = random.Random(0xae219218)
    special = [0., -0., 2., 4., 13., 16., -16., 32., float('inf'), -float('inf'), float('nan'), 2.**-1074, -2.**-1074]
    rows = []
    for side, visual in itertools.product(range(7), range(2)):
        fixtures = [[2., 2., 14., 14., 14., 16.], [4., 4., 13., 12., 12., 14.]]
        fixtures += [[v] * 6 for v in special]
        fixtures += [[rng.uniform(-32, 48) for _ in range(6)] for _ in range(512)]
        for box in fixtures:
            rows.append(f'B {side} {visual} ' + row_numbers(box + [rng.uniform(-1, 2) for _ in range(3)]))
    transform_cases = len(rows)
    for axes, visual in itertools.product(itertools.product(range(6), repeat=3), range(2)):
        for _ in range(8):
            values = [rng.choice(special) if rng.random() < .2 else rng.uniform(-32, 48) for _ in range(6)]
            rows.append('A ' + ' '.join(map(str, axes)) + f' {visual} ' + row_numbers(values + [.5] * 3))
    axis_cases = len(rows) - transform_cases
    # Valid primary reporting hosts; independent boundary probes include both
    # exact endpoints and adjacent double values, not only interior points.
    import math
    queries = [[.5] * 3, [0.] * 3, [1.] * 3]
    for edge in [0., 2/16, 3/16, 4/16, 5/16, 6/16, 10/16, 11/16, 12/16, 13/16, 14/16, 1.]:
        for offset in [-.002, .002]:
            value = edge + offset
            for n in [math.nextafter(value, -math.inf), value, math.nextafter(value, math.inf)]:
                for axis in range(3):
                    q = [.5] * 3
                    q[axis] = n
                    queries.append(q)
    for kind in range(6):
        for parts in (range(64) if kind < 4 else [0]):
            for conn, dense, visual in itertools.product([0, 63, 21, 42], [0, 63], range(2)):
                for point in [queries[rng.randrange(len(queries))], [rng.random() for _ in range(3)]]:
                    rows.append(f'H {kind} {parts} {conn} {dense} {visual} ' + row_numbers(point))
    for point in queries + [[float('nan'), .5, .5]]:
        for kind, parts in [(0, 63), (1, 63), (2, 0), (3, 21), (4, 0), (5, 0)]:
            rows.append(f'H {kind} {parts} 63 63 1 ' + row_numbers(point))
    data = ('\n'.join(rows) + '\n').encode()
    (WORK / 'input.txt').write_bytes(data)
    expected = subprocess.run([str(args.java_home / 'bin/java'), '-cp', str(classes) + os.pathsep + cp, 'oracle.PartGeometryOracle'], cwd=WORK, input=data, capture_output=True)
    (WORK / 'java.stderr').write_bytes(expected.stderr)
    expected.check_returncode()
    actual = subprocess.run(['cargo', 'run', '--quiet', '--locked', '--example', 'part_geometry_oracle'], cwd=ROOT, input=data, capture_output=True)
    (WORK / 'rust.stderr').write_bytes(actual.stderr)
    actual.check_returncode()
    (WORK / 'java.txt').write_bytes(expected.stdout)
    (WORK / 'rust.txt').write_bytes(actual.stdout)
    if expected.stdout != actual.stdout:
        for i, (java, rust) in enumerate(itertools.zip_longest(expected.stdout.splitlines(), actual.stdout.splitlines())):
            if java != rust:
                raise AssertionError(f'case {i}: {rows[i]}\nJava: {java}\nRust: {rust}')
    sha = lambda b: hashlib.sha256(b).hexdigest()
    source_files += ['parts/BusCollisionHelper.java', 'api/parts/IPartCollisionHelper.java', 'parts/reporting/AbstractReportingPart.java', 'parts/CableBusContainer.java']
    report = {
        'passed': True, 'cases': len(rows), 'side_transform_cases': transform_cases, 'explicit_axis_cases': axis_cases,
        'host_shape_selection_cases': len(rows) - transform_cases - axis_cases,
        'source_sha256': {name: sha((BASE / name).read_bytes()) for name in source_files},
        'method_sha256': {'reporting_boxes': sha(reporting.encode()), 'selection': sha(selection.encode()), **{name: sha(body.encode()) for name, body in cable_methods.items() if 'boxes' in name}},
        'binding_sha256': {name: sha(body.encode()) for name, body in bindings.items()},
        'rust_sha256': {name: sha((ROOT / name).read_bytes()) for name in ['mixins/ae2/part_geometry.rs', 'mixins/ae2/reporting.rs', 'mixins/ae2/cable.rs', 'examples/part_geometry_oracle.rs']},
        'minecraft_client_sha256': sha((minecraft_reference.WORK / 'client.jar').read_bytes()),
        'minecraft_mappings_sha256': sha((minecraft_reference.WORK / 'client_mappings.txt').read_bytes()),
        'input_sha256': sha(data), 'output_sha256': sha(actual.stdout),
        'scope': 'Verbatim BusCollisionHelper class, Reporting getBoxes, CableBus selectPartLocal and carried cable box methods; real pinned Minecraft AABB construction, inflation and containment',
        'limits': ['Direction vectors, part storage, cable neighbor/connection state and absent facades are controlled bindings', 'Canonical NaN output comparison; NaN payload bits are not asserted', 'No full VoxelShape union/cache, native world collision callers, part admission/input/render or complete lifecycle proof'],
    }
    (ROOT / 'docs/part-geometry-oracle.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
