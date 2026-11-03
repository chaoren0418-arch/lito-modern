package local.lito381.server;
import java.util.*;
import java.nio.file.*;
import java.security.MessageDigest;
/** Read-only schema-0 reference vectors. Extended numeric layouts require their own negotiation gate. */
public final class NativeAttributesOracle {
    public static void main(String[] args)throws Exception{
        if(NumericBreakthrough.schemaVersion()!=0)throw new IllegalStateException("schema 0 required");
        var jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        var rows=new StringJoiner(",");
        for(int value:new int[]{0,1,127,128,255}){
            byte[] b=Protocol.abilityScores(value,2,3,4,5,6,value);
            rows.add("{\"kind\":\"ability\",\"strength\":"+value+",\"body\":\""+HexFormat.of().formatHex(b)+"\"}");
        }
        for(int value:new int[]{-128,-1,0,1,127}){
            rows.add("{\"kind\":\"armor\",\"value\":"+value+",\"body\":\""+HexFormat.of().formatHex(Protocol.armorClass(value,-128,127,-1,1))+"\"}");
            rows.add("{\"kind\":\"magic\",\"value\":"+value+",\"body\":\""+HexFormat.of().formatHex(Protocol.magicStatus(value,-value-1))+"\"}");
        }
        for(int level:new int[]{1,255})for(long experience:new long[]{0,0x80000000L,0xffffffffL})rows.add("{\"kind\":\"experience\",\"level\":"+level+",\"experience\":"+experience+",\"body\":\""+HexFormat.of().formatHex(Protocol.experience(level,experience))+"\"}");
        System.out.println("{\"jar_sha256\":\""+sha+"\",\"schema\":0,\"vectors\":["+rows+"]}");
    }
}
