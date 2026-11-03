package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
public final class NativeMotionOracle {
 public static void main(String[] args)throws Exception {
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"moves\":[");
  for(int h=0;h<8;h++){
   byte[] b=new Protocol.Writer().c(Protocol.C_MOVE).h(32720).h(32780).c(h).bytes();
   var move=Protocol.moveRequest(b);
   if(h>0)out.append(',');
   out.append("{\"body\":\"").append(HexFormat.of().formatHex(b)).append("\",\"heading\":").append(h).append(",\"destination\":[").append(move.x()).append(',').append(move.y()).append("]}");
  }
  LinkedHashMap<String,byte[]> p=new LinkedHashMap<>();
  p.put("correction",NativeMovement.destination(0xfedcba98L,32720,32780,7));
  for(int h=0;h<8;h++)p.put("remote_"+h,NativeMovement.destination(2,32720,32780,h));
  p.put("heading",Protocol.changeDirection(0xfedcba98L,3));
  p.put("remove",Protocol.removeObject(0xfedcba98L));
  p.put("lock",Protocol.movementLock());
  byte[] unlock=new Protocol.Writer().c(Protocol.C_BLINK).bytes();Protocol.movementUnlock(unlock);
  out.append("],\"unlock\":\"").append(HexFormat.of().formatHex(unlock)).append("\",\"packets\":{");boolean comma=false;
  for(var v:p.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
  out.append("},\"paralysis\":[");comma=false;
  for(var mode:Protocol.ParalysisMode.values()){if(comma)out.append(',');comma=true;out.append("{\"mode\":").append(mode.wireValue()).append(",\"body\":\"").append(HexFormat.of().formatHex(Protocol.paralysis(mode))).append("\"}");}
  System.out.println(out.append("]}"));
 }
}
