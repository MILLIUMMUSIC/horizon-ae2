#!/usr/bin/env python3
"""Compare original Inscriber callback methods with the Rust state machine.

Inventory, recipe lookup, grid availability, energy interfaces and notifications
are controlled bindings. This is not a Java game or a native world/menu test.
"""
import argparse
import hashlib
import json
from pathlib import Path
import random
import struct
import subprocess

import upstream

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / 'target/inscriber-oracle'
SOURCE = upstream.SOURCE / 'src/main/java/appeng/blockentity/misc/InscriberBlockEntity.java'


def method(source, signature):
    start = source.index(signature)
    end = source.index('{', start) + 1
    depth = 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end]


JAVA = r'''
import java.util.*;
import java.util.function.Consumer;
public class Main {
enum TickRateModulation { URGENT, SLOWER, SLEEP }
enum InscriberProcessType { PRESS, INSCRIBE }
enum Actionable { SIMULATE, MODULATE }
enum PowerMultiplier { CONFIG }
enum YesNo { YES, NO }
enum TickRates { Inscriber }
static class TickingRequest { final boolean sleeping; TickingRequest(TickRates ignored,boolean sleeping){this.sleeping=sleeping;} }
static class IGridNode {}
interface IEnergySource { double extractAEPower(double amount, Actionable mode, PowerMultiplier multiplier); }
interface IEnergyService extends IEnergySource {}
static class AEItems { static final Object SPEED_CARD = new Object(); }
static class Settings { static final Object AUTO_EXPORT = new Object(); }
static class ItemStack {
 int count;
 ItemStack(int count) {this.count=count;}
 boolean isEmpty() {return count<=0;}
 ItemStack copy() {return new ItemStack(count);}
}
static class InscriberRecipe {
 final Ref host;
 InscriberRecipe(Ref host) {this.host=host;}
 ItemStack getResultItem() {return new ItemStack(host.result);}
 InscriberProcessType getProcessType() {return host.press?InscriberProcessType.PRESS:InscriberProcessType.INSCRIBE;}
}
static class Ref implements IEnergySource {
 int processingTime,finalStep,cards,result; boolean smash,task,same,press,ready,export,pushed;
 int[] counts = new int[4]; double local,grid;
 StringBuilder trace = new StringBuilder();
 class Inventory {
  final int index;
  Inventory(int index) {this.index=index;}
  ItemStack getStackInSlot(int slot) {return new ItemStack(counts[index+slot]);}
  ItemStack insertItem(int slot,ItemStack stack,boolean simulate) {
   trace.append(simulate?'I':'C');
   int at=index+slot;
   int free=(counts[at]==0||same)?Math.max(0,64-counts[at]):0;
   int accepted=Math.min(stack.count,free);
   if(!simulate)counts[at]+=accepted;
   return new ItemStack(stack.count-accepted);
  }
  ItemStack extractItem(int slot,int amount,boolean simulate) {
   int at=index+slot;trace.append(at==0?'A':at==1?'B':'M');
   int taken=Math.min(amount,counts[at]);if(!simulate)counts[at]-=taken;
   return new ItemStack(taken);
  }
 }
 final Inventory topItemHandler=new Inventory(0),bottomItemHandler=new Inventory(1),sideItemHandler=new Inventory(2);
 class Upgrades {int getInstalledUpgrades(Object ignored) {return cards;}}
 final Upgrades upgrades=new Upgrades();
 class Config {YesNo getSetting(Object ignored) {return export?YesNo.YES:YesNo.NO;}}
 final Config configManager=new Config();
 class Energy implements IEnergyService {
  public double extractAEPower(double amount,Actionable mode,PowerMultiplier ignored) {
   trace.append(mode==Actionable.SIMULATE?'g':'G');
   double taken=Math.min(grid,amount);if(mode==Actionable.MODULATE)grid-=taken;return taken;
  }
 }
 class Grid {IEnergyService getEnergyService() {return new Energy();}}
 class Node {void ifPresent(Consumer<Grid> consumer) {if(ready)consumer.accept(new Grid());}}
 Node getMainNode() {return new Node();}
 public double extractAEPower(double amount,Actionable mode,PowerMultiplier ignored) {
  trace.append(mode==Actionable.SIMULATE?'l':'L');
  double taken=Math.min(local,amount);if(mode==Actionable.MODULATE)local-=taken;return taken;
 }
 boolean isSmash() {return smash;}
 void setSmash(boolean value) {smash=value;}
 int getProcessingTime() {return processingTime;}
 void setProcessingTime(int value) {processingTime=value;}
 int getMaxProcessingTime() {return 200;}
 void markForUpdate() {trace.append('U');}
 void saveChanges() {trace.append('S');}
 InscriberRecipe getTask() {trace.append('T');return task&&counts[2]>0?new InscriberRecipe(this):null;}
 boolean pushOutResult() {trace.append('P');if(pushed&&export&&counts[3]>0){counts[3]=0;return true;}return false;}
 // ORIGINAL_METHODS
}
static boolean b(Scanner in) {return in.nextInt()!=0;}
static double d(Scanner in) {return Double.longBitsToDouble(Long.parseUnsignedLong(in.next()));}
public static void main(String[] args) {
 Scanner in=new Scanner(System.in);int cases=in.nextInt();StringBuilder out=new StringBuilder();
 for(int c=0;c<cases;c++) {
  Ref h=new Ref();h.processingTime=in.nextInt();h.smash=b(in);h.finalStep=in.nextInt();h.task=b(in);
  for(int i=0;i<4;i++)h.counts[i]=in.nextInt();
  h.same=b(in);h.result=in.nextInt();h.press=b(in);h.cards=in.nextInt();h.local=d(in);h.grid=d(in);
  h.ready=b(in);h.export=b(in);h.pushed=b(in);int elapsed=in.nextInt(),callbacks=in.nextInt();
  if(args.length>0)out.append(h.getTickingRequest(new IGridNode()).sleeping?1:0).append(' ').append(h.processingTime).append('\n');
  for(int i=0;i<callbacks;i++) {
   h.trace.setLength(0);var modulation=h.tickingRequest(new IGridNode(),elapsed);
   out.append(h.processingTime).append(' ').append(h.smash?1:0).append(' ').append(h.finalStep);
   for(int n:h.counts)out.append(' ').append(n);
   out.append(' ').append(Long.toUnsignedString(Double.doubleToRawLongBits(h.local)));
   out.append(' ').append(Long.toUnsignedString(Double.doubleToRawLongBits(h.grid)));
   out.append(' ').append(modulation.ordinal()).append(' ').append(h.trace).append('\n');
  }
 }
 System.out.print(out);
}
}
'''


