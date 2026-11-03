package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
public final class NativeRoomOracle {
    public static void main(String[] args)throws Exception{
        var p=new LinkedHashMap<String,byte[]>();
        p.put("joined",new Protocol.Writer().c(Protocol.S_LITO_JOIN_RESULT).c(0).d(0xfedcba98L).s("測試房間").c(4).s("密碼").h(2).h(8).c(1).h(32768).c(1).c(2).c(3).c(4).c(5).s("測試房主").c(1).c(2).c(3).c(4).c(2).c(5).c(6).bytes());
        p.put("join_request",new Protocol.Writer().c(Protocol.C_LITO_JOIN).d(0xfedcba98L).s("密碼").bytes());
        for(var failure:Protocol.LitoJoinFailure.values())p.put("reject_"+failure.wireValue(),Protocol.joinRejected(failure));
        for(var state:Protocol.LitoParticipantState.values()){
            p.put("member_"+state.wireValue(),Protocol.litoRoomParticipant(0xfedcba98L,"測試成員",Protocol.LitoForce.FORCE_1,4,1,new GameStateStore.TournamentStatistics(10,6,4,1),"測試盟",-1,Integer.MAX_VALUE,state,14,GameStateStore.TournamentAwards.none()));
            p.put("ready_"+state.wireValue(),Protocol.readyState("測試成員",state));
            Protocol.litoReadyRequest(new byte[]{(byte)0xd2,(byte)state.wireValue()});
        }
        p.put("departure",new Protocol.Writer().c(Protocol.S_LITO_EXIT_ROOM).d(0xfedcba98L).s("測試成員").bytes());
        p.put("deleted",new Protocol.Writer().c(Protocol.S_LITO_DELETE_ROOM).d(0xfedcba98L).bytes());
        p.put("owner",Protocol.litoRoomOwner(0xfedcba98L,"測試成員"));
        p.put("countdown",new Protocol.Writer().c(Protocol.S_LITO_COUNTDOWN).c(255).bytes());
        p.put("start",Protocol.battleBootstrap(4,300).start());
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");boolean comma=false;
        for(var v:p.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
        System.out.println(out.append("}}"));
    }
}
