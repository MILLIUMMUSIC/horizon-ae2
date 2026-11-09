#!/usr/bin/env python3
"""Original QuadRotator with real pinned JOML, MC winding and Fabric accessors.

Only default scalar JOML options are covered. Model loading, face-normal cache,
AO, mesh lighting and native rendering/world activation are separate work.
"""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import random
import re
import shutil
import struct
import subprocess
import zipfile

import minecraft_reference
import upstream
from inscriber_oracle import method

ROOT = upstream.ROOT
BASE = upstream.SOURCE / 'src/main/java/appeng'
WORK = ROOT / 'target/quad-rotation-oracle'
JOML = minecraft_reference.WORK / 'libraries/org/joml/joml/1.10.5/joml-1.10.5.jar'

ORACLE = r'''
package oracle;
import java.io.*;import java.util.*;import java.lang.reflect.*;
import appeng.api.orientation.BlockOrientation;
import appeng.client.render.cablebus.QuadRotator;
import appeng.thirdparty.fabric.*;
import net.minecraft.core.Direction;
public class QuadRotationOracle {
 static class Quad extends MutableQuadViewImpl {
  void load(StringTokenizer t){for(int i=0;i<35;i++)data[i]=Integer.parseUnsignedInt(t.nextToken(),16);nominalFace=Direction.values()[Integer.parseInt(t.nextToken())];isGeometryInvalid=Integer.parseInt(t.nextToken())!=0;}
  String output(boolean identity){var s=new StringBuilder().append(nominalFace.ordinal()).append(' ').append(isGeometryInvalid?1:0);for(int i=0;i<35;i++){int v=data[i];if(!identity&&i>=3&&(i-3)%8<3)v=Float.floatToIntBits(Float.intBitsToFloat(v));s.append(' ').append(Integer.toUnsignedString(v,16));}return s.toString();}
 }
 public static void main(String[] args)throws Exception {
  if(org.joml.Options.USE_MATH_FMA||org.joml.Options.FASTMATH||org.joml.Options.SIN_LOOKUP)throw new AssertionError("nondefault JOML options");
  if(args.length!=0){
   for(var r:BlockOrientation.values()){var q=r.getQuaternion();System.out.print("Q");for(float v:new float[]{q.x,q.y,q.z,q.w})System.out.print(" "+Integer.toUnsignedString(Float.floatToIntBits(v),16));for(var d:Direction.values())System.out.print(" "+r.rotate(d).ordinal());System.out.println();}
   Class<?> dir=Class.forName("ji"),info=Class.forName("geq"),vertex=Class.forName("geq$b");
   var a=info.getDeclaredMethod("a",dir);var b=info.getDeclaredMethod("a",int.class);a.setAccessible(true);b.setAccessible(true);
   for(Object d:dir.getEnumConstants()){Object face=a.invoke(null,d);System.out.print("V");for(int i=0;i<4;i++){Object v=b.invoke(face,i);for(String f:new String[]{"a","b","c"}){var field=vertex.getDeclaredField(f);field.setAccessible(true);System.out.print(" "+field.getInt(v));}}System.out.println();}
   return;
  }
  var reader=new BufferedReader(new InputStreamReader(System.in));String line;var out=new StringBuilder();
  while((line=reader.readLine())!=null){var t=new StringTokenizer(line);String op=t.nextToken();
   if(op.equals("N")){float[] v=new float[4];for(int i=0;i<4;i++)v[i]=Float.intBitsToFloat(Integer.parseUnsignedInt(t.nextToken(),16));out.append(Integer.toUnsignedString(NormalHelper.packNormal(v[0],v[1],v[2],v[3]),16)).append('\n');continue;}
   int facing=Integer.parseInt(t.nextToken()),spin=Integer.parseInt(t.nextToken());var quad=new Quad();quad.load(t);boolean identity=false;
   try{var rotation=BlockOrientation.get(Direction.values()[facing],spin);identity=rotation.isRedundant();QuadRotator.get(rotation).transform(quad);out.append("OK ");}
   catch(ArrayIndexOutOfBoundsException e){out.append("ERR ");identity=true;}
   out.append(quad.output(identity)).append('\n');
  }
  System.out.print(out);
 }
}
'''


