package local.lito381.server;
import java.lang.reflect.*;
import java.nio.file.*;
import java.security.*;
import java.util.*;
/** Read-only catalog inventory. A catalog record alone does not prove gameplay support. */
public final class NativeSkillCoverageOracle {
    static String json(Object x)throws Exception {
        if(x==null)return "null";
        if(x instanceof Number || x instanceof Boolean)return x.toString();
        if(x instanceof Map<?,?> m){var out=new ArrayList<String>();for(var e:m.entrySet())out.add(json(e.getKey().toString())+":"+json(e.getValue()));return "{"+String.join(",",out)+"}";}
        if(x instanceof Collection<?> c){var out=new ArrayList<String>();for(var e:c)out.add(json(e));return "["+String.join(",",out)+"]";}
        if(x.getClass().isRecord()){var fields=new LinkedHashMap<String,Object>();for(var f:x.getClass().getRecordComponents())fields.put(f.getName(),f.getAccessor().invoke(x));return json(fields);}
        var out=new StringBuilder("\"");for(char c:x.toString().toCharArray()){if(c=='"'||c=='\\')out.append('\\').append(c);else if(c<32)out.append(String.format("\\u%04x",(int)c));else out.append(c);}return out.append('"').toString();
    }
    public static void main(String[] args)throws Exception {
        var methods=Arrays.stream(BattleSpellCatalog.class.getDeclaredMethods()).filter(m->Modifier.isStatic(m.getModifiers())
            && m.getParameterCount()==1 && m.getParameterTypes()[0]==int.class && m.getReturnType().isRecord()
            && (m.getName().endsWith("Spell")||m.getName().equals("reservedSkillSlot"))).sorted(Comparator.comparing(Method::getName)).toList();
        var skills=new TreeMap<Integer,Object>();
        for(int id=1;id<=220;id++){
            var records=new LinkedHashMap<String,Object>();
            for(Method m:methods){m.setAccessible(true);Object value=m.invoke(null,id);if(value!=null)records.put(m.getName(),value);}
            if(!records.isEmpty())skills.put(id,records);
        }
        Path jar=Path.of(BattleSpellCatalog.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        System.out.println(json(Map.of("jar_sha256",sha,"range",List.of(1,220),"getter_count",methods.size(),
            "skills",skills,"scope","Read-only catalog presence; actual endpoint reachability and full client behavior require separate verification")));
    }
}
