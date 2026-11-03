package local.lito381.server;
import java.util.*;
import java.nio.file.*;
import java.security.MessageDigest;
public final class NativeClassicEntryOracle {
    public static void main(String[] args)throws Exception{
        var jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        var replies=new StringJoiner(",");var requests=new StringJoiner(",");
        for(var result:Protocol.EnterWorldCheckResult.values())for(long value:new long[]{0,0x80000000L,0xffffffffL}){
            replies.add("{\"accepted\":"+(result==Protocol.EnterWorldCheckResult.ACCEPTED)+",\"wire_value\":"+result.wireValue()+",\"value\":"+value+",\"body\":\""+HexFormat.of().formatHex(Protocol.enterWorldCheck(result,value))+"\"}");
        }
        for(String name:new String[]{"測試角色","N".repeat(27)}){
            byte[] body=new Protocol.Writer().c(5).s(name).bytes();
            if(!Protocol.classicCharacterRequest(body,5).name().equals(name))throw new AssertionError();
            requests.add("{\"name\":\""+name+"\",\"body\":\""+HexFormat.of().formatHex(body)+"\"}");
        }
        System.out.println("{\"jar_sha256\":\""+sha+"\",\"requests\":["+requests+"],\"replies\":["+replies+"]}");
    }
}
