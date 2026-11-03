package local.lito381.server;
import java.util.*;
import java.nio.file.*;
import java.security.MessageDigest;
public final class NativeServicesOracle {
    public static void main(String[] args)throws Exception{
        var jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        var vectors=new StringJoiner(",");
        for(int size:new int[]{0,1,127}){
            var rooms=new ArrayList<Protocol.LitoRoomDirectoryEntry>();var servers=new ArrayList<Protocol.LitoVsServerDirectoryEntry>();
            for(int i=0;i<size;i++)rooms.add(new Protocol.LitoRoomDirectoryEntry(i==0?"租裝村":"R".repeat(256),0x0200007fL,65535,0xffffffffL-i,0xffffffffL));
            for(int i=0;i<(size==127?10:size);i++)servers.add(new Protocol.LitoVsServerDirectoryEntry("戰鬥服",0x0100007fL,1,0x80000000L,0xffffffffL));
            vectors.add("{\"rooms\":"+rooms.size()+",\"servers\":"+servers.size()+",\"body\":\""+HexFormat.of().formatHex(Protocol.litoServerDirectory(rooms,servers))+"\"}");
        }
        System.out.println("{\"jar_sha256\":\""+sha+"\",\"vectors\":["+vectors+"],\"rental_join\":\""+HexFormat.of().formatHex(Protocol.litoRentalTownJoinResult(0xffffffffL,"租裝村",500,"玩家",2))+"\"}");
    }
}
