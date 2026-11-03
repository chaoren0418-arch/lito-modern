package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
public final class NativeCombatOracle {
 public static void main(String[] args)throws Exception {
  LinkedHashMap<String,byte[]> p=new LinkedHashMap<>();
  byte[] request=new Protocol.Writer().c(Protocol.C_ATTACK).d(2).h(32741).h(32801).bytes();
  Protocol.attackRequest(request, Protocol.C_ATTACK);
  p.put("request",request);
  p.put("hit",Protocol.physicalAttack(1,1,2,true,7));
  p.put("miss",Protocol.physicalAttack(1,1,2,false,0));
  p.put("counter",Protocol.physicalDamageAttack(1,2,1,5,0));
  p.put("clamped",Protocol.physicalDamageAttack(1,1,2,9999,0));
  p.put("death",Protocol.objectAction(2,8));
  p.put("own_death",Protocol.objectAction(1,8));
  p.put("own_hp_zero",new Protocol.Writer().c(Protocol.S_HIT_POINT).h(0).h(50).bytes());
  byte[] confirm=new Protocol.Writer().c(Protocol.C_LITO_CONFIRM_DEATH).bytes();
  Protocol.litoConfirmDeathRequest(confirm);p.put("confirm_death",confirm);
  p.put("ask_confirm_death",new Protocol.Writer().c(Protocol.S_LITO_CONFIRM_DEATH).bytes());
  p.put("ratio_0",Protocol.hitRatio(2,0,100));
  p.put("ratio_50",Protocol.hitRatio(2,50,100));
  p.put("ratio_100",Protocol.hitRatio(2,100,100));
  for(var result:Protocol.LitoBattleResult.values()) p.put("result_"+result.name(),Protocol.battleResult(result));
  p.put("score",new Protocol.Writer().c(0xdb).s("測試角色").d(10).d(1).d(0).d(2).c(0).c(0).bytes());
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");boolean comma=false;
  for(var v:p.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
  System.out.println(out.append("}}"));
 }
}
