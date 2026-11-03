    // Called only by the separate QA mutation probe. Observe original fields
    // and immutable record views; do not refresh views or invoke game getters.
    static String retainedMutationState(RuntimeInventory inventory,Object store,Object entry)
            throws ReflectiveOperationException {
        synchronized(inventory) {
            Object actor=field(entry,"actor");
            var core=object("count",field(actor,"_count"),"equipped",field(actor,"_isEquipped"),
                "identified",field(actor,"_isIdentified"),"sealed",field(actor,"_isSealed"),
                "enchant_level",field(actor,"_enchantLevel"),"durability",field(actor,"_durability"),
                "bless",field(actor,"_bless"),"charge_count",field(actor,"_chargeCount"),
                "template_observer_id",id(field(actor,"_item")));
            return json(object("schema","G06_E149_retained_mutation_state_v1",
                "store_observer_id",id(store),"entry_observer_id",id(entry),"actor_observer_id",id(actor),
                "entry_view_observer_id",id(field(entry,"view")),"actor_core",core,
                "inventory",inventory(inventory)));
        }
    }
