package local.lito381.server;
import java.util.*;import java.nio.file.*;import java.security.MessageDigest;
public final class NativePortalOracle{
 static String hex(byte[] b){return HexFormat.of().formatHex(b);}
 public static void main(String[] args)throws Exception{
  var jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());Protocol.gotoPortal(new byte[]{8});
  var rooms=new StringJoiner(",");for(var room:NativeLitoRentalHotelCatalog.rooms()){
   rooms.add("{\"map\":"+room.mapId()+",\"keys_min\":"+room.minimumKeys()+",\"keys_max\":"+room.maximumKeys()+",\"x\":"+room.entryX()+",\"y\":"+room.entryY()+",\"heading\":"+room.entryHeading()+",\"seconds\":"+room.durationSeconds()+",\"npcs\":"+NativeLitoRentalTownNpcCatalog.forMap(room.mapId()).size()+",\"portal\":\""+hex(Protocol.portal(room.mapId(),room.entryX(),room.entryY()))+"\"}");
  }
  System.out.println("{\"jar_sha256\":\""+hex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)))+"\",\"ack\":\"08\",\"same_map\":\""+hex(Protocol.portal(564,32720,32780))+"\",\"boundary\":\""+hex(Protocol.portal(65535,0,65535))+"\",\"rooms\":["+rooms+"]}");
 }
}
