package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
/** Current deployed Protocol vectors, independent of historical Modern contracts. */
public final class NativeRangedOracle {
 public static void main(String[] args)throws Exception {
  LinkedHashMap<String,byte[]> p=new LinkedHashMap<>();
  byte[] request=new Protocol.Writer().c(Protocol.C_FAR_ATTACK).d(2).h(32741).h(32801).bytes();
  Protocol.attackRequest(request,Protocol.C_FAR_ATTACK);p.put("request",request);
  p.put("hit",Protocol.rangedPhysicalAttack(21,1,2,5,3,17,66,32729,32797,32741,32801));
  p.put("hit_zero",Protocol.rangedPhysicalAttack(21,1,2,0,3,19,66,32729,32797,32741,32801));
  p.put("miss",Protocol.rangedPhysicalAttack(21,1,0,0,3,18,66,32729,32797,32741,32801));
  p.put("boundary",Protocol.rangedPhysicalAttack(63,0xffffffffL,2,255,7,0xffffffffL,65535,0,65535,65535,0));
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");boolean comma=false;
  for(var v:p.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
  System.out.println(out.append("}}"));
 }
}
