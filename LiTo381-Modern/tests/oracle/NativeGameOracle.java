package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
public final class NativeGameOracle {
    static String hex(byte[] bytes){return HexFormat.of().formatHex(bytes);}
    public static void main(String[] args)throws Exception {
        var rank=new Protocol.LitoCharacterRankings(1,2,3,4,5,-1,2,-3,4,-5);
        var battle=new Protocol.LitoCharacterBattleStatistics(2,10,6,4,3);
        var packets=new LinkedHashMap<String,byte[]>();
        packets.put("count",Protocol.litoCharacterDirectoryCount(2));
        packets.put("empty",Protocol.litoCharacterDirectoryCount(0));
        packets.put("row",Protocol.litoCharacterDirectoryRow(new Protocol.LitoCharacterDirectoryRow("重建角色","測試盟",6,1,-32768,32767,30000,-128,127,127,126,125,124,123,122,0xffffffffL,rank,battle)));
        packets.put("ready",Protocol.litoNewsTrigger());packets.put("notice438",Protocol.notice(438));packets.put("notice45",Protocol.notice(45));
        packets.put("news",Protocol.commonNews("請選擇角色。\n重建測試。"));
        for(var value:Protocol.LitoCharacterSelectResult.values())packets.put(value.name(),Protocol.litoCharacterSelectResult(value));
        byte[] select=new Protocol.Writer().c(Protocol.C_LITO_SELECT_CHAR).s("重建角色").bytes();
        if(!Protocol.litoCharacterSelectRequest(select).equals("重建角色"))throw new IllegalStateException("select parser");
        packets.put("select",select);
        Protocol.litoNewsRequest(new byte[]{(byte)0xe2});Protocol.loginResult(new byte[]{0x61});Protocol.readNotice(new byte[]{0x46});Protocol.readNews(new byte[]{0x33});
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=hex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        StringBuilder json=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"packets\":{");
        boolean comma=false;for(var item:packets.entrySet()){if(comma)json.append(',');comma=true;json.append('"').append(item.getKey()).append("\":\"").append(hex(item.getValue())).append('"');}
        System.out.println(json.append("}}"));
    }
}
