package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
public final class NativeLifecycleOracle {
    public static void main(String[] args)throws Exception {
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        StringJoiner modes=new StringJoiner(","),packets=new StringJoiner(",");
        for(var mode:BattleRules.Mode.values()){
            var settings=new GameHub.RoomSettings("LifecycleFixture","",32,0,mode.wireId(),0,0,0,0,List.of());
            var rules=BattleRules.decode(settings);
            if(rules.mode()!=mode)throw new AssertionError("mode constructor mismatch");
            modes.add("{\"id\":"+mode.wireId()+",\"key\":\""+mode.key()+"\",\"respawn\":"+rules.respawn()+",\"pvp\":"+mode.playerVersusPlayer()+",\"minimum_players\":"+rules.minimumPlayers()+",\"map\":"+rules.mapId()+",\"maps\":"+rules.mapPool()+"}");
        }
        for(long id:new long[]{1,0xfedcba98L,0xffffffffL})for(int action:new int[]{0,1,255})for(int gfx:new int[]{0,61,65535}){
            byte[] b=Protocol.resurrect(id,action,0xffffffffL,gfx);
            packets.add("{\"target\":"+id+",\"action\":"+action+",\"source\":4294967295,\"gfx\":"+gfx+",\"body\":\""+HexFormat.of().formatHex(b)+"\"}");
        }
        System.out.println("{\"schema\":1,\"jar_sha256\":\""+sha+"\",\"modes\":["+modes+"],\"resurrections\":["+packets+"]}");
    }
}
