package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
public final class NativeLabelUpdatesOracle {
 public static void main(String[] args)throws Exception {
  LinkedHashMap<String,byte[]> packets=new LinkedHashMap<>();
  packets.put("title",Protocol.title(2,"稱號更新"));
  packets.put("title_empty",Protocol.title(2,""));
  packets.put("title_max",Protocol.title(0xfedcba98L,"一二三四五六七八"));
  packets.put("pledge",Protocol.pledgeMetadata(2,"測試盟","盟員稱號",0xffffffffL));
  packets.put("pledge_empty",Protocol.pledgeMetadata(2,"","",0));
  packets.put("clan",Protocol.battleClanMetadata(2,0x80000000L,"新血盟",true));
  packets.put("clan_empty",Protocol.battleClanMetadata(2,0,"",false));
  packets.put("clan_long",Protocol.battleClanMetadata(2,7,"12345678901234567890",true));
  packets.put("clan_wide",Protocol.battleClanMetadata(2,7,"一二三四五六七八九",true));
  packets.put("clan_split",Protocol.battleClanMetadata(2,7,"123456789012345中",true));
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
  String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");boolean comma=false;
  for(var e:packets.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(e.getKey()).append("\":\"").append(HexFormat.of().formatHex(e.getValue())).append('"');}
  System.out.println(out.append("}}"));
 }
}
