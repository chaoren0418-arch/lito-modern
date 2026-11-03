package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
/** Read-only deployed catalog/protocol/lease oracle; no SQL or production process. */
public final class NativeReagentSkillOracle {
 public static void main(String[] args)throws Exception {
  var s=BattleSpellCatalog.groundSpell(63);
  if(s.row()!=7 || s.column()!=6 || s.actionId()!=19 || s.effectGfx()!=2231 || s.mpCost()!=20 || s.maximumRange()!=-1 || s.reuseDelayMillis()!=10 || s.durationSeconds()!=320 || s.segmentCount()!=1 || s.pulseIntervalMillis()!=0 || s.effectRadius()!=3 || s.itemConsumeId()!=40318 || s.itemConsumeCount()!=1)throw new AssertionError("reagent catalog drift");
  LinkedHashMap<String,byte[]> p=new LinkedHashMap<>();
  byte[] request=new Protocol.Writer().c(20).c(7).c(6).h(32735).h(32800).bytes();
  var r=Protocol.groundSpellRequest(request);
  if(r.skillId()!=63 || r.x()!=32735 || r.y()!=32800)throw new AssertionError();
  for(int n=0;n<request.length;n++) {
   boolean refused=false;try{Protocol.groundSpellRequest(Arrays.copyOf(request,n));}catch(IllegalArgumentException e){refused=true;}
   if(!refused)throw new AssertionError("truncated request accepted "+n);
  }
  boolean trailing=false;try{Protocol.groundSpellRequest(Arrays.copyOf(request,8));}catch(IllegalArgumentException e){trailing=true;}
  if(!trailing)throw new AssertionError("trailing byte accepted");
  p.put("request",request);
  p.put("self",new Protocol.Writer().c(20).c(7).c(6).h(32729).h(32797).bytes());
  p.put("boundary",new Protocol.Writer().c(20).c(7).c(6).h(65535).h(0).bytes());
  p.put("cast",new Protocol.Writer().c(Protocol.S_ACTION).d(1).c(19).bytes());
  p.put("effect",Protocol.effectLocation(32735,32800,2231));
  p.put("book",Protocol.addSpells(Set.of(63)));
  p.put("mana",Protocol.manaPoint(60,80));
  var effects=new BattleGroundEffects();var arena=ArenaMapCatalog.forMap(564);
  for(int n=0;n<2;n++){
   var placed=effects.cast(s,1,32729,32797,32729,32797,arena,0);
   if(!placed.accepted() || placed.points().size()!=1 || placed.points().get(0).x()!=32729 || placed.points().get(0).y()!=32797)throw new AssertionError("own-cell/overlap placement");
  }
  var actors=List.of(new BattleGroundEffects.Actor(2,32729,32797,true,true));
  if(effects.activeCount()!=2 || !effects.advance(319_999_999_999L,actors).isEmpty() || effects.activeCount()!=2 || !effects.advance(320_000_000_000L,actors).isEmpty() || effects.activeCount()!=0)throw new AssertionError("lease/no-damage authority");
  StringBuilder screen=new StringBuilder();
  for(int dx=-20;dx<=20;dx++)for(int dy=-20;dy<=20;dy++)screen.append(BattleSpellCatalog.isInNativeScreen(32729,32797,32729+dx,32797+dy)?'1':'0');
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");boolean comma=false;
  for(var v:p.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
  System.out.println(out.append("},\"screen_dx_dy_minus20_through20\":\"").append(screen).append("\",\"lease\":{\"overlapping\":2,\"damage_pulses\":0,\"expires_ns\":320000000000}}"));
 }
}