def generate():
    src = WORK / 'src'
    src.mkdir(parents=True, exist_ok=True)
    copied = ['client/render/cablebus/QuadRotator.java', 'thirdparty/fabric/ColorHelper.java', 'thirdparty/fabric/EncodingFormat.java']
    for name in copied:
        dest = src / 'appeng' / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(BASE / name, dest)
    orientation = (BASE / 'api/orientation/BlockOrientation.java').read_text()
    constants = orientation[orientation.index('    DOWN_NORTH('):orientation.index('    // @formatter:on')]
    ctor = method(orientation, 'BlockOrientation(int angleX,')
    orientation_methods = [method(orientation, sig) for sig in ['public boolean isRedundant(', 'public Quaternionf getQuaternion(', 'public Direction rotate(', 'public static BlockOrientation get(Direction facing, int spin)']]
    body = '''package appeng.api.orientation;
import org.joml.*;import net.minecraft.core.Direction;import net.minecraft.util.Mth;
public enum BlockOrientation {
CONSTANTS
private final int angleX,angleY,angleZ,spin;private final Quaternionf quaternion;private final Transformation transformation;private final Direction[] rotatedSideTo,rotatedSideFrom;
static class Transformation {Transformation(Matrix4f ignored){}static Transformation identity(){return new Transformation(null);}}
CTOR
METHODS
}'''.replace('CONSTANTS', constants).replace('CTOR', ctor).replace('METHODS', '\n'.join(orientation_methods))
    methods = {'orientation_constructor': ctor, **{f'orientation_{i}': v for i, v in enumerate(orientation_methods)}}
    normal = (BASE / 'thirdparty/fabric/NormalHelper.java').read_text()
    normal_methods = [method(normal, s) for s in ['public static int packNormal(float ', 'public static float getPackedNormalComponent(']]
    methods.update({f'normal_{i}': v for i, v in enumerate(normal_methods)})
    read = (BASE / 'thirdparty/fabric/QuadViewImpl.java').read_text()
    write = (BASE / 'thirdparty/fabric/MutableQuadViewImpl.java').read_text()
    getters = [method(read, s) for s in ['public int normalFlags(', 'public final Direction cullFace(', 'public final Direction nominalFace(', 'public Vector3f copyPos(', 'public boolean hasNormal(', 'protected final int normalIndex(', 'public Vector3f copyNormal(', 'public final void toVanilla(']]
    setters = [method(write, s) for s in ['public MutableQuadViewImpl pos(', 'protected void normalFlags(', 'public MutableQuadViewImpl normal(', 'public final MutableQuadViewImpl cullFace(', 'public final MutableQuadViewImpl nominalFace(', 'public final MutableQuadViewImpl fromVanilla(int[] ']]
    methods.update({f'quad_read_{i}': v for i, v in enumerate(getters)})
    methods.update({f'quad_write_{i}': v for i, v in enumerate(setters)})
    mutable = '''package appeng.thirdparty.fabric;
import static appeng.thirdparty.fabric.EncodingFormat.*;
import org.joml.Vector3f;import net.minecraft.core.Direction;import org.jetbrains.annotations.Nullable;
public class MutableQuadViewImpl implements MutableQuadView {
protected int[] data=new int[35];protected int baseIndex=0;protected Direction nominalFace;protected boolean isGeometryInvalid;
static final int VANILLA_VERTEX_STRIDE=8,VANILLA_QUAD_STRIDE=32;
GETTERS
SETTERS
}'''.replace('GETTERS', '\n'.join(getters)).replace('SETTERS', '\n'.join(setters))
    bindings = {
        'appeng/api/orientation/BlockOrientation.java': body,
        'appeng/thirdparty/fabric/MutableQuadViewImpl.java': mutable,
        'appeng/thirdparty/fabric/NormalHelper.java': 'package appeng.thirdparty.fabric;import net.minecraft.util.Mth;public class NormalHelper {' + '\n'.join(normal_methods) + '}',
        'appeng/thirdparty/fabric/MutableQuadView.java': '''package appeng.thirdparty.fabric;import org.joml.Vector3f;import net.minecraft.core.Direction;
public interface MutableQuadView {Vector3f copyPos(int i,Vector3f v);Vector3f copyNormal(int i,Vector3f v);boolean hasNormal(int i);Direction nominalFace();Direction cullFace();MutableQuadView cullFace(Direction d);MutableQuadView nominalFace(Direction d);void toVanilla(int[] data,int start);MutableQuadView fromVanilla(int[] data,int start);MutableQuadView pos(int i,float x,float y,float z);MutableQuadView normal(int i,float x,float y,float z);
default MutableQuadView pos(int i,Vector3f v){return pos(i,v.x,v.y,v.z);}default MutableQuadView normal(int i,Vector3f v){return normal(i,v.x,v.y,v.z);}}''',
        'appeng/thirdparty/fabric/RenderContext.java': 'package appeng.thirdparty.fabric;public interface RenderContext {interface QuadTransform {boolean transform(MutableQuadView quad);}}',
        'appeng/thirdparty/fabric/QuadView.java': 'package appeng.thirdparty.fabric;public interface QuadView {int VANILLA_VERTEX_STRIDE=8,VANILLA_QUAD_STRIDE=32;}',
        'appeng/thirdparty/fabric/GeometryHelper.java': 'package appeng.thirdparty.fabric;public class GeometryHelper {public static final int FLAG_BIT_COUNT=3;}',
        'appeng/thirdparty/fabric/ModelHelper.java': 'package appeng.thirdparty.fabric;import net.minecraft.core.Direction;public class ModelHelper {public static final int NULL_FACE_ID=6;static final Direction[] FACES=java.util.Arrays.copyOf(Direction.values(),7);public static Direction faceFromIndex(int n){return FACES[n];}public static int toFaceIndex(Direction d){return d==null?6:d.ordinal();}}',
        'net/minecraft/core/Direction.java': '''package net.minecraft.core;import org.joml.Vector3f;
public enum Direction {DOWN(0,-1,0),UP(0,1,0),NORTH(0,0,-1),SOUTH(0,0,1),WEST(-1,0,0),EAST(1,0,0);
int x,y,z;Direction(int x,int y,int z){this.x=x;this.y=y;this.z=z;}public Vector3f step(){return new Vector3f(x,y,z);}
public static Direction getNearest(float x,float y,float z){try{return values()[((Enum<?>)Class.forName("ji").getMethod("a",float.class,float.class,float.class).invoke(null,x,y,z)).ordinal()];}catch(Exception e){throw new RuntimeException(e);}}}''',
        'net/minecraft/util/Mth.java': '''package net.minecraft.util;public class Mth {
public static final float DEG_TO_RAD;static{try{DEG_TO_RAD=Class.forName("ayo").getField("d").getFloat(null);}catch(Exception e){throw new ExceptionInInitializerError(e);}}
public static int smallestEncompassingPowerOfTwo(int n){try{return (int)Class.forName("ayo").getMethod("c",int.class).invoke(null,n);}catch(Exception e){throw new RuntimeException(e);}}
public static float clamp(float n,float a,float b){try{return (float)Class.forName("ayo").getMethod("a",float.class,float.class,float.class).invoke(null,n,a,b);}catch(Exception e){throw new RuntimeException(e);}}}''',
        'net/minecraft/client/renderer/block/model/FaceBakery.java': '''package net.minecraft.client.renderer.block.model;
import net.minecraft.core.Direction;import java.lang.reflect.*;
public class FaceBakery {
public static final int VERTEX_INT_SIZE;private static final Object BAKERY;private static final Method WIND;private static final Object[] DIR;
static{try{Class<?> c=Class.forName("ggd"),d=Class.forName("ji");var stride=c.getDeclaredField("a");stride.setAccessible(true);VERTEX_INT_SIZE=stride.getInt(null);BAKERY=c.getConstructor().newInstance();WIND=c.getDeclaredMethod("a",int[].class,d);WIND.setAccessible(true);DIR=d.getEnumConstants();}catch(Exception e){throw new ExceptionInInitializerError(e);}}
public void recalculateWinding(int[] data,Direction side){try{WIND.invoke(BAKERY,data,DIR[side.ordinal()]);}catch(Exception e){throw new RuntimeException(e);}}}''',
        'net/minecraft/client/renderer/block/model/BlockModel.java': 'package net.minecraft.client.renderer.block.model;public class BlockModel {public static final FaceBakery FACE_BAKERY=new FaceBakery();}',
        'com/mojang/blaze3d/vertex/VertexFormat.java': 'package com.mojang.blaze3d.vertex;public class VertexFormat {}',
        'com/mojang/blaze3d/vertex/DefaultVertexFormat.java': 'package com.mojang.blaze3d.vertex;public class DefaultVertexFormat {public static final VertexFormat BLOCK=new VertexFormat();}',
        'org/jetbrains/annotations/Nullable.java': 'package org.jetbrains.annotations;import java.lang.annotation.*;@Target({ElementType.TYPE_USE,ElementType.PARAMETER,ElementType.METHOD})public @interface Nullable {}',
        'oracle/QuadRotationOracle.java': ORACLE,
    }
    for name, text in bindings.items():
        dest = src / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(text + '\n')
    return src, bindings, methods, copied


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write-tables', action='store_true', help='refresh committed derived constants and JOML licence')
    parser.add_argument('--java-home', type=Path, default=Path('/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home'))
    args = parser.parse_args()
    upstream.verify(upstream.SOURCE)
    cp = minecraft_reference.prepare()
    src, bindings, methods, copied = generate()
    classes = WORK / 'classes'
    binary = lambda name: str(args.java_home / 'bin' / name)
    compiled = subprocess.run([binary('javac'), '-proc:none', '-cp', cp, '-d', str(classes), *map(str, src.rglob('*.java'))], capture_output=True)
    (WORK / 'javac.stderr').write_bytes(compiled.stderr)
    if compiled.returncode:
        raise RuntimeError(compiled.stderr.decode()[-6000:])
    command = [binary('java'), '-cp', str(classes) + os.pathsep + cp, 'oracle.QuadRotationOracle']
    tables = subprocess.run([*command, '--tables'], capture_output=True, cwd=WORK)
    (WORK / 'tables.stderr').write_bytes(tables.stderr)
    if tables.returncode:
        raise RuntimeError(tables.stderr.decode()[-6000:])
    qs, faces, vertices = [], [], []
    for line in tables.stdout.decode().splitlines():
        s = line.split()
        if s[0] == 'Q':
            qs.append([int(n, 16) for n in s[1:5]])
            faces.append([int(n) for n in s[5:]])
        elif s[0] == 'V':
            values = [int(n) for n in s[1:]]
            vertices.append([values[i:i+3] for i in range(0, 12, 3)])
    assert len(qs) == len(faces) == 24 and len(vertices) == 6
    table_text = 'const QUATERNIONS: [[u32; 4]; 24] = [\n' + ''.join('    [' + ', '.join(f'0x{v:08x}' for v in q) + '],\n' for q in qs) + '];\n'
    table_text += 'const ROTATED_FACES: [[u8; 6]; 24] = ' + str(faces) + ';\n'
    table_text += 'const FACE_VERTICES: [[[usize; 3]; 4]; 6] = ' + str(vertices) + ';\n'
    table_text = subprocess.run(['rustfmt', '--edition', '2024'], input=table_text, text=True, capture_output=True, check=True).stdout
    port = ROOT / 'mixins/ae2/quad_rotation.rs'
    start_marker, end_marker = '// BEGIN SOURCE ROTATION TABLES\n', '// END SOURCE ROTATION TABLES'
    text = port.read_text()
    current = text.split(start_marker)[1].split(end_marker)[0]
    license_text = zipfile.ZipFile(JOML).read('META-INF/LICENSE')
    if args.write_tables:
        port.write_text(text.replace(start_marker + current + end_marker, start_marker + table_text + end_marker))
        (ROOT / 'LICENSES/JOML-MIT').write_bytes(license_text)
    else:
        assert current == table_text, 'source rotation table differs; rerun with --write-tables after reviewing the pin'
        assert (ROOT / 'LICENSES/JOML-MIT').read_bytes() == license_text
    # Constants are checked independently before comparing any Rust operations.
    rng = random.Random(0xae219218)
    bits = lambda n: int.from_bytes(struct.pack('>f', n), 'big')
    def quad_row(facing, spin, nominal, cull, normal_mask, p, invalid=False):
        header = [(rng.getrandbits(32) & ~0x3c7) | cull | (normal_mask << 6), rng.getrandbits(32), rng.getrandbits(32)]
        data = header[:]
        for point in p:
            data += [*point, rng.getrandbits(32), bits(rng.uniform(-1,2)), bits(rng.uniform(-1,2)), rng.getrandbits(32), rng.getrandbits(32)]
        return f'Q {facing} {spin} ' + ' '.join(f'{n:x}' for n in data) + f' {nominal} {int(invalid)}'
    bounds = [bits(2/16), bits(14/16), bits(2/16), bits(14/16), bits(0), bits(2/16)]
    # FaceInfo indexes are Ymin,Ymax,Zmin,Zmax,Xmin,Xmax, not XYZ pairs.
    bound_order = [bounds[2], bounds[3], bounds[4], bounds[5], bounds[0], bounds[1]]
    rows = []
    for facing, spin, nominal, cull, normals in itertools.product(range(6), range(4), range(6), range(7), range(16)):
        p = [[bound_order[i] for i in v] for v in vertices[nominal]]
        rows.append(quad_row(facing, spin, nominal, cull, normals, p))
    panel_cases = len(rows)
    for _ in range(12000):
        facing, spin, nominal = rng.randrange(6), rng.randrange(-28,29), rng.randrange(6)
        p = [[rng.getrandbits(32) if rng.random()<.1 else bits(rng.uniform(-2000,2000)) for _ in range(3)] for _ in range(4)]
        if rng.random() < .3:
            p = [p[0][:] for _ in range(4)]
        rows.append(quad_row(facing, spin, nominal, rng.randrange(7), rng.randrange(16), p, bool(rng.randrange(2))))
    malformed_cases = len(rows) - panel_cases
    special = [bits(v) for v in [-float('inf'),float('inf'),float('nan'),-2.,-1.,-0.,0.,1.,2.]]
    rows += ['N ' + ' '.join(f'{v:x}' for v in values) for values in itertools.product(special, repeat=4)]
    for _ in range(4000):
        rows.append('N ' + ' '.join(f'{rng.getrandbits(32):x}' for _ in range(4)))
    data = ('\n'.join(rows)+'\n').encode()
    (WORK / 'input.txt').write_bytes(data)
    expected = subprocess.run(command, input=data, capture_output=True, cwd=WORK)
    (WORK / 'java.stderr').write_bytes(expected.stderr)
    if expected.returncode:
        raise RuntimeError(expected.stderr.decode()[-6000:])
    actual = subprocess.run(['cargo','run','--quiet','--locked','--example','quad_rotation_oracle'], input=data, capture_output=True, cwd=ROOT)
    (WORK / 'rust.stderr').write_bytes(actual.stderr)
    actual.check_returncode()
    (WORK / 'java.txt').write_bytes(expected.stdout)
    (WORK / 'rust.txt').write_bytes(actual.stdout)
    if expected.stdout != actual.stdout:
        for i,(java,rust) in enumerate(itertools.zip_longest(expected.stdout.splitlines(),actual.stdout.splitlines())):
            if java != rust:
                raise AssertionError(f'case {i}: {rows[i]}\nJava {java}\nRust {rust}')
    sha = lambda b: hashlib.sha256(b).hexdigest()
    source_files = copied + ['api/orientation/BlockOrientation.java','thirdparty/fabric/NormalHelper.java','thirdparty/fabric/QuadViewImpl.java','thirdparty/fabric/MutableQuadViewImpl.java']
    report = {'passed':True,'cases':len(rows),'panel_quad_cases':panel_cases,'arbitrary_quad_cases':malformed_cases,'normal_pack_cases':len(rows)-panel_cases-malformed_cases,
        'source_sha256':{name:sha((BASE/name).read_bytes()) for name in source_files},'method_sha256':{name:sha(body.encode()) for name,body in methods.items()},
        'binding_sha256':{name:sha(body.encode()) for name,body in bindings.items()},'table_sha256':sha(tables.stdout),
        'rust_sha256':{name:sha((ROOT/name).read_bytes()) for name in ['mixins/ae2/quad_rotation.rs','examples/quad_rotation_oracle.rs']},
        'minecraft_client_sha256':sha((minecraft_reference.WORK/'client.jar').read_bytes()),'minecraft_mappings_sha256':sha((minecraft_reference.WORK/'client_mappings.txt').read_bytes()),'joml_sha256':sha(JOML.read_bytes()),
        'input_sha256':sha(data),'output_sha256':sha(actual.stdout),
        'scope':'Original QuadRotator class, source BlockOrientation constructor/direction methods and extracted Fabric packed accessors; real MC FaceBakery/FaceInfo/Mth and JOML 1.10.5 default scalar arithmetic',
        'limits':['Controlled quad storage, direction vector, VertexFormat stride and unused Transformation bindings; nominal faces are non-null and cull indices valid', 'NaN position outputs canonicalized after nonidentity rotations; identity is compared as untouched raw words', 'No optional JVM JOML modes, full face-normal/AO/mesh caches, model loading, native GPU or side-part world activation proof']}
    (ROOT/'docs/quad-rotation-oracle.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))


if __name__ == '__main__':
    main()
