package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
public final class NativeEmblemOracle {
 public static void main(String[] args)throws Exception {
  byte[] pixels=new byte[384];int[] words={0,0x8000,0x7fff,0xffff,0x7c00,0x03e0,0x001f,0x4210};
  for(int i=0;i<192;i++){int w=words[i%words.length];pixels[i*2]=(byte)w;pixels[i*2+1]=(byte)(w>>>8);}
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String hash=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(hash).append("\",\"cases\":[");
  boolean comma=false;
  for(long id:new long[]{1,0x7fffffffL,0x80000000L,0xffffffffL}){
   byte[] request=new Protocol.Writer().c(0x30).d(id).bytes();
   if(Protocol.emblemRequest(request)!=id)throw new IllegalStateException("request round trip");
   byte[] response=Protocol.emblem(id,pixels);
   if(comma)out.append(',');comma=true;
   out.append("{\"id\":").append(id).append(",\"request\":\"").append(HexFormat.of().formatHex(request)).append("\",\"response\":\"").append(HexFormat.of().formatHex(response)).append("\"}");
  }
  int rejected=0;
  for(byte[] body:new byte[][]{{0x30,0,0,0,0},{0x30,1,0,0},{0x30,1,0,0,0,0},{0x31,1,0,0,0}}){
   try{Protocol.emblemRequest(body);throw new AssertionError("invalid request accepted");}catch(IllegalArgumentException expected){rejected++;}
  }
  System.out.println(out.append("],\"invalid_requests_rejected\":").append(rejected).append("}"));
 }
}
