#!/usr/bin/env python3
"""Execute original transform matching and server clock methods with controlled bindings.

Entity query/recipe iteration order, fluid lookup, inventories, assembly and RNG
are bound by this harness. Native movement, world iteration and source effects
are separate checks. Identity-map iteration order is not compared.
"""
import argparse
import hashlib
import json
from pathlib import Path
import random
import subprocess

import upstream
from inscriber_oracle import method

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / 'target/transform-oracle'
JAVA = r'''
import java.util.*;
import java.util.function.*;
public class Main {
 static class CallbackInfo {}
 static class BlockPos { BlockPos(int x,int y,int z) {} }
 static class Mth { static int floor(double d){return (int)Math.floor(d);} }
 static class AABB {double minY=0,maxY=.25;AABB(double... ignored) {} }
 static class FluidState {boolean valid;boolean isEmpty(){return !valid;} }
 static class TransformCircumstance {boolean isExplosion(){return false;} boolean isFluid(FluidState s){return s.valid;} }
 static class Ingredient {int mask;Ingredient(int mask){this.mask=mask;}boolean test(ItemStack s){return s.count>0&&(mask&(1<<s.kind))!=0;} }
 static class ItemStack {
  int kind,count;ItemStack(int kind,int count){this.kind=kind;this.count=count;}
  boolean isEmpty(){return count<=0;}
  int getCount(){return count;}
  ItemStack split(int n){int take=Math.min(n,count);count-=take;return new ItemStack(kind,take);}
 }
 static class ItemEntity {
  Level level;ItemStack stack;boolean removed;int pickupDelay;
  ItemEntity(Level level,double x,double y,double z,ItemStack stack){this.level=level;this.stack=stack;}
  Level level(){return level;}ItemStack getItem(){return stack;}
  double getX(){return 0;}double getY(){return 0;}double getZ(){return 0;}
  AABB getBoundingBox(){return new AABB();}boolean isRemoved(){return removed;}
  void discard(){removed=true;}void setDeltaMovement(double x,double y,double z){}
 }
 static class TransformRecipeInput {TransformRecipeInput(List<ItemStack> ignored){} }
 static class Recipe {
  List<Ingredient> ingredients=new ArrayList<>();TransformCircumstance circumstance=new TransformCircumstance();
  ItemStack assemble(TransformRecipeInput ignored,Object registries){return new ItemStack(7,2);}
 }
 static class Holder {Recipe recipe;Holder(Recipe recipe){this.recipe=recipe;}Recipe value(){return recipe;} }
 static class TransformRecipe {static final Object TYPE=new Object();}
 static class Manager {List<Holder> recipes=new ArrayList<>();List<Holder> byType(Object type){return recipes;} }
 static class RandomBinding {int draws;double nextDouble(){draws++;return .5;} }
 static class Level {
  boolean wet,isClientSide=false;List<ItemEntity> entities=new ArrayList<>();Manager manager=new Manager();
  RandomBinding random=new RandomBinding();int products;
  List<ItemEntity> getEntities(Object ignored,AABB region){return entities;}
  Manager getRecipeManager(){return manager;}RandomBinding getRandom(){return random;}
  Object registryAccess(){return null;}boolean isClientSide(){return false;}
  FluidState getFluidState(BlockPos pos){FluidState s=new FluidState();s.valid=wet;return s;}
  void addFreshEntity(ItemEntity item){if(item.pickupDelay!=0)throw new AssertionError();products++;}
 }
 static class Lists {static <T> ArrayList<T> newArrayList(Collection<T> values){return new ArrayList<>(values);} }
 interface Reference2IntMap<K> {
  int size();int getInt(K key);void merge(K key,int n,BinaryOperator<Integer> f);
  List<IntEntry<K>> reference2IntEntrySet();
 }
 record IntEntry<K>(K key,int value){K getKey(){return key;}int getIntValue(){return value;} }
 static class Reference2IntOpenHashMap<K> implements Reference2IntMap<K> {
  Map<K,Integer> map=new LinkedHashMap<>();Reference2IntOpenHashMap(int size){}
  public int size(){return map.size();}
  public int getInt(K key){return map.getOrDefault(key,0);}
  public void merge(K key,int n,BinaryOperator<Integer> f){map.merge(key,n,f);}
  public List<IntEntry<K>> reference2IntEntrySet(){return map.entrySet().stream().map(e->new IntEntry<K>(e.getKey(),e.getValue())).toList();}
 }
 static class TransformLogic {
  static boolean canTransformInAnyFluid(ItemEntity e){return e.level.manager.recipes.stream().anyMatch(h->!h.recipe.ingredients.isEmpty()&&h.recipe.ingredients.getFirst().test(e.stack));}
  static boolean canTransformInFluid(ItemEntity e,FluidState s){return s.valid&&canTransformInAnyFluid(e);}
  // ORIGINAL_TRANSFORM
 }
 static class AEConfig {static AEConfig instance(){return new AEConfig();}boolean isEnableEffects(){return false;} }
 static class EffectType {static Object Lightning=new Object();}
 static class AppEng {static AppEng instance(){return new AppEng();}void spawnEffect(Object... args){} }
 static class Ref extends ItemEntity {
  int ae2_transformTime,ae2_delay;
  Ref(Level level,ItemStack stack){super(level,0,0,0,stack);}
  // ORIGINAL_CLOCK
 }
 public static void main(String[] args){
  Scanner in=new Scanner(System.in);int cases=in.nextInt();StringBuilder out=new StringBuilder();
  for(int c=0;c<cases;c++){
   Level level=new Level();int clock=in.nextInt(),trigger=in.nextInt(),n=in.nextInt();Ref ref=null;
   for(int i=0;i<n;i++){
    ItemStack stack=new ItemStack(in.nextInt(),in.nextInt());boolean alive=in.nextInt()!=0;
    ItemEntity entity;if(i==trigger){ref=new Ref(level,stack);ref.ae2_transformTime=clock;entity=ref;}else entity=new ItemEntity(level,0,0,0,stack);
    entity.removed=!alive;level.entities.add(entity);
   }
   int nr=in.nextInt();for(int r=0;r<nr;r++){Recipe recipe=new Recipe();int ni=in.nextInt();for(int i=0;i<ni;i++)recipe.ingredients.add(new Ingredient(in.nextInt()));level.manager.recipes.add(new Holder(recipe));}
   int frames=in.nextInt();for(int f=0;f<frames;f++){
    level.wet=in.nextInt()!=0;ref.handleEntityTransform(new CallbackInfo());
    out.append(ref.ae2_transformTime).append(' ').append(level.products).append(' ').append(level.random.draws);
    for(var entity:level.entities)out.append(' ').append(entity.stack.count).append(' ').append(entity.removed?0:1);
    out.append('\n');
   }
  }
  System.out.print(out);
 }
}
'''


