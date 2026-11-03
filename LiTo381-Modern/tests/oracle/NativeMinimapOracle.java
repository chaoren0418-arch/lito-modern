package local.lito381.server;
import java.nio.file.*;import java.security.MessageDigest;import java.util.*;
/** Read-only wire vectors from the pinned current Protocol implementation. */
public final class NativeMinimapOracle {
 public static void main(String[] args)throws Exception{
  var jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  var rows=new StringJoiner(",");
  for(long id:new long[]{1,3,0xffffffffL})for(int[] pos:new int[][]{{0,0},{32729,32797},{65535,65535}})for(boolean allied:new boolean[]{false,true})
   rows.add("{\"id\":"+id+",\"x\":"+pos[0]+",\"y\":"+pos[1]+",\"allied\":"+allied+",\"hex\":\""+HexFormat.of().formatHex(Protocol.minimapPlayer(id,pos[0],pos[1],allied))+"\"}");
  System.out.println("{\"jar_sha256\":\""+HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)))+"\",\"vectors\":["+rows+"]}");
 }
}
