 // Separate QA-only extension. Keep the original eight-setter probe unchanged.
 static void rememberMorphTarget(ClientSession session,String observation,
     IdentityHashMap<NativeDeItemApplication.Owner,Object[]> mutationTargets,
     IdentityHashMap<NativeDeItemApplication.Owner,Object[]> targets)throws Exception {
  Object value=NativePeerHarness.field(session,"sourceItemOwner");
  if(!(value instanceof NativeDeItemApplication.Owner owner)||targets.containsKey(owner)
      ||!owner.active||!owner.matches(session))return;
  Object[] original=mutationTargets.get(owner);if(original==null||!observation.equals(original[6]))return;
  var inventory=(RuntimeInventory)original[1];Object store=original[2],entry=original[3],actor=original[4];
  synchronized(inventory) {
   var entries=(Map<?,?>)NativePeerHarness.field(store,"entries");
   require(entries.size()<=512,"morph capture inventory exceeds bound");
   Object template=NativePeerHarness.field(actor,"_item");int itemId=((Number)NativePeerHarness.field(actor,"_itemId")).intValue();
   require(template!=null&&((Number)NativePeerHarness.field(template,"_itemId")).intValue()==itemId,
       "original morph template/item ID differs");
   for(var row:entries.entrySet()) {
    Object alternativeEntry=row.getValue(),alternativeActor=NativePeerHarness.field(alternativeEntry,"actor");
    if(alternativeEntry==entry||alternativeActor==null)continue;
    require(alternativeActor.getClass().getName().equals("local.lito381.server.NativeDeItemInventory$BoundItem")
       &&NativePeerHarness.field(alternativeActor,"store")==store,"alternative is not an attached original BoundItem");
    Object alternativeTemplate=NativePeerHarness.field(alternativeActor,"_item");
    int alternativeId=((Number)NativePeerHarness.field(alternativeActor,"_itemId")).intValue();
    if(alternativeTemplate==null||alternativeTemplate==template||alternativeId==itemId)continue;
    require(alternativeId>0&&((Number)NativePeerHarness.field(alternativeTemplate,"_itemId")).intValue()==alternativeId,
        "alternative template/item ID differs");
    try {owner.requireActive(inventory);}
    catch(IllegalStateException error) {
     if(!owner.active&&error.getClass()==IllegalStateException.class&&"source item owner is stale".equals(error.getMessage()))return;
     throw error;
    }
    targets.put(owner,new Object[]{original,alternativeEntry,alternativeActor,alternativeTemplate,row.getKey(),itemId,alternativeId});
    require(targets.size()<=4,"unexpected fifth original morph admission");return;
   }
  }
 }
 static int retiredMorphDenials(BufferedWriter proofs,
     IdentityHashMap<NativeDeItemApplication.Owner,Object[]> targets)throws Exception {
  require(targets.size()==4,"four original admissions with two distinct actual templates required");int attempts=0;
  for(var retained:targets.entrySet()) {
   var owner=retained.getKey();Object[] value=retained.getValue(),original=(Object[])value[0];
   var session=(ClientSession)original[0];var inventory=(RuntimeInventory)original[1];
   Object store=original[2],entry=original[3],alternativeEntry=value[1],alternativeActor=value[2];
   var actor=(NativeDeWorldKinds.Item)original[4];var template=(NativeDeItemTemplate)value[3];
   synchronized(inventory) {
    require(!owner.active&&!owner.matches(session)&&Boolean.TRUE.equals(NativePeerHarness.field(session,"sourceItemDisconnected")),
        "original morph owner not retired");
    var entries=(Map<?,?>)NativePeerHarness.field(store,"entries");
    require(NativePeerHarness.field(inventory,"items")==store&&NativePeerHarness.field(store,"owner")==owner
        &&NativePeerHarness.field(store,"rentalCharacter")==null&&NativePeerHarness.field(store,"rentalAccount")==null,
        "retained morph inventory no longer has original unstaged owner");
    require(entries.get(original[5])==entry&&NativePeerHarness.field(entry,"actor")==actor
        &&NativePeerHarness.field(actor,"store")==store&&entries.get(value[4])==alternativeEntry
        &&NativePeerHarness.field(alternativeEntry,"actor")==alternativeActor
        &&NativePeerHarness.field(alternativeActor,"store")==store&&NativePeerHarness.field(alternativeActor,"_item")==template,
        "actual morph target or alternative was detached/replaced");
    int current=((Number)NativePeerHarness.field(actor,"_itemId")).intValue(),alternative=(Integer)value[6];
    Object originalTemplate=NativePeerHarness.field(actor,"_item");
    require(current==(Integer)value[5]&&current!=alternative&&originalTemplate!=template
        &&((Number)NativePeerHarness.field(alternativeActor,"_itemId")).intValue()==alternative
        &&((Number)NativePeerHarness.field(template,"_itemId")).intValue()==alternative,
        "original different-template argument identity changed");
    for(String method:List.of("setItem","setItemId")) {
     String before=NativeRecoveryReadonlySnapshot.retainedMorphState(inventory,store,entry,alternativeEntry);RuntimeException failure=null;
     try {if(method.equals("setItem"))actor.setItem(template);else actor.setItemId(alternative);}
     catch(RuntimeException error) {failure=error;}
     String after=NativeRecoveryReadonlySnapshot.retainedMorphState(inventory,store,entry,alternativeEntry);
     // Seal the actual outcome before accepting or rejecting it. A wrong
     // validation exception or accepted mutation must retain its own receipt.
     String rejection=failure==null?null:failure.getMessage();
     var row=new LinkedHashMap<String,Object>();row.put("schema","G06_E149_retired_bound_morph_v1");row.put("method",method);
     row.put("observer_owner_id",NativeRecoveryReadonlySnapshot.observationId(owner));row.put("session_identity",NativeRecoveryReadonlySnapshot.observationId(session));
     row.put("inventory_observer_id",NativeRecoveryReadonlySnapshot.observationId(inventory));row.put("actor_observer_id",NativeRecoveryReadonlySnapshot.observationId(actor));row.put("item_id",original[5]);
     row.put("original_item_id",current);row.put("original_template_observer_id",NativeRecoveryReadonlySnapshot.observationId(originalTemplate));
     row.put("argument_kind",method.equals("setItem")?"retained_actual_template":"retained_actual_template_item_id");
     row.put("argument_template_observer_id",method.equals("setItem")?NativeRecoveryReadonlySnapshot.observationId(template):null);row.put("argument_item_id",alternative);
     row.put("alternate_item_id",value[4]);row.put("alternate_actor_observer_id",NativeRecoveryReadonlySnapshot.observationId(alternativeActor));
     row.put("alternate_template_observer_id",NativeRecoveryReadonlySnapshot.observationId(template));
     row.put("source_qualified_character",owner.source);row.put("source_qualified_account",owner.account);
     row.put("positive_guard_at_capture",true);row.put("owner_retired_before_attempt",true);row.put("both_actors_attached_before_attempt",true);
     row.put("original_session_futures_completed_before_probe",true);row.put("attempted_real_setter",true);row.put("argument_differs_from_original",true);
     row.put("rejection_class",failure==null?null:failure.getClass().getName());row.put("rejection",rejection);row.put("before_state",before);row.put("after_state",after);row.put("source_observation",original[6]);
     row.put("full_native_owner_qualified",false);row.put("full_attribute_matrix_qualified",false);
     proofs.write(NativeRecoveryReadonlySnapshot.json(row)+"\n");proofs.flush();attempts++;
     if(failure!=null&&(failure.getClass()!=IllegalStateException.class||!"source item owner is stale".equals(rejection)))throw failure;
     require(failure!=null&&before.equals(after),"retired morph accepted or changed observed state");
    }
   }
  }
  require(attempts==8,"two morph setters per four original admissions required");return attempts;
 }
