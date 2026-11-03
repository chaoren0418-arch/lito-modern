package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;

/** Formal codec and isolated door state vectors, not a live-player acceptance. */
public final class NativeDoorOracle {
    static String hex(byte[] b){return "\""+HexFormat.of().formatHex(b)+"\"";}
    static String snap(BattleDoor.Snapshot s){return "{\"id\":"+s.objectId()+",\"x\":"+s.x()+",\"y\":"+s.y()+
        ",\"action\":"+s.action()+",\"open\":"+s.open()+",\"direction\":"+s.direction()+",\"passable\":"+s.passable()+"}";}
    public static void main(String[] args)throws Exception{
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        List<String> attributes=new ArrayList<>(),opens=new ArrayList<>();
        for(int[] xy:new int[][]{{0,0},{32768,65535},{65535,32768},{32736,32800}}){
            for(int direction:new int[]{0,1})for(int passable:new int[]{0,65})
                attributes.add("{\"x\":"+xy[0]+",\"y\":"+xy[1]+",\"direction\":"+direction+",\"passable\":"+passable+
                    ",\"body\":"+hex(Protocol.doorAttribute(xy[0],xy[1],direction,passable))+"}");
            for(long id:new long[]{1,0x62000000L,0xffffffffL}){
                byte[] body=new Protocol.Writer().c(0x0e).h(xy[0]).h(xy[1]).d(id).bytes();
                var request=Protocol.openRequest(body);
                if(request.x()!=xy[0]||request.y()!=xy[1]||request.worldObjectId()!=id)throw new AssertionError("open roundtrip");
                opens.add("{\"x\":"+request.x()+",\"y\":"+request.y()+",\"id\":"+request.worldObjectId()+",\"body\":"+hex(body)+"}");
            }
        }
        int accepted=0,rejected=0;
        for(int direction=0;direction<256;direction++)for(int passable=0;passable<256;passable++){
            try{Protocol.doorAttribute(32736,32800,direction,passable);accepted++;}catch(IllegalArgumentException e){rejected++;}
        }
        if(accepted!=4||rejected!=65532)throw new AssertionError("attribute domain");
        BattleDoor door=new BattleDoor();door.reset(ArenaMapCatalog.forMap(564));var closed=door.snapshot();
        if(!door.blocks(closed.x(),closed.y())||door.blocks(closed.x()+1,closed.y()))throw new AssertionError("closed occupancy");
        var open=door.toggle();if(door.blocks(open.x(),open.y())||!open.open()||open.action()!=28||open.passable()!=0)throw new AssertionError("open occupancy");
        var again=door.toggle();if(!door.blocks(again.x(),again.y())||again.open()||again.action()!=29||again.passable()!=65)throw new AssertionError("reclosed occupancy");
        System.out.println("{\"schema\":1,\"jar_sha256\":\""+sha+"\",\"attribute_accepted\":"+accepted+",\"attribute_rejected\":"+rejected+
            ",\"attributes\":["+String.join(",",attributes)+"],\"open_requests\":["+String.join(",",opens)+"],\"door_states\":["+snap(closed)+","+snap(open)+","+snap(again)+"],\"live_players_verified\":false}");
    }
}