def cases():
    rng = random.Random(0xAE219)
    rows = []
    for clock in [0, 59, 60, 61, 2147483646, 2147483647, -2147483648]:
        for alive in [0, 1]:
            rows.append([clock, 0, 3, 0, 3, alive, 1, 3, 1, 2, 3, 1, 1, 3, 1, 2, 4, 100, *([1]*100)])
    for _ in range(5000):
        n = rng.randrange(1, 9); trigger = rng.randrange(n)
        row = [rng.choice([0,59,60,61,2147483647,-2147483648,rng.randrange(-100,100)]), trigger, n]
        for i in range(n): row.extend([rng.randrange(6), rng.choice([0,1,2,3,64]), int(rng.random()>.1)])
        nr=rng.randrange(1,5);row.append(nr)
        for _ in range(nr):
            ni=rng.randrange(0,6);row.append(ni)
            row.extend(rng.randrange(1,64) for _ in range(ni))
        frames=rng.randrange(1,85);row.append(frames)
        row.extend(int(rng.random()>.05) for _ in range(frames))
        rows.append(row)
    return rows


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home',type=Path,default=Path('/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home'))
    args=parser.parse_args();upstream.verify(upstream.SOURCE);WORK.mkdir(parents=True,exist_ok=True)
    source_paths=['src/main/java/appeng/mixins/ItemEntityMixin.java','src/main/java/appeng/recipes/transform/TransformLogic.java']
    sources=[(upstream.SOURCE / p).read_text() for p in source_paths]
    clock=method(sources[0],'void handleEntityTransform(')
    transform=method(sources[1],'public static boolean tryTransform(')
    java=JAVA.replace('// ORIGINAL_CLOCK',clock).replace('// ORIGINAL_TRANSFORM',transform)
    (WORK/'Main.java').write_text(java)
    subprocess.run([str(args.java_home/'bin/javac'),'-d',str(WORK),str(WORK/'Main.java')],check=True)
    rows=cases();data=(str(len(rows))+'\n'+'\n'.join(' '.join(map(str,row)) for row in rows)+'\n').encode()
    (WORK/'input.txt').write_bytes(data)
    expected=subprocess.run([str(args.java_home/'bin/java'),'-cp',str(WORK),'Main'],input=data,capture_output=True,check=True,cwd=WORK).stdout
    actual=subprocess.run(['cargo','run','--quiet','--locked','--example','transform_oracle'],input=data,capture_output=True,check=True,cwd=ROOT).stdout
    (WORK/'java.txt').write_bytes(expected);(WORK/'rust.txt').write_bytes(actual)
    if expected!=actual:
        for n,(a,b) in enumerate(zip(expected.splitlines(),actual.splitlines())):
            if a!=b:raise AssertionError(f'callback {n}: Java={a!r}, Rust={b!r}; see {WORK}')
        raise AssertionError('callback count differs')
    sha=lambda value:hashlib.sha256(value).hexdigest()
    report={'passed':True,'cases':len(rows),'callbacks':len(actual.splitlines()),
      'source_paths':source_paths,'source_sha256':[sha(s.encode()) for s in sources],
      'method_sha256':[sha(clock.encode()),sha(transform.encode())],'binding_sha256':sha(java.encode()),
      'rust_sha256':sha((ROOT/'mixins/ae2/transform_throw.rs').read_bytes()),
      'driver_sha256':sha((ROOT/'examples/transform_oracle.rs').read_bytes()),
      'input_sha256':sha(data),'output_sha256':sha(actual),
      'scope':'Verbatim server handleEntityTransform and tryTransform control flow: counters, greedy claims, discard and six RNG draws',
      'limits':['Controlled fluid/cache/query/recipe order, Ingredient tests, stacks, assembly and RNG',
                'Identity-map iteration order, native physics, lifecycle, packet transport and source client effects are not compared']}
    (ROOT/'docs/transform-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))


if __name__=='__main__':main()
