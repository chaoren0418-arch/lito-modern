package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
/** Current deployed ground selector and location-effect vectors. */
public final class NativeGroundSkillOracle {
 public static void main(String[] args)throws Exception {
  var s=BattleSpellCatalog.groundSpell(58);
  if(s.row()!=7 || s.column()!=1 || s.actionId()!=18 || s.effectGfx()!=168 || s.mpCost()!=60 || s.maximumRange()!=8 || s.reuseDelayMillis()!=10 || s.durationSeconds()!=10 || s.segmentCount()!=8 || s.pulseIntervalMillis()!=1650 || s.effectRadius()!=1 || s.itemConsumeId()!=0)throw new AssertionError("ground catalog drift");
  LinkedHashMap<String,byte[]> p=new LinkedHashMap<>();
  byte[] request=new Protocol.Writer().c(20).c(7).c(1).h(32735).h(32800).bytes();
  var r=Protocol.groundSpellRequest(request);
  if(r.skillId()!=58 || r.x()!=32735 || r.y()!=32800)throw new AssertionError();
  p.put("request",request);
  p.put("boundary",new Protocol.Writer().c(20).c(7).c(1).h(65535).h(0).bytes());
  p.put("cast",new Protocol.Writer().c(Protocol.S_ACTION).d(1).c(18).bytes());
  p.put("direction",new Protocol.Writer().c(Protocol.S_CHANGE_DIRECTION).d(1).c(3).bytes());
  p.put("effect",Protocol.effectLocation(32735,32800,168));
  p.put("book",Protocol.addSpells(Set.of(58)));
  p.put("mana",Protocol.manaPoint(60,120));
  var effects=new BattleGroundEffects();
  var placed=effects.cast(s,1,32729,32797,32730,32797,ArenaMapCatalog.forMap(564),0);
  if(!placed.accepted() || placed.points().isEmpty())throw new AssertionError("ground placement");
  var first=placed.points().get(0);
  var actors=List.of(new BattleGroundEffects.Actor(2,first.x(),first.y(),true,true));
  if(!effects.advance(1_649_999_999L,actors).isEmpty())throw new AssertionError("early pulse");
  int firstPulses=effects.advance(1_650_000_000L,actors).size();
  if(firstPulses==0 || effects.activeCount()!=placed.points().size())throw new AssertionError("pulse absent");
  int latePulses=effects.advance(9_999_999_999L,actors).size();
  if(latePulses==0 || !effects.advance(10_000_000_000L,actors).isEmpty() || effects.activeCount()!=0)throw new AssertionError("lease expiration");
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");boolean comma=false;
  for(var v:p.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
  System.out.println(out.append("},\"lease\":{\"segments\":").append(placed.points().size()).append(",\"first_pulses\":").append(firstPulses).append(",\"late_pulses\":").append(latePulses).append(",\"expires_ns\":10000000000}}"));
 }
}
