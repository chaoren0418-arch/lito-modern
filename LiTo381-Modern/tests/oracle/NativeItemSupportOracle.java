package local.lito381.server;

import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;

/** Read-only E149 catalog audit. No sessions, Main, database or runtime mutation. */
public final class NativeItemSupportOracle {
    public static void main(String[] args) throws Exception {
        Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        if(!sha.equals("e1498871c7be6b287821c57ef5d95933db1740d9b4d4a80f6c3465f21d99d498"))throw new AssertionError(sha);
        System.out.println("jar="+sha+"; catalog scan=1..600000 inclusive");
        Set<Integer> types=Set.of(6,12,13,15,28,29,31,32,33,34,35,36);
        Map<Integer,Integer> counts=new TreeMap<>();
        for(int type:types) counts.put(type,0);
        for(int id=1;id<=600000;id++) {
            var d=InventoryCatalog.find(id);
            if(d!=null&&types.contains(d.useType())) {
                counts.merge(d.useType(),1,Integer::sum);
                System.out.println("catalog="+id+" type="+d.useType()+" effect="+d.effect()+" name="+d.name());
            }
        }
        if(!counts.equals(Map.ofEntries(Map.entry(6,1),Map.entry(12,1),Map.entry(13,2),Map.entry(15,1),
            Map.entry(28,5),Map.entry(29,0),Map.entry(31,0),Map.entry(32,0),Map.entry(33,0),Map.entry(34,0),Map.entry(35,0),Map.entry(36,0))))
            throw new AssertionError("formal catalog changed: "+counts);
        System.out.println("PASS counts="+counts+"; absence only applies to the stated scan range");
    }
}
