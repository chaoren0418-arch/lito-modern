package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
public final class NativeLobbyOracle {
    public static void main(String[] args)throws Exception {
        LinkedHashMap<String,byte[]> packets=new LinkedHashMap<>();
        for(var selection:Protocol.LitoChannelSelection.values()) {
            byte[] request=new byte[]{(byte)0xca,(byte)selection.wireValue()};
            if(Protocol.litoEnterChannelRequest(request)!=selection)throw new IllegalStateException("request");
            packets.put("request_"+selection.wireValue(),request);
            if(selection.wireValue()==100)continue;
            packets.put("success_"+selection.wireValue(),Protocol.litoEnterChannelSuccess(selection));
            packets.put("unavailable_"+selection.wireValue(),Protocol.litoEnterChannelFailure(Protocol.LitoEnterChannelFailure.CHANNEL_NOT_IN_SERVICE,selection));
            if(selection.wireValue()!=99)packets.put("restricted_"+selection.wireValue(),Protocol.litoEnterChannelFailure(Protocol.LitoEnterChannelFailure.CHARACTER_LEVEL_RESTRICTED,selection));
        }
        packets.put("population",Protocol.litoChannelInfo(Protocol.LitoChannelSelection.CHANNEL_3,65535,65535));
        packets.put("return",Protocol.allowPostTransition());
        packets.put("room",new Protocol.Writer().c(Protocol.S_LITO_ROOM_INFO).c(0).d(0xffffffffL).s("中文房間").c(0x81).h(32768).h(65535).h(0).c(255).bytes());
        packets.put("long_room",new Protocol.Writer().c(Protocol.S_LITO_ROOM_INFO).c(0).d(1).s("N".repeat(89)).c(0).h(1).h(8).h(0).c(0).bytes());
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");
        boolean comma=false;for(var p:packets.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(p.getKey()).append("\":\"").append(HexFormat.of().formatHex(p.getValue())).append('"');}
        out.append("},\"channel_names\":[");comma=false;for(String name:GameHub.CHANNEL_NAMES){if(comma)out.append(',');comma=true;out.append('"').append(name).append('"');}
        System.out.println(out.append("]}"));
    }
}
