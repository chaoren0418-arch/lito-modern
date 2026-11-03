package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
/** Formal placement, not injected gameplay damage. Captured miss plus both spawn neighborhoods. */
public final class NativeWallPlacementOracle {
 public static void main(String[] args)throws Exception {
  var spell=BattleSpellCatalog.groundSpell(58);var map=ArenaMapCatalog.forMap(564);
  var miss=new BattleGroundEffects().cast(spell,1,32735,32795,32729,32797,map,0);
  if(!miss.accepted() || miss.points().stream().anyMatch(p->Math.max(Math.abs(p.x()-32729),Math.abs(p.y()-32797))<=1))throw new AssertionError("captured miss changed");
  var rows=new ArrayList<String>();
  for(int[] target:new int[][]{{32729,32797},{32741,32801}})for(int dx=-1;dx<=1;dx++)for(int dy=-1;dy<=1;dy++) {
   if(dx==0&&dy==0)continue;
   int x=target[0]+dx,y=target[1]+dy;
   var effects=new BattleGroundEffects();var cast=effects.cast(spell,1,x,y,target[0],target[1],map,0);
   if(!cast.accepted() || cast.points().stream().noneMatch(p->p.x()==target[0]&&p.y()==target[1]))throw new AssertionError("adjacent target absent: "+x+","+y);
   var hits=effects.advance(1_650_000_000L,List.of(new BattleGroundEffects.Actor(3,target[0],target[1],true,true)));
   if(hits.isEmpty())throw new AssertionError("adjacent pulse absent");
   rows.add("{\"caster\":["+x+","+y+"],\"target\":["+target[0]+","+target[1]+"],\"segments\":"+cast.points().size()+",\"pulses\":"+hits.size()+"}");
  }
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  System.out.println("{\"jar_sha256\":\""+sha+"\",\"captured_miss_confirmed\":true,\"adjacent_cases\":["+String.join(",",rows)+"],\"live_gameplay_verified\":false}");
 }
}
