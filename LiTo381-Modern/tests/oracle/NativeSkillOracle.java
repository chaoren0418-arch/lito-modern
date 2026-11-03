package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
/** Fixed deployed JAR skill selectors, book and effects; no gameplay emulation. */
public final class NativeSkillOracle {
 public static void main(String[] args)throws Exception {
  LinkedHashMap<String,byte[]> p=new LinkedHashMap<>();
  var healing=BattleSpellCatalog.healingSpell(1);var protection=BattleSpellCatalog.shieldSpell(3);
  if(healing.actionId()!=19 || healing.mpCost()!=4 || healing.reuseDelayMillis()!=10
      || protection.actionId()!=19 || protection.mpCost()!=8 || protection.reuseDelayMillis()!=10)throw new AssertionError("skill catalog changed");
  byte[] heal=new Protocol.Writer().c(20).c(0).c(0).d(1).bytes();
  if(Protocol.objectSpellRequest(heal).skillId()!=1)throw new AssertionError();p.put("heal",heal);
  byte[] shield=new Protocol.Writer().c(20).c(0).c(2).bytes();
  if(Protocol.selectorOnlySpellRequest(shield).skillId()!=3)throw new AssertionError();p.put("shield",shield);
  byte[] bolt=new Protocol.Writer().c(20).c(0).c(3).d(2).h(32741).h(32801).bytes();
  if(Protocol.attackSpellRequest(bolt).skillId()!=4)throw new AssertionError();p.put("bolt",bolt);
  p.put("add",Protocol.addSpells(Set.of(1,3,4,43,220)));
  p.put("remove",Protocol.removeSpells(Set.of(3,220)));
  p.put("empty",Protocol.addSpells(Set.of()));
  p.put("heal_effect",Protocol.effectObject(1,11722));
  p.put("shield_effect",Protocol.effectObject(1,221));
  p.put("location_effect",Protocol.effectLocation(32729,32797,221));
  p.put("effect_boundary",Protocol.effectObject(0xffffffffL,65535));
  p.put("mana",Protocol.manaPoint(6,10));
  p.put("no_mana",Protocol.messageCode(278));
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");boolean comma=false;
  for(var v:p.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
  System.out.println(out.append("}}"));
 }
}
