package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;

/** Actual deployed encoders, including unsigned coordinates and empty ownership. */
public final class NativeWorldMetadataOracle {
    public static void main(String[] args)throws Exception {
        LinkedHashMap<String,byte[]> packets=new LinkedHashMap<>();
        packets.put("master_chinese",Protocol.master(0xfedcba98L,"測試主人"));
        packets.put("master_empty",Protocol.master(0xfedcba98L,""));
        packets.put("master_max",Protocol.master(0xffffffffL,"a".repeat(63)));
        packets.put("master_companion",Protocol.master(0x60000003L,"TestMage"));
        packets.put("box_arena",Protocol.observerBox(32704,32768,32768,32832));
        packets.put("box_max",Protocol.observerBox(0,0,65535,65535));
        packets.put("box_single",Protocol.observerBox(65534,65534,65535,65535));
        int rejected=0;
        for(String s:new String[]{"a".repeat(64),"測".repeat(32),"a\nb","a\u007fb"}){
            try{Protocol.master(1,s);throw new AssertionError("bad master accepted");}catch(IllegalArgumentException e){rejected++;}
        }
        for(long id:new long[]{0,-1,0x100000000L}){
            try{Protocol.master(id,"owner");throw new AssertionError("bad id accepted");}catch(IllegalArgumentException e){rejected++;}
        }
        for(int[] b:new int[][]{{0,0,0,1},{0,0,1,0},{2,0,1,1},{0,2,1,1},{-1,0,1,1},{0,-1,1,1},{0,0,65536,1},{0,0,1,65536}}){
            try{Protocol.observerBox(b[0],b[1],b[2],b[3]);throw new AssertionError("bad box accepted");}catch(IllegalArgumentException e){rejected++;}
        }
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        StringJoiner entries=new StringJoiner(",");
        for(var p:packets.entrySet())entries.add("\""+p.getKey()+"\":\""+HexFormat.of().formatHex(p.getValue())+"\"");
        System.out.println("{\"schema\":1,\"jar_sha256\":\""+sha+"\",\"rejected\":"+rejected+",\"packets\":{"+entries+"}}");
    }
}
