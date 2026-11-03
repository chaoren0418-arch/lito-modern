    // This cache holds reflection metadata only. It never stores/changes a
    // game inventory view, actor field, template, owner or ability value.
    private static final java.util.concurrent.ConcurrentHashMap<Class<?>,Map<String,Field>> observedFields=new java.util.concurrent.ConcurrentHashMap<>();
    private static Map<String,Field> declaredFields(Class<?> type) {
        if(observedFields.size()>=256)throw new IllegalStateException("bounded observer metadata cache full");
        var fields=new LinkedHashMap<String,Field>();
        for(Class<?> level=type;level!=null&&level.getName().startsWith("local.lito381.server.");level=level.getSuperclass()) {
            for(Field field:level.getDeclaredFields())if(!java.lang.reflect.Modifier.isStatic(field.getModifiers())) {
                field.setAccessible(true);fields.putIfAbsent(field.getName(),field);
            }
        }
        return Map.copyOf(fields);
    }
    private static Object field(Object value,String name)throws ReflectiveOperationException {
        Field found=observedFields.computeIfAbsent(value.getClass(),NativeRecoveryReadonlySnapshot::declaredFields).get(name);
        if(found==null)throw new NoSuchFieldException(value.getClass().getName()+"."+name);
        return found.get(value);
    }
    private static List<Integer> integerValues(Object value) {
        if(value==null)return null;var result=new ArrayList<Integer>();
        if(value.getClass()==int[].class) {
            int[] copy=((int[])value).clone();if(copy.length>1024)throw new IllegalStateException("observer integer array too large");
            for(int item:copy)result.add(item);
        } else {
            if(value.getClass()!=ArrayList.class)throw new IllegalStateException("unexpected native integer-list backing");
            Object[] copy=((ArrayList<?>)value).toArray();if(copy.length>1024)throw new IllegalStateException("observer integer list too large");
            for(Object item:copy){if(item!=null&&item.getClass()!=Integer.class)throw new IllegalStateException("unexpected native integer-list value");result.add((Integer)item);}
        }
        return result;
    }
    private static Object timestamp(Object value) {
        if(value==null)return null;
        if(value.getClass()!=java.sql.Timestamp.class)throw new IllegalStateException("unexpected native timestamp class");
        // Date/Timestamp getters can normalize their own calendar cache. Use
        // a clone, so even such JDK cache work cannot affect the game object.
        java.sql.Timestamp copy=(java.sql.Timestamp)((java.sql.Timestamp)value).clone();
        return object("epoch_millis",copy.getTime(),"nanos",copy.getNanos());
    }
    private static long atomicLong(Object value) {
        if(value==null||value.getClass()!=java.util.concurrent.atomic.AtomicLong.class)throw new IllegalStateException("unexpected native atomic-long class");
        return ((java.util.concurrent.atomic.AtomicLong)value).get();
    }
    private static int atomicInt(Object value) {
        if(value==null||value.getClass()!=java.util.concurrent.atomic.AtomicInteger.class)throw new IllegalStateException("unexpected native atomic-int class");
        return ((java.util.concurrent.atomic.AtomicInteger)value).get();
    }
    private static Map<String,Object> cachedAbilities(Object ability)throws ReflectiveOperationException {
        if(ability==null)return Map.of();Object values=field(ability,"_abilities");
        if(values.getClass()!=EnumMap.class)throw new IllegalStateException("unexpected native ability-map backing");
        EnumMap<?,?> copy=((EnumMap<?,?>)values).clone();var result=new TreeMap<String,Object>();
        if(copy.size()>512)throw new IllegalStateException("observer ability map too large");
        for(var entry:copy.entrySet()) {
            Enum<?> key=entry.getKey();if(key==null)throw new IllegalStateException("unexpected native ability key");
            Object value=entry.getValue();if(value!=null&&value.getClass()!=Integer.class)throw new IllegalStateException("unexpected native ability value");
            result.put(key.name(),value);
        }
        return result;
    }
    private static LinkedHashMap<String,Object> nativeAttributes(Object entry)throws ReflectiveOperationException {
        if(entry instanceof RuntimeInventory.Item)return object("present",false);
        Object actor=field(entry,"actor");if(actor==null)return object("present",false);
        var permanent=object("elemental_kind",field(actor,"_attrEnchantKind"),"elemental_level",field(actor,"_attrEnchantLevel"),
            "greater",field(actor,"_greater"),"inlay_cards",integerValues(field(actor,"_inlayCards")),
            "magic_stone_item_id",field(actor,"_magicStoneItemId"),"magic_stone_level",field(actor,"_magicStoneLevel"),
            "magic_stone_time",timestamp(field(actor,"_magicStoneTime")),"awaken_power_ids",integerValues(field(actor,"_awakenPowerIds")));
        var diagnostics=object("observer_actor_id",id(actor),"item_id",field(actor,"_itemId"),
            "count_mutation_version",atomicLong(field(actor,"_countMutationVersion")),"remaining_time",atomicInt(field(actor,"_remainingTime")),
            "item_owner_id",field(actor,"_itemOwnerId"),"character_object_id",field(actor,"_char_objid"),
            "equip_window",field(actor,"_equipWindow"),"temporarily_equipped",field(actor,"_isEquippedTemp"),
            "ability_dirty",field(actor,"_abilityDirty"),"now_lighting",field(actor,"_isNowLighting"),"match",field(actor,"_isMatch"),
            "dark_item_present",field(actor,"_darkItem")!=null,"magic_stone_generation",field(actor,"_magicStoneGeneration"),
            "equipment_timer_generation",field(actor,"_equipmentTimerGeneration"));
        return object("present",true,"schema","G06_E149_native_item_fields_v1","permanent",permanent,
            "cached_abilities",cachedAbilities(field(actor,"_ability")),"diagnostics",diagnostics);
    }
