package local.lito381.server;
import java.util.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.security.MessageDigest;
/** Immutable formal catalogue only; no sessions or authority mutation. */
public final class NativeRentalCatalogOracle {
    static String quote(String s){return "\""+s.replace("\\","\\\\").replace("\"","\\\"").replace("\n","\\n").replace("\r","\\r")+"\"";}
    static String json(Object value)throws Exception{
        if(value==null)return "null";if(value instanceof Number||value instanceof Boolean)return value.toString();
        if(value instanceof Iterable<?> rows){var j=new StringJoiner(",","[","]");for(var row:rows)j.add(json(row));return j.toString();}
        if(value.getClass().isRecord()){var j=new StringJoiner(",","{","}");for(var field:value.getClass().getRecordComponents())j.add(quote(field.getName())+":"+json(field.getAccessor().invoke(value)));return j.toString();}
        return quote(value.toString());
    }
    public static void main(String[] args)throws Exception{
        var jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        var flags=new StringJoiner(",");
        for(var f:NativeLitoRentalTownNpcCatalog.class.getDeclaredFields())if(Modifier.isStatic(f.getModifiers())&&f.getType()==boolean.class)flags.add(quote(f.getName())+":"+f.get(null));
        var rows=new StringJoiner(",");for(int map:new int[]{500,560})rows.add("{\"map\":"+map+",\"scene\":"+json(NativeLitoTownScenePlan.require(map))+",\"npcs\":"+json(NativeLitoRentalTownNpcCatalog.forMap(map))+"}");
        System.out.println("{\"jar_sha256\":\""+sha+"\",\"flags\":{"+flags+"},\"maps\":["+rows+"]}");
    }
}
