    // No mutation, game getter, refresh or owner admission occurs here.
    static String retainedMorphState(RuntimeInventory inventory,Object store,Object entry,Object alternativeEntry)
            throws ReflectiveOperationException {
        synchronized(inventory) {
            Object actor=field(entry,"actor"),alternative=field(alternativeEntry,"actor");
            Object template=field(actor,"_item"),alternativeTemplate=field(alternative,"_item");
            return json(object("schema","G06_E149_retained_morph_state_v1",
                "retained_state",retainedMutationState(inventory,store,entry),
                "actor_native_item_id",field(actor,"_itemId"),"template_native_item_id",field(template,"_itemId"),
                "alternate_actor_observer_id",id(alternative),"alternate_native_item_id",field(alternative,"_itemId"),
                "alternate_template_observer_id",id(alternativeTemplate),"alternate_template_native_item_id",field(alternativeTemplate,"_itemId"),
                "alternate_entry_observer_id",id(alternativeEntry),"alternate_view_observer_id",id(field(alternativeEntry,"view"))));
        }
    }