def bits(value):
    return struct.unpack('>Q', struct.pack('>d', value))[0]


def cases():
    rows = []
    for cards in range(7):
        speed = {1: 3, 2: 5, 3: 10, 4: 50}.get(cards, 2)
        cost = speed * 10
        for processing in [0, 199, 200, 201]:
            for local in [0, cost - .010000001, cost - .01, cost - .009999999, cost, 1600]:
                for grid in [0, cost, 10000]:
                    rows.append([processing, 0, 0, 1, 3, 3, 3, 0, 1, 1, 0, cards,
                                 bits(local), bits(grid), 1, 0, 0, 20, 140])
    rng = random.Random(0xAE219218)
    for _ in range(4000):
        rows.append([rng.choice([0, 199, 200, 201, 2**31-1]), rng.randrange(2),
                     rng.choice([0, 6, 7, 8, 14, 15, 16, 2**31-1]), rng.randrange(2),
                     rng.randrange(4), rng.randrange(4), rng.randrange(4), rng.randrange(70),
                     rng.randrange(2), rng.randrange(1, 6), rng.randrange(2), rng.randrange(7),
                     bits(rng.choice([0, 19.99, 20, 100, 1599.99, 1600, float("nan"), float("inf")])),
                     bits(rng.choice([0, 19.99, 20, 500, 10000, float("nan"), float("inf")])),
                     rng.randrange(2), rng.randrange(2), rng.randrange(2), rng.randrange(1, 1000),
                     rng.randrange(1, 30)])
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home', type=Path, default=Path('/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home'))
    args = parser.parse_args()
    upstream.verify(upstream.SOURCE)
    WORK.mkdir(parents=True, exist_ok=True)
    source = SOURCE.read_text()
    methods = [method(source, signature) for signature in [
        'public TickRateModulation tickingRequest(',
        'private boolean hasCraftWork()',
        'private boolean hasAutoExportWork()',
        'public TickingRequest getTickingRequest(',
    ]]
    java = JAVA.replace('// ORIGINAL_METHODS', '\n'.join(methods))
    (WORK / 'Main.java').write_text(java)
    subprocess.run([str(args.java_home / 'bin/javac'), '-d', str(WORK), str(WORK / 'Main.java')], check=True)
    rows = cases()
    data = (str(len(rows)) + '\n' + '\n'.join(' '.join(map(str, row)) for row in rows) + '\n').encode()
    (WORK / 'input.txt').write_bytes(data)
    combined = b''
    for mode in [[], ['--initial']]:
        expected = subprocess.run([str(args.java_home / 'bin/java'), '-cp', str(WORK), 'Main', *mode], input=data, capture_output=True, check=True, cwd=WORK).stdout
        actual = subprocess.run(['cargo', 'run', '--quiet', '--locked', '--example', 'inscriber_oracle', '--', *mode], cwd=ROOT, input=data, capture_output=True, check=True).stdout
        suffix = '-initial' if mode else ''
        (WORK / f'java{suffix}.txt').write_bytes(expected)
        (WORK / f'rust{suffix}.txt').write_bytes(actual)
        if expected != actual:
            for n, (a, b) in enumerate(zip(expected.splitlines(), actual.splitlines())):
                if a != b:
                    raise AssertionError(f'callback {n} {mode}: Java {a!r}, Rust {b!r}; see {WORK}')
            raise AssertionError(f'callback count mismatch; see {WORK}')
        combined += actual
    sha = lambda value: hashlib.sha256(value).hexdigest()
    report = {'passed': True, 'cases': len(rows), 'callbacks': 2 * sum(row[-1] for row in rows), 'initial_requests': len(rows),
              'source': str(SOURCE.relative_to(upstream.SOURCE)), 'source_sha256': sha(SOURCE.read_bytes()),
              'method_sha256': [sha(m.encode()) for m in methods], 'binding_sha256': sha(java.encode()),
              'rust_sha256': sha((ROOT / 'mixins/ae2/inscriber.rs').read_bytes()),
              'driver_sha256': sha((ROOT / 'examples/inscriber_oracle.rs').read_bytes()),
              'input_sha256': sha(data), 'output_sha256': sha(combined),
              'scope': 'verbatim tickingRequest, hasCraftWork, hasAutoExportWork and getTickingRequest methods; state and callback order',
              'limits': ['controlled inventories, recipes, energy interfaces and notifications',
                         'pushOutResult uses a controlled result; external inventories are not executed',
                         'native TickManager scheduling, configuration, menus, rendering and world lifecycle are separate']}
    (ROOT / 'docs/inscriber-oracle.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
