package local.lito381.server;
import java.util.*;import java.nio.file.*;import java.security.MessageDigest;
/** Read-only protocol vectors from the pinned formal JAR, no server state. */
public final class NativeHotelInputOracle {
 static String hex(byte[] b){return HexFormat.of().formatHex(b);}
 public static void main(String[] args)throws Exception{
  var jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  long npc=1812987995L;
  byte[] reply=new Protocol.Writer().c(0x51).d(npc).d(8).c(0).s("").bytes();
  var parsed=Protocol.hypertextInputResult(reply);
  if(parsed.worldObjectId()!=npc)throw new IllegalStateException("identity");
  byte[] input=Protocol.hypertextInput(npc,0,1,1,8,0,"inn2","","",List.of("旅館服務生-席巴"));
  byte[] edge=Protocol.hypertextInput(0xffffffffL,0xffffffffL,2147483647L,0,2147483647L,255,"inn2","提示","回覆",List.of("中文",""));
  System.out.println("{\"jar_sha256\":\""+hex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)))+"\",\"npc\":"+npc+",\"input\":\""+hex(input)+"\",\"boundary\":\""+hex(edge)+"\",\"reply\":\""+hex(reply)+"\"}");
 }
}
