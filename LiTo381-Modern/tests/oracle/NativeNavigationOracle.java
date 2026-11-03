package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
public final class NativeNavigationOracle {
 public static void main(String[] args)throws Exception {
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  byte[] masks=new byte[4096];int[] dx={0,1,1,1,0,-1,-1,-1},dy={-1,-1,0,1,1,1,0,-1};var arena=ArenaMapCatalog.forMap(564);
  for(int y=0;y<64;y++)for(int x=0;x<64;x++)for(int h=0;h<8;h++)if(arena.nativeCanStep(x+32704,y+32768,x+32704+dx[h],y+32768+dy[h]))masks[y*64+x]|=(byte)(1<<h);
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"map\":{\"id\":564,\"origin\":[32704,32768],\"size\":[64,64],\"masks\":\"").append(HexFormat.of().formatHex(masks)).append("\"},\"speed\":[");boolean comma=false;
  for(var mode:Protocol.SpeedMode.values()){if(comma)out.append(',');comma=true;int seconds=mode.wireValue()==0?0:30;out.append("{\"mode\":").append(mode.wireValue()).append(",\"body\":\"").append(HexFormat.of().formatHex(Protocol.speed(0xfedcba98L,mode,seconds))).append("\"}");}
  out.append("],\"brave\":[");comma=false;
  for(var mode:Protocol.BraveMode.values()){if(comma)out.append(',');comma=true;int seconds=mode.wireValue()==0?0:45;out.append("{\"mode\":").append(mode.wireValue()).append(",\"body\":\"").append(HexFormat.of().formatHex(Protocol.braveSpeed(0xfedcba98L,mode,seconds))).append("\"}");}
  System.out.println(out.append("]}"));
 }
}
