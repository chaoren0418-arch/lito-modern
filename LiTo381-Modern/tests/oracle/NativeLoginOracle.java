package local.lito381.server;
import java.util.*;
import java.nio.file.*;
import java.security.MessageDigest;

/** Independent fixtures from the deployed Protocol builders and parsers. */
public final class NativeLoginOracle {
    static String hex(byte[] bytes){return HexFormat.of().formatHex(bytes);}
    public static void main(String[] args)throws Exception {
        byte[] login=new Protocol.Writer().c(Protocol.C_LOGIN).s("重建帳號").s("測試密碼").d(0x0100007fL).bytes();
        Protocol.LoginRequest parsed=Protocol.loginRequest(login);
        if(!parsed.account().equals("重建帳號")||!parsed.password().equals("測試密碼")||parsed.hasConnector())throw new IllegalStateException("login parser");
        byte[] select=new Protocol.Writer().c(Protocol.C_LITO_LIN_CHARACTER_INFO).s("測試乙").bytes();
        if(!Protocol.litoCharacterSourceRequest(select).sourceName().equals("測試乙"))throw new IllegalStateException("source parser");
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=hex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"login\":\"").append(hex(login)).append("\",\"source_request\":\"").append(hex(select)).append("\",\"replies\":{");
        LinkedHashMap<String,byte[]> replies=new LinkedHashMap<>();
        for(Protocol.LoginCheckResult result:Protocol.LoginCheckResult.values())replies.put(result.name(),Protocol.loginCheck(result,3600,"LiTo 重建測試",0,0));
        replies.put("login_ok",Protocol.litoLoginOk());
        replies.put("sources",Protocol.classicServerList(List.of(new Protocol.ClassicServerEndpoint("測試甲",0x0100007fL,32000),new Protocol.ClassicServerEndpoint("測試乙",0x0200007fL,32100))));
        replies.put("redirect",Protocol.litoVsServerIp(0,0x0200007fL,32100));
        boolean comma=false;for(var entry:replies.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(entry.getKey()).append("\":\"").append(hex(entry.getValue())).append('"');}
        System.out.println(out.append("}}").toString());
    }
}
