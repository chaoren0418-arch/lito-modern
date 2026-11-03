package local.lito381.server;
import java.nio.file.*;import java.security.*;import java.util.*;
public final class NativeForceOracle {
    public static void main(String[] args)throws Exception {
        StringBuilder packets=new StringBuilder();int valid=0;
        for(int n=0;n<256;n++){
            try {
                var force=Protocol.litoForceRequest(new byte[]{(byte)0xd3,(byte)n});
                if(valid++>0)packets.append(',');
                packets.append('"').append(n).append("\":\"").append(HexFormat.of().formatHex(Protocol.litoForceChanged("測試成員",force))).append('"');
            }catch(IllegalArgumentException expected){}
        }
        if(valid!=2)throw new IllegalStateException("force contract changed");
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        System.out.println("{\"jar_sha256\":\""+sha+"\",\"packets\":{"+packets+"},\"valid_forces\":2,\"rejected_forces\":254}");
    }
}
