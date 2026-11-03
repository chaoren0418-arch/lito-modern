package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
public final class NativeWorldOracle {
 public static void main(String[] args)throws Exception {
  LinkedHashMap<String,byte[]> p=new LinkedHashMap<>();
  var bootstrap=Protocol.battleBootstrap(564,300);
  p.put("world",bootstrap.world());p.put("start",bootstrap.start());p.put("limit",bootstrap.timeLimit());
  p.put("status",Protocol.characterStatus(0xfedcba98L,52,0xffffffffL,18,16,14,12,10,8,100,50,200,90,-20,225,120,-32768,1,2,3,4));
  p.put("actor",Protocol.battleActorWithHeading(0xfedcba98L,32720,32780,0,0,7,3,2,"測試角色","稱號",4,1,"測試盟",-32768,255,52));
  p.put("door",Protocol.battleDoor(0x62000000L,32736,32800,29));
  p.put("bot",Protocol.battleActorWithHeading(2,32721,32781,61,0,0,0,0,"ArenaBot","",0,0,"",0,100,1));
  p.put("hp",new Protocol.Writer().c(Protocol.S_HIT_POINT).h(100).h(200).bytes());
  p.put("mp",Protocol.manaPoint(50,90));p.put("time",Protocol.worldTime(123456));
  p.put("challenge",Protocol.addressChallenge(0xffffffffL,0x401000,4096));
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");boolean comma=false;
  for(var v:p.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
  System.out.println(out.append("}}"));
 }
}
