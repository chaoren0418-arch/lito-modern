package local.lito381.server;
import java.nio.file.*;import java.security.MessageDigest;import java.util.*;
public final class NativeRoomModeOracle {
    public static void main(String[] args)throws Exception{
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        StringJoiner packets=new StringJoiner(","),steps=new StringJoiner(",");int rejected=0;
        for(int v=0;v<256;v++){
            byte[] b=new byte[]{(byte)0xd1,2,(byte)v};
            try{var request=(Protocol.LitoModeStepRequest)Protocol.litoGameInfoRequest(b);
                steps.add("{\"value\":"+v+",\"direction\":\""+request.direction()+"\",\"next_from_deathmatch\":"+BattleRules.Mode.stepNative(1,request.direction()).wireId()+",\"body\":\""+HexFormat.of().formatHex(b)+"\"}");
            }catch(IllegalArgumentException e){rejected++;}
        }
        for(var m:BattleRules.Mode.values())packets.add("\"mode_"+m.wireId()+"\":\""+HexFormat.of().formatHex(Protocol.litoRoomMode(0xffffffffL,m))+"\"");
        for(int map:new int[]{0,510,65535})packets.add("\"map_"+map+"\":\""+HexFormat.of().formatHex(Protocol.litoRoomMap(0xffffffffL,map))+"\"");
        var settings=new GameHub.RoomSettings("模式房間","12",8,4,5,0,0,1,0,List.of());
        packets.add("\"summary\":\""+HexFormat.of().formatHex(Protocol.litoRoomSummary(0xffffffffL,settings,510,"測試角色"))+"\"");
        packets.add("\"configuration\":\""+HexFormat.of().formatHex(Protocol.litoRoomConfiguration(0xffffffffL,settings))+"\"");
        System.out.println("{\"jar_sha256\":\""+sha+"\",\"rejected_directions\":"+rejected+",\"steps\":["+steps+"],\"packets\":{"+packets+"}}");
    }
}
