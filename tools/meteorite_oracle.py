#!/usr/bin/env python3
"""Compare full source meteorite placement/write/RNG traces in a controlled world.

Original placer/putter/fallout/settings classes are nested without changing their
method bodies. Terrain, block properties, biomes, chunks and entity query results
are controlled bindings. This does not execute native worldgen/physics/lifecycle.
"""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import random
import re
import struct
import subprocess

import minecraft_reference
import upstream

ROOT=upstream.ROOT
WORK=ROOT/'target/meteorite-oracle'
BASE=upstream.SOURCE/'src/main/java/appeng/worldgen/meteorite'
FILES=['MeteoritePlacer.java','MeteoriteBlockPutter.java','CraterType.java','PlacedMeteoriteSettings.java','Constants.java'] + ['fallout/'+n+'.java' for n in ['Fallout','FalloutCopy','FalloutSand','FalloutSnow','FalloutMode']]

HOST=r'''
package oracle;
import java.io.*;import java.util.*;import java.util.stream.Stream;import com.google.common.collect.ImmutableList;
public class MeteoriteOracle {
 record P(int x,int y,int z) implements Comparable<P> {public int compareTo(P p){int c=Integer.compare(x,p.x);if(c==0)c=Integer.compare(y,p.y);if(c==0)c=Integer.compare(z,p.z);return c;}public String toString(){return x+","+y+","+z;}}
 static class BlockPos {
  int x,y,z;BlockPos(int x,int y,int z){this.x=x;this.y=y;this.z=z;}
  int getX(){return x;}int getY(){return y;}int getZ(){return z;}P key(){return new P(x,y,z);}
  BlockPos offset(int dx,int dy,int dz){return new BlockPos(x+dx,y+dy,z+dz);}
  long asLong(){throw new UnsupportedOperationException("settings codec excluded");}static BlockPos of(long n){throw new UnsupportedOperationException("settings codec excluded");}
 }
 static class MutableBlockPos extends BlockPos {
  MutableBlockPos(){super(0,0,0);}void setX(int x){this.x=x;}void setY(int y){this.y=y;}void setZ(int z){this.z=z;}
  void set(BlockPos p){x=p.x;y=p.y;z=p.z;}void move(Direction side){if(side!=Direction.DOWN)throw new AssertionError();y--;}
 }
 enum Direction {DOWN,UP}
 static class BoundingBox {
  int[] min,max;BoundingBox(int[] a,int[] b){min=a;max=b;}
  int minX(){return min[0];}int maxX(){return max[0];}int minZ(){return min[2];}int maxZ(){return max[2];}
  boolean isInside(BlockPos p){return p.x>=min[0]&&p.x<=max[0]&&p.y>=min[1]&&p.y<=max[1]&&p.z>=min[2]&&p.z<=max[2];}
  BlockPos getCenter(){try{Class<?> c=Class.forName("ejj");Object box=c.getConstructor(int.class,int.class,int.class,int.class,int.class,int.class).newInstance(min[0],min[1],min[2],max[0],max[1],max[2]);Object p=c.getMethod("g").invoke(box);Class<?> v=Class.forName("kh");return new BlockPos((int)v.getMethod("u").invoke(p),(int)v.getMethod("v").invoke(p),(int)v.getMethod("w").invoke(p));}catch(Exception e){throw new RuntimeException(e);}}
 }
 static class AABB {double[] v;AABB(double... v){this.v=v;}}
 static class CompoundTag {
  void putLong(String k,long v){throw new UnsupportedOperationException();}void putFloat(String k,float v){throw new UnsupportedOperationException();}void putByte(String k,byte v){throw new UnsupportedOperationException();}void putBoolean(String k,boolean v){throw new UnsupportedOperationException();}
  long getLong(String k){throw new UnsupportedOperationException();}float getFloat(String k){throw new UnsupportedOperationException();}byte getByte(String k){throw new UnsupportedOperationException();}boolean getBoolean(String k){throw new UnsupportedOperationException();}
 }
 record TagKey<T>(int bit) {} static class Biome {}
 static class Holder<T> {int mask;Holder(int n){mask=n;}boolean is(TagKey<T> tag){return (mask&(1<<tag.bit))!=0;}}
 static class BiomeTags {static final TagKey<Biome> IS_BADLANDS=new TagKey<>(0),IS_BEACH=new TagKey<>(2),IS_FOREST=new TagKey<>(5);}
 static class Tags {static class Biomes {static final TagKey<Biome> IS_SNOWY=new TagKey<>(1),IS_SANDY=new TagKey<>(3),IS_PLAINS=new TagKey<>(4),IS_COLD=new TagKey<>(6);}}
 static class Block {
  static final int UPDATE_ALL=3;String id;static Map<String,Block> all=new HashMap<>();Block(String id){this.id=id;}static Block of(String id){return all.computeIfAbsent(id,Block::new);}
  BlockState defaultBlockState(){return BlockState.of(id.equals("minecraft:snow")?id+"[layers=1]":id);}
 }
 static class BlockTags {static final String REPLACEABLE="replaceable";}
 static class AmethystClusterBlock {static final String FACING="facing";}
 static class Fluids {static final Object WATER=new Object();}
 static class Fluid {boolean empty;Fluid(boolean empty){this.empty=empty;}boolean isEmpty(){return empty;}}
 static class BlockState {
  String name;Block block;static Map<String,BlockState> all=new HashMap<>();
  static BlockState of(String name){return all.computeIfAbsent(name,s->{var v=new BlockState();v.name=s;v.block=Block.of(s.split("\\[")[0]);return v;});}
  Block getBlock(){return block;}
  boolean isAir(){return Set.of("minecraft:air","minecraft:cave_air","minecraft:void_air").contains(block.id);}
  boolean canBeReplaced(){return isAir()||Set.of("minecraft:water","minecraft:lava","minecraft:short_grass","minecraft:snow").contains(block.id);}
  boolean isSolid(){return !isAir()&&!Set.of("minecraft:water","minecraft:lava","minecraft:short_grass","minecraft:snow","test:tagged","ae2:small_quartz_bud","ae2:medium_quartz_bud","ae2:large_quartz_bud").contains(block.id);}
  Fluid getFluidState(){return new Fluid(!Set.of("minecraft:water","minecraft:lava").contains(block.id)&&!name.contains("waterlogged=true"));}
  boolean is(String tag){return tag.equals(BlockTags.REPLACEABLE)&&Set.of("test:tagged","minecraft:short_grass").contains(block.id);}
  BlockState setValue(String property,Direction d){if(!property.equals("facing")||d!=Direction.UP)throw new AssertionError();return of(block.id+"[facing=up]");}
 }
 static class Blocks {VANILLA_BLOCKS}
 record Definition(Block block) {}
 static class AEBlocks {AE_BLOCKS}
 static class AEConfig {static boolean flawless,presses;static AEConfig instance(){return new AEConfig();}boolean isSpawnFlawlessOnlyEnabled(){return flawless;}boolean isSpawnPressesInMeteoritesEnabled(){return presses;}}
 static class Util {static <T>T getRandom(List<T> values,RandomSource random){return values.get(random.nextInt(values.size()));}}
 static class RandomSource {
  java.util.Random rng;String label;LevelAccessor level;
  RandomSource(java.util.Random rng,String label,LevelAccessor level){this.rng=rng;this.label=label;this.level=level;}
  int nextInt(int bound){int value=rng.nextInt(bound);level.trace.append(label).append("I:").append(bound).append(':').append(value).append(';');return value;}
  float nextFloat(){float value=rng.nextFloat();level.trace.append(label).append("F:").append(Integer.toUnsignedString(Float.floatToIntBits(value),16)).append(';');return value;}
  double nextDouble(){double value=rng.nextDouble();level.trace.append(label).append("D:").append(Long.toUnsignedString(Double.doubleToLongBits(value),16)).append(';');return value;}
 }
 static class ChunkAccess {LevelAccessor level;ChunkAccess(LevelAccessor l){level=l;}BlockState getBlockState(BlockPos p){return level.getBlockState(p);}}
 static class ItemEntity {LevelAccessor level;ItemEntity(LevelAccessor l){level=l;}void discard(){level.trace.append("X;");}}
 static class LevelAccessor {
  int max,sea,floor,terrain,mask;RandomSource placement,levelRandom;StringBuilder trace=new StringBuilder();Map<P,BlockState> blocks=new TreeMap<>();
  LevelAccessor(int max,int sea,int floor,int terrain,int mask,long seed,long levelSeed,boolean alias){this.max=max;this.sea=sea;this.floor=floor;this.terrain=terrain;this.mask=mask;var random=new java.util.Random(seed);placement=new RandomSource(random,"P",this);levelRandom=new RandomSource(alias?random:new java.util.Random(levelSeed),"L",this);}
  int getMaxBuildHeight(){return max;}int getSeaLevel(){return sea;}RandomSource getRandom(){return levelRandom;}
  int biomeMask(BlockPos p){return mask^Math.floorMod(p.x+2*p.y+3*p.z,64);}
  Holder<Biome> getBiome(BlockPos p){int n=biomeMask(p);trace.append("B:").append(p.key()).append(':').append(n).append(';');return new Holder<>(n);}
  BlockState initial(BlockPos p){
   if(p.y < -16 || p.y>=max)return Blocks.AIR.defaultBlockState();
   int n=Math.floorMod(p.x*31+p.z*17+terrain,23),ground=floor+n%5-2;
   if(p.y<=ground){if(n%11==0&&p.y>=ground-2)return Blocks.BEDROCK.defaultBlockState();return (p.y==ground?Blocks.DIRT:Blocks.STONE).defaultBlockState();}
   if(p.y==ground+1){switch(n%7){case 0:return Blocks.SHORT_GRASS.defaultBlockState();case 1:return Blocks.SNOW.defaultBlockState();case 2:return Blocks.CAVE_AIR.defaultBlockState();case 3:return Blocks.VOID_AIR.defaultBlockState();case 4:return BlockState.of("test:tagged");}}
   if(p.y==ground+2&&n%3==0)return BlockState.of("minecraft:oak_stairs[facing=east,waterlogged=true]");
   if(p.y<sea&&n%4==0)return Blocks.WATER.defaultBlockState();return Blocks.AIR.defaultBlockState();
  }
  BlockState getBlockState(BlockPos p){return blocks.getOrDefault(p.key(),initial(p));}
  boolean isEmptyBlock(BlockPos p){return getBlockState(p).isAir();}
  boolean setBlock(BlockPos p,BlockState state,int flags){trace.append("S:").append(p.key()).append(':').append(state.name).append(':').append(flags).append(';');if(p.y < -16 || p.y>=max)return false;blocks.put(p.key(),state);return true;}
  List<ItemEntity> getEntitiesOfClass(Class<ItemEntity> c,AABB bounds){trace.append("E");for(double n:bounds.v)trace.append(':').append(Long.toUnsignedString(Double.doubleToLongBits(n),16));trace.append(';');return List.of(new ItemEntity(this));}
  ChunkAccess getChunk(BlockPos p){return new ChunkAccess(this);}
  void scheduleTick(BlockPos p,Object fluid,int delay){if(fluid!=Fluids.WATER)throw new AssertionError();trace.append("W:").append(p.key()).append(':').append(delay).append(';');}
  String finish(){var out=new StringBuilder(trace);out.append("END:");for(var e:blocks.entrySet())out.append(e.getKey()).append(':').append(e.getValue().name).append(';');return out.toString();}
 }
 SOURCE_CLASSES
 public static void main(String[] args)throws Exception {
  var input=new BufferedReader(new InputStreamReader(System.in));String line;
  while((line=input.readLine())!=null){var t=new StringTokenizer(line);int x=Integer.parseInt(t.nextToken()),y=Integer.parseInt(t.nextToken()),z=Integer.parseInt(t.nextToken());float radius=Float.intBitsToFloat(Integer.parseUnsignedInt(t.nextToken(),16));int kind=Integer.parseInt(t.nextToken()),fallout=Integer.parseInt(t.nextToken());boolean pure=t.nextToken().equals("1"),lake=t.nextToken().equals("1");AEConfig.flawless=t.nextToken().equals("1");AEConfig.presses=t.nextToken().equals("1");
   int[] min={Integer.parseInt(t.nextToken()),Integer.parseInt(t.nextToken()),Integer.parseInt(t.nextToken())},max={Integer.parseInt(t.nextToken()),Integer.parseInt(t.nextToken()),Integer.parseInt(t.nextToken())};
   int top=Integer.parseInt(t.nextToken()),sea=Integer.parseInt(t.nextToken()),floor=Integer.parseInt(t.nextToken()),terrain=Integer.parseInt(t.nextToken()),mask=Integer.parseInt(t.nextToken());long seed=Long.parseLong(t.nextToken()),levelSeed=Long.parseLong(t.nextToken());boolean alias=t.nextToken().equals("1");
   var level=new LevelAccessor(top,sea,floor,terrain,mask,seed,levelSeed,alias);var settings=new PlacedMeteoriteSettings(new BlockPos(x,y,z),radius,CraterType.values()[kind],FalloutMode.values()[fallout],pure,lake);
   MeteoritePlacer.place(level,settings,new BoundingBox(min,max),level.placement);System.out.println(level.finish());
  }
 }
}
'''


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home',type=Path,default=Path('/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home'))
    args=parser.parse_args();upstream.verify(upstream.SOURCE);WORK.mkdir(parents=True,exist_ok=True)
    cp=minecraft_reference.prepare()
    originals={name:(BASE/name).read_text() for name in FILES}
    nested=[]
    for name,body in originals.items():
        body=re.sub(r'^package [^;]+;\s*|^import [^;]+;\s*','',body,flags=re.M)
        name=Path(name).stem
        body=re.sub(r'public (final )?class '+name+r'\b',lambda m:'static '+(m[1] or '')+'class '+name,body,count=1)
        body=body.replace('public enum '+name,'enum '+name)
        nested.append(body)
    all_source='\n'.join(originals.values())
    vanilla=set(re.findall(r'\bBlocks\.([A-Z_]+)',all_source))|{'BEDROCK','SHORT_GRASS','CAVE_AIR','VOID_AIR','DIRT','STONE','SNOW','WATER','AIR'}
    ae=set(re.findall(r'\bAEBlocks\.([A-Z_]+)',all_source))
    vanilla_fields='\n'.join(f'static final Block {name}=Block.of("minecraft:{name.lower()}");' for name in sorted(vanilla))
    ae_fields='\n'.join(f'static final Definition {name}=new Definition(Block.of("ae2:{name.lower()}"));' for name in sorted(ae))
    java=HOST.replace('VANILLA_BLOCKS',vanilla_fields).replace('AE_BLOCKS',ae_fields).replace('SOURCE_CLASSES','\n'.join(nested))
    (WORK/'MeteoriteOracle.java').write_text(java)
    classes=WORK/'classes'
    compiled=subprocess.run([str(args.java_home/'bin/javac'),'-proc:none','-cp',cp,'-d',str(classes),str(WORK/'MeteoriteOracle.java')],capture_output=True)
    (WORK/'javac.stderr').write_bytes(compiled.stderr)
    if compiled.returncode:raise RuntimeError(compiled.stderr.decode()[-6000:])
    rng=random.Random(0xae2c7a7e)
    bits=lambda n:struct.pack('>f',n).hex()
    rows=[]
    for kind,mode,flags in itertools.product(range(7),range(5),range(8)):
        center=[-2 if flags&1 else 0,24+(flags%3)-1,1]
        radius=[2.,3.5,6.75,8.][flags%4]
        bounds=[[-3,0,-2],[3,63,3]] if flags%3 else [[0,0,-2],[4,63,3]]
        values=[*center,bits(radius),kind,mode,flags&1,(flags>>1)&1,(flags>>2)&1,int(flags%3!=0),*bounds[0],*bounds[1],48,31 if flags&2 else 24,24,flags*7,flags*13,rng.randrange(-2**31,2**31),rng.randrange(-2**31,2**31),flags&1]
        rows.append(' '.join(map(str,values)))
    # Disjoint slices, wider chunk, both config toggles and numerical radius boundaries.
    for radius,kind in itertools.product([0.,1.,float('nan'),float('inf'),-float('inf'),-1.],range(7)):
        rows.append(' '.join(map(str,[0,24,0,bits(radius),kind,2,0,1,0,1,5,0,-3,8,63,3,40,31,24,17,63,123,456,0])))
    for kind in range(7):
        rows.append(' '.join(map(str,[0,24,0,bits(6.5),kind,4,1,1,1,1,-8,0,-8,7,63,7,48,31,22,3,31,123,456,1])))
    data=('\n'.join(rows)+'\n').encode();(WORK/'input.txt').write_bytes(data)
    expected=subprocess.run([str(args.java_home/'bin/java'),'-cp',str(classes)+os.pathsep+cp,'oracle.MeteoriteOracle'],cwd=WORK,input=data,capture_output=True)
    (WORK/'java.stderr').write_bytes(expected.stderr)
    if expected.returncode:raise RuntimeError(expected.stderr.decode()[-6000:])
    actual=subprocess.run(['cargo','run','--quiet','--locked','--example','meteorite_oracle'],cwd=ROOT,input=data,capture_output=True)
    (WORK/'rust.stderr').write_bytes(actual.stderr);actual.check_returncode()
    (WORK/'java.txt').write_bytes(expected.stdout);(WORK/'rust.txt').write_bytes(actual.stdout)
    if expected.stdout!=actual.stdout:
        for i,(a,b) in enumerate(itertools.zip_longest(expected.stdout.splitlines(),actual.stdout.splitlines())):
            if a!=b:
                aa=(a or b'').split(b';');bb=(b or b'').split(b';');j=next((j for j,p in enumerate(itertools.zip_longest(aa,bb)) if p[0]!=p[1]),-1)
                raise AssertionError(f'case {i}: {rows[i]}\ntrace event {j}: Java {aa[max(0,j-2):j+3]} Rust {bb[max(0,j-2):j+3]}')
    sha=lambda b:hashlib.sha256(b).hexdigest()
    report={'passed':True,'cases':len(rows),'events':sum(row.split(b'END:', 1)[0].count(b';') for row in expected.stdout.splitlines()),'writes':expected.stdout.count(b'S:'),'placement_random_calls':sum(expected.stdout.count(s) for s in [b'PI:',b'PF:',b'PD:']),'level_random_calls':expected.stdout.count(b'LF:'),'water_ticks':expected.stdout.count(b'W:'),
      'source_sha256':{name:sha(body.encode()) for name,body in originals.items()},'binding_sha256':sha(java.encode()),
      'rust_sha256':{name:sha((ROOT/name).read_bytes()) for name in ['mixins/ae2/meteorite_placer.rs','mixins/ae2/meteorite_fallout.rs','examples/meteorite_oracle.rs']},
      'minecraft_client_sha256':sha((minecraft_reference.WORK/'client.jar').read_bytes()),'input_sha256':sha(data),'output_sha256':sha(actual.stdout),
      'scope':'Full original placer/putter/fallout/settings constructor/getter program, ordered full-state writes, both RNG streams (separate/aliased), entity discard callbacks and water scheduling; actual MC bounding-box center',
      'limits':['Controlled terrain, canonical state/property/biome/chunk bindings; java.util.Random supplies controlled legacy draws; no native physics/light/neighbor lifecycle or natural worldgen admission', 'Entity query returns one controlled object; bounding-box construction/discard order is checked, not source/native entity collision queries', 'Settings saved-NBT codecs are compiled but deliberately unsupported/not executed; source read-query count is not compared']}
    (ROOT/'docs/meteorite-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))


if __name__=='__main__':main()
