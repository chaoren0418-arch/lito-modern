package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
public final class NativeGuardianOracle {
    public static void main(String[] args)throws Exception {
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        StringJoiner rows=new StringJoiner(",");
        for(int gfx:new int[]{2665,2667,2669,2671}){
            var tower=new BattleGuardianTower();var arena=ArenaMapCatalog.forMap(510);
            tower.reset(0,arena,new ArenaMapCatalog.Point(32765,32810),gfx,"Guardian","isolated oracle");
            for(int damage:new int[]{0,751,750,750,748,1}) {
                if(damage>0)tower.damage(damage);var s=tower.snapshot();
                boolean blocked=tower.blocks(s.x(),s.y());
                if(blocked!=(s.hp()>0)||tower.blocks(s.x()+1,s.y()))throw new AssertionError("occupancy");
                rows.add("{\"id\":"+s.objectId()+",\"gfx\":"+gfx+",\"hp\":"+s.hp()+",\"action\":"+s.action()+",\"blocks\":"+blocked+
                    ",\"body\":\""+HexFormat.of().formatHex(Protocol.guardianTower(s))+"\"}");
            }
            tower.deactivate();if(tower.snapshot()!=null||tower.blocks(32765,32810))throw new AssertionError("deactivation");
        }
        System.out.println("{\"schema\":1,\"jar_sha256\":\""+sha+"\",\"states\":["+rows+"]}");
    }
}
