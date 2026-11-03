package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
/** Exact current C14/S23 class4 boundary. No gameplay emulation. */
public final class NativeAttackSkillOracle {
 public static void main(String[] args)throws Exception {
  var s=BattleSpellCatalog.attackSpell(4);
  if(s.actionId()!=18 || s.effectGfx()!=167 || s.mpCost()!=3 || s.maximumRange()!=10 || s.reuseDelayMillis()!=10)throw new AssertionError("attack catalog drift");
  LinkedHashMap<String,byte[]> p=new LinkedHashMap<>();
  byte[] request=new Protocol.Writer().c(20).c(0).c(3).d(2).h(32735).h(32800).bytes();
  if(Protocol.attackSpellRequest(request).skillId()!=4)throw new AssertionError();p.put("request",request);
  p.put("hit",Protocol.attackSpell(18,1,2,9,3,17,167,32729,32797,32735,32800));
  p.put("zero",Protocol.attackSpell(18,1,2,0,3,18,167,32729,32797,32735,32800));
  p.put("miss",Protocol.attackSpell(18,1,0,0,3,19,167,32729,32797,32735,32800));
  p.put("adjacent",Protocol.attackSpell(18,1,2,9,3,20,167,32729,32797,32730,32797));
  p.put("boundary",Protocol.attackSpell(18,0xffffffffL,0xfffffffeL,255,7,0xffffffffL,167,65535,65535,0,0));
  p.put("book",Protocol.addSpells(Set.of(4)));
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");boolean comma=false;
  for(var v:p.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
  System.out.println(out.append("}}"));
 }
}
