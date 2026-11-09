#!/usr/bin/env python3
"""Compare primary multipart topology with original CableBus/Reporting methods.

Part registry/admission, managed-node events, grid merge/pathing, level access
and transport infrastructure are controlled bindings. No menu/world/render or
complete source lifecycle is executed by this harness.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

import upstream
import cable_oracle
from inscriber_oracle import method

ROOT=Path(__file__).resolve().parents[1]
WORK=ROOT/'target/multipart-oracle'
BASE=upstream.SOURCE/'src/main/java/appeng'
JAVA=r'''
package oracle;
import java.io.*;
import java.util.*;
import java.util.concurrent.atomic.AtomicBoolean;
import net.minecraft.core.Direction;
import appeng.api.util.AECableType;
public class MultipartOracle {
 static class BlockEntity {}
 interface IPartItem<T> {}
 enum BusSupport {CABLE,DENSE_CABLE}
 static class GridFlags {static final int REQUIRE_CHANNEL=1;}
 static class IGridNode extends CableOracle.Node {int label;IGridNode(int label){this.label=label;max=8;mode=CableOracle.ChannelMode.DEFAULT;} }
 static class Managed {
  int flags;double idle;Managed setFlags(int flags){this.flags=flags;return this;}
  Managed setIdlePowerUsage(double idle){this.idle=idle;return this;}
 }
 interface IPart {
  IGridNode getGridNode();default IGridNode getExternalFacingNode(){return null;}
  void setPartHostInfo(Direction side,Host host,BlockEntity be);
  void addToWorld();void removeFromWorld();
 }
 interface ICablePart extends IPart {void setExposedOnSides(EnumSet<Direction> sides);}
 static List<Integer> added=new ArrayList<>();static Set<String> edges=new TreeSet<>();
 static class Part extends CableOracle.Part implements IPart {
  IGridNode owned;boolean ready;Managed main=new Managed();
  Part(IPartItem<?> ignored){owned=new IGridNode(-1);node=owned;}
  Managed getMainNode(){return main;}
  public IGridNode getGridNode(){return ready?owned:null;}
  public void setPartHostInfo(Direction side,Host host,BlockEntity be){owned.label=side==null?0:side.ordinal()+1;}
  public void addToWorld(){ready=true;added.add(owned.label);}
  public void removeFromWorld(){ready=false;}
 }
 static class AbstractReportingPart extends Part {
  // REPORTING_CONSTRUCTOR
 }
 static class Terminal extends AbstractReportingPart {Terminal(){super(null,true);} }
 static class Cable extends Part implements ICablePart {
  EnumSet<Direction> exposed=EnumSet.allOf(Direction.class);
  Cable(){super(null);}
  public void setExposedOnSides(EnumSet<Direction> sides){exposed=sides;}
 }
 static class Storage {Cable center;IPart[] parts=new IPart[6];Cable getCenter(){return center;} }
 static class Tcb {BlockEntity be=new BlockEntity();BlockEntity getBlockEntity(){return be;} }
 static class Platform {static final Direction[] DIRECTIONS_WITH_NULL={Direction.DOWN,Direction.UP,Direction.NORTH,Direction.SOUTH,Direction.WEST,Direction.EAST,null};}
 static class GridHelper {
  static void createConnection(IGridNode a,IGridNode b){
   if(a==b)throw new AssertionError();String key=Math.min(a.label,b.label)+":"+Math.max(a.label,b.label);
   if(!edges.add(key))throw new AssertionError("duplicate internal edge");
   a.connections.add(new CableOracle.Connection(0));b.connections.add(new CableOracle.Connection(0));
  }
 }
 static class Host {
  static AtomicBoolean IS_LOADING=new AtomicBoolean();boolean inWorld;Storage storage=new Storage();Tcb tcb=new Tcb();int blocked;
  IPart getPart(Direction side){return side==null?storage.center:storage.parts[side.ordinal()];}
  BlockEntity getBlockEntity(){return tcb.be;}boolean isBlocked(Direction side){return (blocked&(1<<side.ordinal()))!=0;}
  void partChanged(){updateConnections();}void invalidateShapes(){}
  // HOST_METHODS
 }
 public static void main(String[] args)throws Exception {
  var reader=new BufferedReader(new InputStreamReader(System.in));String line;StringBuilder output=new StringBuilder();
  while((line=reader.readLine())!=null){
   var in=new StringTokenizer(line);int kind=Integer.parseInt(in.nextToken()),parts=Integer.parseInt(in.nextToken()),blocked=Integer.parseInt(in.nextToken()),neighbors=Integer.parseInt(in.nextToken());
   added.clear();edges.clear();Host host=new Host();host.blocked=blocked;if(kind!=0)host.storage.center=new Cable();
   Terminal profile=new Terminal();
   for(var side:Direction.values())if((parts&(1<<side.ordinal()))!=0)host.storage.parts[side.ordinal()]=new Terminal();
   host.addToWorld();int count=added.size();host.addToWorld();if(added.size()!=count)throw new AssertionError("double addToWorld");
   int internal=0;for(var side:Direction.values())if(edges.contains("0:"+(side.ordinal()+1)))internal|=1<<side.ordinal();
   int external=0;if(host.storage.center!=null)for(var side:Direction.values())if((neighbors&(1<<side.ordinal()))!=0 && host.storage.center.exposed.contains(side) && host.getGridNode(side)!=null)external|=1<<side.ordinal();
   // Existing pathing has its own original-Java oracle. Here finalized counts
   // are a controlled binding for this controllerless star's source ad-hoc mode.
   int demand=Integer.bitCount(internal)+Integer.bitCount(external),used=demand>8?0:demand;
   var cable=CableOracle.cable(kind);cable.node=host.storage.center==null?null:host.storage.center.owned;
   if(cable.node!=null)cable.node.max=AECableType.values()[kind].isDense()?32:8;
   for(var side:Direction.values()){
    int bit=1<<side.ordinal();Part p=(Part)host.storage.parts[side.ordinal()];cable.host.parts[side.ordinal()]=p;
    if(p!=null)p.owned.connections.replaceAll(x->new CableOracle.Connection(used));
    if(cable.node!=null&&(external&bit)!=0)cable.node.external.put(side,new CableOracle.Connection(used));
   }
   var packet=new CableOracle.RegistryFriendlyByteBuf();cable.writeToStream(packet);cable.readFromStream(new CableOracle.RegistryFriendlyByteBuf(packet.out.toByteArray()));
   output.append(count).append(' ').append(internal).append(' ').append(external).append(' ').append(profile.main.flags).append(' ').append((int)(profile.main.idle*16)).append(' ').append(CableOracle.mask(cable.connections));
   for(int ch:cable.channelsOnSide)output.append(' ').append(ch);
   cable.powered=false;for(var side:Direction.values())output.append(' ').append(cable.getChannelsOnSide(side));
   output.append('\n');host.removeFromWorld();
   for(var side:Platform.DIRECTIONS_WITH_NULL){var part=host.getPart(side);if(part!=null&&part.getGridNode()!=null)throw new AssertionError("removeFromWorld left a node");}
  }
  System.out.print(output);
 }
}
'''


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home',type=Path,default=Path('/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home'))
    args=parser.parse_args();upstream.verify(upstream.SOURCE);WORK.mkdir(parents=True,exist_ok=True)
    cable_oracle.WORK=WORK;src,wire_methods,source_files=cable_oracle.generate()
    container=(BASE/'parts/CableBusContainer.java').read_text()
    host_methods=[method(container,s) for s in ['public void updateConnections()','public void addToWorld()','public void removeFromWorld()','public IGridNode getGridNode(']]
    reporting=(BASE/'parts/reporting/AbstractReportingPart.java').read_text()
    constructor=method(reporting,'protected AbstractReportingPart(')
    java=JAVA.replace('// HOST_METHODS','\n'.join(host_methods)).replace('// REPORTING_CONSTRUCTOR',constructor)
    (src/'MultipartOracle.java').write_text(java)
    subprocess.run([str(args.java_home/'bin/javac'),'-d',str(WORK/'classes'),*map(str,src.rglob('*.java'))],check=True)
    rows=[]
    for kind in range(6):
        for parts in (range(64) if kind<4 else [0]):
            for blocked in range(64):
                for neighbors in [0,63,21,42]:rows.append(f'{kind} {parts} {blocked} {neighbors}')
    data=('\n'.join(rows)+'\n').encode();(WORK/'input.txt').write_bytes(data)
    expected=subprocess.run([str(args.java_home/'bin/java'),'-cp',str(WORK/'classes'),'oracle.MultipartOracle'],input=data,capture_output=True,check=True,cwd=WORK).stdout
    run=subprocess.run(['cargo','run','--quiet','--locked','--example','multipart_oracle'],input=data,capture_output=True,cwd=ROOT)
    (WORK/'rust.stderr').write_bytes(run.stderr);run.check_returncode();actual=run.stdout
    (WORK/'java.txt').write_bytes(expected);(WORK/'rust.txt').write_bytes(actual)
    if expected!=actual:
        for i,(a,b) in enumerate(zip(expected.splitlines(),actual.splitlines())):
            if a!=b:raise AssertionError(f'case {i} ({rows[i]}): Java {a!r}, Rust {b!r}; see {WORK}')
        raise AssertionError('case count mismatch')
    sha=lambda b:hashlib.sha256(b).hexdigest()
    source_files+=['parts/CableBusContainer.java','parts/reporting/AbstractReportingPart.java']
    report={'passed':True,'cases':len(rows),'method_sha256':[sha(m.encode()) for m in host_methods+[constructor]],
      'source_sha256':{name:sha((BASE/name).read_bytes()) for name in source_files},
      'wire_method_sha256':{name:sha(wire_methods[name].encode()) for name in ['write','read','visual','channels']},
      'binding_sha256':sha(java.encode()),'rust_sha256':sha((ROOT/'mixins/ae2/network.rs').read_bytes()),
      'reporting_sha256':sha((ROOT/'mixins/ae2/reporting.rs').read_bytes()),
      'driver_sha256':sha((ROOT/'examples/multipart_oracle.rs').read_bytes()),'input_sha256':sha(data),'output_sha256':sha(actual),
      'scope':'Verbatim CableBus updateConnections/addToWorld/removeFromWorld/getGridNode, Reporting constructor and CablePart write/read/visual/channel methods; primary slot topology and wire/render distinction',
      'limits':['Controlled part storage/registry, node lifecycle callbacks, grid helper, finalized ad-hoc counts, level and packet bindings',
                'Grid merge/pathing algorithms are covered by separate existing oracles; no full lifecycle/order equivalence is claimed here',
                'Only valid primary-node host configurations; source admission/recovery, external tunnel/fiber nodes, menus, shapes, rendering and native world activation remain separate']}
    (ROOT/'docs/multipart-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))


if __name__=='__main__':main()
