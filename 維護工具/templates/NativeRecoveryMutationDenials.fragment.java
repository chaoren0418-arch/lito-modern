 // This separate QA probe retains actual admitted objects. No owner, actor,
 // template or inventory is constructed, adopted, replaced or reflectively set.
 static final List<String> MUTATION_METHODS=List.of("setCount","setEquipped","setIdentified","setSealed",
     "setEnchantLevel","set_durability","setBless","setChargeCount");
 static void rememberMutationTarget(ClientSession session,String observation,
     IdentityHashMap<NativeDeItemApplication.Owner,Object[]> targets)throws Exception {
  if(!observation.endsWith(",\"stable_observed\":true,\"full_native_owner_qualification_inferred\":false}"))return;
  Object value=NativePeerHarness.field(session,"sourceItemOwner");
  if(!(value instanceof NativeDeItemApplication.Owner owner)||targets.containsKey(owner)||!owner.active||!owner.matches(session))return;
  boolean rental=Boolean.TRUE.equals(NativePeerHarness.field(session,"rentalTownRoomState"))
      &&"BATTLE".equals(String.valueOf(NativePeerHarness.field(session,"phase")));
  RuntimeInventory inventory=(RuntimeInventory)NativePeerHarness.field(session,rental?"rentalTownInventory":"inventory");
  if(inventory==null)return;
  synchronized(inventory) {
   Object store=NativePeerHarness.field(inventory,"items");
   if(!store.getClass().getName().equals("local.lito381.server.NativeDeItemInventory")
      ||NativePeerHarness.field(store,"owner")!=owner||NativePeerHarness.field(store,"rentalCharacter")!=null)return;
   long iid=NativeRecoveryReadonlySnapshot.observationId(inventory),oid=NativeRecoveryReadonlySnapshot.observationId(owner);
   if(!observation.contains("\"observer_inventory_id\":"+iid+",")
      ||!observation.contains("\"observer_owner_id\":"+oid+",\"active\":true,"))return;
   var entries=(Map<?,?>)NativePeerHarness.field(store,"entries");
   require(entries.size()<=512,"native mutation target inventory exceeds bound");
   for(var entry:entries.entrySet()) {
    Object row=entry.getValue(),actor=NativePeerHarness.field(row,"actor");
    if(actor==null)continue;
    require(actor.getClass().getName().equals("local.lito381.server.NativeDeItemInventory$BoundItem")
      &&NativePeerHarness.field(actor,"store")==store,"target is not an actual attached BoundItem");
    try {owner.requireActive(inventory);}
    catch(IllegalStateException error) {
     // Sampling may race the original retirement. Skip only its exact stale
     // rejection; four positively captured targets remain mandatory at exit.
     if(!owner.active&&error.getClass()==IllegalStateException.class&&"source item owner is stale".equals(error.getMessage()))return;
     throw error;
    }
    targets.put(owner,new Object[]{session,inventory,store,row,actor,entry.getKey(),observation});
    require(targets.size()<=4,"unexpected fifth native mutation owner admission");return;
   }
  }
 }
 static Object mutationArgument(Object actor,String method)throws Exception {
  String field=switch(method) {
   case "setCount" -> "_count";case "setEquipped" -> "_isEquipped";
   case "setIdentified" -> "_isIdentified";case "setSealed" -> "_isSealed";
   case "setEnchantLevel" -> "_enchantLevel";case "set_durability" -> "_durability";
   case "setBless" -> "_bless";case "setChargeCount" -> "_chargeCount";
   default -> throw new IllegalStateException("unreviewed native setter");
  };
  Object before=NativePeerHarness.field(actor,field);
  if(before instanceof Boolean flag)return !flag;
  long current=((Number)before).longValue();
  // Always use a different representable value. Only the original owner
  // guard, preceding validation and writes, may qualify this rejection.
  if(method.equals("setCount"))return current==1?2L:1L;
  return current==0?1:0;
 }
 static void attemptNativeMutation(NativeDeWorldKinds.Item actor,String method,Object argument) {
  switch(method) {
   case "setCount" -> actor.setCount((Long)argument);
   case "setEquipped" -> actor.setEquipped((Boolean)argument);
   case "setIdentified" -> actor.setIdentified((Boolean)argument);
   case "setSealed" -> actor.setSealed((Boolean)argument);
   case "setEnchantLevel" -> actor.setEnchantLevel((Integer)argument);
   case "set_durability" -> actor.set_durability((Integer)argument);
   case "setBless" -> actor.setBless((Integer)argument);
   case "setChargeCount" -> actor.setChargeCount((Integer)argument);
   default -> throw new IllegalStateException("unreviewed native mutation entry");
  }
 }
 static int retiredMutationDenials(BufferedWriter proofs,
     IdentityHashMap<NativeDeItemApplication.Owner,Object[]> targets)throws Exception {
  require(targets.size()==4,"four actual positively admitted native mutation targets required");
  int attempts=0;
  for(var retained:targets.entrySet()) {
   var owner=retained.getKey();Object[] target=retained.getValue();
   var session=(ClientSession)target[0];var inventory=(RuntimeInventory)target[1];
   Object store=target[2],entry=target[3];var actor=(NativeDeWorldKinds.Item)target[4];
   synchronized(inventory) {
    require(!owner.active&&!owner.matches(session)
      &&Boolean.TRUE.equals(NativePeerHarness.field(session,"sourceItemDisconnected")),"original mutation owner not retired");
    require(NativePeerHarness.field(inventory,"items")==store&&NativePeerHarness.field(store,"owner")==owner
      &&NativePeerHarness.field(store,"rentalCharacter")==null,"retained actor no longer has the original unstaged owner");
    require(((Map<?,?>)NativePeerHarness.field(store,"entries")).get(target[5])==entry
      &&NativePeerHarness.field(entry,"actor")==actor&&NativePeerHarness.field(actor,"store")==store,
      "retained native actor detached; do not relabel detached rejection as owner retirement");
    for(String method:MUTATION_METHODS) {
     String before=NativeRecoveryReadonlySnapshot.retainedMutationState(inventory,store,entry);
     Object argument=mutationArgument(actor,method);String rejection=null;
     try {attemptNativeMutation(actor,method,argument);}
     catch(IllegalStateException error) {
      require(error.getClass()==IllegalStateException.class&&"source item owner is stale".equals(error.getMessage()),
        "wrong native mutation rejection; preserve original failure");rejection=error.getMessage();
     }
     String after=NativeRecoveryReadonlySnapshot.retainedMutationState(inventory,store,entry);
     require(rejection!=null&&before.equals(after),"retired native setter accepted or changed observed state");
     var row=new LinkedHashMap<String,Object>();
     row.put("schema","G06_E149_retired_bound_item_mutation_v1");row.put("method",method);row.put("argument",argument);
     row.put("observer_owner_id",NativeRecoveryReadonlySnapshot.observationId(owner));
     row.put("session_identity",NativeRecoveryReadonlySnapshot.observationId(session));
     row.put("inventory_observer_id",NativeRecoveryReadonlySnapshot.observationId(inventory));
     row.put("actor_observer_id",NativeRecoveryReadonlySnapshot.observationId(actor));row.put("item_id",target[5]);
     row.put("source_qualified_character",owner.source);row.put("source_qualified_account",owner.account);
     row.put("positive_guard_at_capture",true);row.put("owner_retired_before_attempt",true);row.put("actor_attached_before_attempt",true);
     row.put("original_session_futures_completed_before_probe",true);row.put("attempted_real_setter",true);
     row.put("argument_differs_from_original",true);row.put("rejection_class","java.lang.IllegalStateException");row.put("rejection",rejection);
     row.put("before_state",before);row.put("after_state",after);row.put("source_observation",target[6]);
     row.put("full_native_owner_qualified",false);row.put("full_attribute_matrix_qualified",false);
     proofs.write(NativeRecoveryReadonlySnapshot.json(row)+"\n");proofs.flush();attempts++;
    }
   }
  }
  require(attempts==32,"expected eight exact native setters per four original owner admissions");return attempts;
 }
