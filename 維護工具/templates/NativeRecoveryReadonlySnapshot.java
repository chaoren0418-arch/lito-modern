package local.lito381.server;

import java.lang.ref.WeakReference;
import java.lang.reflect.Field;
import java.util.*;

/** Narrow G06 observer. No gameplay getter, inventory view refresh or field set. */
final class NativeRecoveryReadonlySnapshot {
    private static final IdentityHashMap<Object,Long> observedIds=new IdentityHashMap<>();
    private static synchronized long id(Object value) {
        if(value==null)return 0;
        Long old=observedIds.get(value);if(old!=null)return old;
        if(observedIds.size()>=16384)throw new IllegalStateException("bounded observer identity table full");
        long next=observedIds.size()+1L;observedIds.put(value,next);return next;
    }
    private static Object field(Object value,String name)throws ReflectiveOperationException {
        for(Class<?> type=value.getClass();type!=null;type=type.getSuperclass()) {
            try {Field f=type.getDeclaredField(name);f.setAccessible(true);return f.get(value);}
            catch(NoSuchFieldException absent) { /* Continue only through the object's declared hierarchy. */ }
        }
        throw new NoSuchFieldException(value.getClass().getName()+"."+name);
    }
    private static LinkedHashMap<String,Object> object(Object... pairs) {
        if(pairs.length%2!=0)throw new IllegalArgumentException("JSON pairs");
        var row=new LinkedHashMap<String,Object>();
        for(int i=0;i<pairs.length;i+=2)row.put((String)pairs[i],pairs[i+1]);return row;
    }
    private static String json(Object value) {
        if(value==null)return "null";
        if(value instanceof Boolean||value instanceof Number)return String.valueOf(value);
        if(value instanceof Map<?,?> values) {
            var out=new StringJoiner(",","{","}");
            for(var e:values.entrySet())out.add(json(String.valueOf(e.getKey()))+":"+json(e.getValue()));return out.toString();
        }
        if(value instanceof Collection<?> values) {
            var out=new StringJoiner(",","[","]");for(Object v:values)out.add(json(v));return out.toString();
        }
        String text=String.valueOf(value);var out=new StringBuilder("\"");
        for(int i=0;i<text.length();i++) {
            char c=text.charAt(i);
            if(c=='"'||c=='\\')out.append('\\').append(c);
            else if(c<32)out.append(String.format(Locale.ROOT,"\\u%04x",(int)c));else out.append(c);
        }
        return out.append('"').toString();
    }
    private static List<?> entries(Object inventory)throws ReflectiveOperationException {
        Object map=field(inventory,"items");
        if(map.getClass().getName().equals("local.lito381.server.NativeDeItemInventory")) {
            Object stored=field(map,"entries");
            if(stored.getClass()!=LinkedHashMap.class)throw new IllegalStateException("unexpected native entry container");
            return new ArrayList<>(((Map<?,?>)stored).values());
        }
        if(map.getClass()!=LinkedHashMap.class&&map.getClass()!=HashMap.class)
            throw new IllegalStateException("unsupported inventory backing: "+map.getClass().getName());
        return new ArrayList<>(((Map<?,?>)map).values());
    }
    private static RuntimeInventory.Item view(Object entry)throws ReflectiveOperationException {
        return entry instanceof RuntimeInventory.Item item?item:(RuntimeInventory.Item)field(entry,"view");
    }
    private static LinkedHashMap<String,Object> item(Object entry)throws ReflectiveOperationException {
        RuntimeInventory.Item row=view(entry);
        Object actor=entry instanceof RuntimeInventory.Item?null:field(entry,"actor");
        var result=object("id",row.objectId(),"count",actor==null?row.quantity():field(actor,"_count"),
            "catalog_id",row.definition().catalogId(),"name",row.definition().name(),
            "equipped",actor==null?row.equipped():field(actor,"_isEquipped"),
            "durability",actor==null?row.durability():field(actor,"_durability"),
            "enchant_level",actor==null?row.enchantLevel():field(actor,"_enchantLevel"),
            "identified",actor==null?row.identified():field(actor,"_isIdentified"),
            "cached_packet_bless",row.blessing(),"raw_native_bless",actor==null?null:field(actor,"_bless"),
            "charges",actor==null?row.chargeCount():field(actor,"_chargeCount"),
            "sealed",actor==null?row.sealed():field(actor,"_isSealed"),
            "source_bound",row.sourceBound(),"tournament_reward",row.tournamentReward(),
            "expires_at",row.expiresAtEpochMillis(),"lito_owned",row.litoOwned(),"bound_actor_present",actor!=null);
        return result;
    }
    static List<Long> inventoryIds(Object inventory)throws ReflectiveOperationException {
        if(inventory==null)return List.of();
        synchronized(inventory) {
            var ids=new ArrayList<Long>();for(Object entry:entries(inventory))ids.add(view(entry).objectId());
            Collections.sort(ids);return ids;
        }
    }
    private static LinkedHashMap<String,Object> inventory(Object inventory)throws ReflectiveOperationException {
        if(inventory==null)return object("exists",false,"rows",List.of(),"stable_observed",true);
        synchronized(inventory) {
            Object map=field(inventory,"items");boolean nativeMap=map.getClass().getName().equals("local.lito381.server.NativeDeItemInventory");
            long before=nativeMap?((Number)field(map,"mutationRevision")).longValue():-1;
            Object owner=nativeMap?field(map,"owner"):null;
            var rows=new ArrayList<Map<String,Object>>();boolean stable=true;
            for(Object entry:entries(inventory)) {
                var first=item(entry);var second=item(entry);stable&=first.equals(second);rows.add(first);
            }
            rows.sort(Comparator.comparingLong(row->((Number)row.get("id")).longValue()));
            long after=nativeMap?((Number)field(map,"mutationRevision")).longValue():-1;
            stable&=before==after&&(!nativeMap||owner==field(map,"owner"));
            return object("exists",true,"observer_inventory_id",id(inventory),"backing",map.getClass().getName(),
                "native_owner_id",id(owner),"mutation_revision",after,"stable_observed",stable,"rows",rows,
                "rental_character",nativeMap?field(map,"rentalCharacter"):null,
                "rental_account",nativeMap?field(map,"rentalAccount"):null);
        }
    }
    static String observe(NativePeerHarness.Session session,GameHub hub)throws ReflectiveOperationException {
        ClientSession c=session.owner();
        // No client/action/hub lock is acquired here. Inventory monitors are
        // taken one at a time; before/after fields describe observation
        // stability, not an atomic gameplay snapshot or authority grant.
        {
            Object source=field(c,"selectedClassicServerName"),selected=field(c,"selectedCharacter"),phase=field(c,"phase");
            Object owner=field(c,"sourceItemOwner");
            var nativeInventory=inventory(field(c,"inventory"));var rentalInventory=inventory(field(c,"rentalTownInventory"));
            Object ownerRow=owner==null?null:object("observer_owner_id",id(owner),"active",field(owner,"active"),
                "object_id",field(owner,"objectId"),"selected",field(owner,"selected"),"source",field(owner,"source"),
                "account",field(owner,"account"),"session_matches",((WeakReference<?>)field(owner,"session")).get()==c);
            Object receivedCharacters=field(c,"characters");Long selectedId=null;
            var rows=new ArrayList<String>();for(Object value:(List<?>)receivedCharacters) {
                CharacterSummary row=(CharacterSummary)value;rows.add(row.name());
                if(Objects.equals(row.name(),selected))selectedId=row.objectId();
            }
            boolean stable=Objects.equals(source,field(c,"selectedClassicServerName"))&&Objects.equals(selected,field(c,"selectedCharacter"))
                &&phase==field(c,"phase")&&owner==field(c,"sourceItemOwner")&&receivedCharacters==field(c,"characters")&&Boolean.TRUE.equals(nativeInventory.get("stable_observed"))&&Boolean.TRUE.equals(rentalInventory.get("stable_observed"));
            return json(object("observer_kind","G06_E149_direct_fields_v1","observer_no_gameplay_getters",true,
                "session_identity",id(c),"test_account",field(c,"account"),"phase",String.valueOf(phase),
                "character",selected==null?"":String.valueOf(selected),"source",String.valueOf(source),
                "object_id",selectedId,"room",field(c,"currentRoomId"),
                "rental_town",field(c,"rentalTownRoomState"),"rental_map",field(c,"rentalTownMapId"),"rental_set",field(c,"rentalItemSetSelection"),
                "x",field(c,"positionX"),"y",field(c,"positionY"),"heading",field(c,"playerHeading"),
                "hp",field(c,"playerHp"),"mp",field(c,"playerMp"),"shop_points",field(c,"shopPoints"),
                "available_characters",rows,"native_inventory",nativeInventory.get("rows"),"rental_inventory",rentalInventory.get("rows"),
                "native_inventory_state",nativeInventory,"rental_inventory_state",rentalInventory,
                "source_item_application_active",field(hub,"sourceItemApplication")!=null,
                "source_item_owner",ownerRow,"source_item_disconnected",field(c,"sourceItemDisconnected"),"stable_observed",stable,
                "full_native_owner_qualification_inferred",false));
        }
    }
}
