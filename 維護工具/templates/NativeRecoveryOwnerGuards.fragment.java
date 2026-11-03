 // QA guard probes are separate from the read-only snapshot observer. They
 // invoke original authorization checks only, never adopt/retire/mutate items.
 static void rememberOwner(ClientSession session,String observation,
     IdentityHashMap<NativeDeItemApplication.Owner,Object[]> references)throws Exception {
  if(!observation.endsWith(",\"stable_observed\":true,\"full_native_owner_qualification_inferred\":false}"))return;
  Object value=NativePeerHarness.field(session,"sourceItemOwner");
  if(!(value instanceof NativeDeItemApplication.Owner owner)||!owner.active)return;
  long id=NativeRecoveryReadonlySnapshot.observationId(owner);
  if(!observation.contains("\"observer_owner_id\":"+id+",\"active\":true,"))return;
  require(owner.session.get()==session,"observed owner belongs to another session");
  boolean rental=Boolean.TRUE.equals(NativePeerHarness.field(session,"rentalTownRoomState"))
      &&"BATTLE".equals(String.valueOf(NativePeerHarness.field(session,"phase")));
  RuntimeInventory inventory=(RuntimeInventory)NativePeerHarness.field(session,rental?"rentalTownInventory":"inventory");
  if(inventory!=null)references.putIfAbsent(owner,new Object[]{session,inventory,observation});
  require(references.size()<=16,"owner probe reference bound exceeded");
 }
 static String rejectedOwnerGuard(NativeDeItemApplication.Owner owner,RuntimeInventory inventory,String message) {
  try{owner.requireActive(inventory);}
  catch(IllegalStateException error){
   require(error.getClass()==IllegalStateException.class&&message.equals(error.getMessage()),"unexpected owner guard rejection");
   return error.getMessage();
  }
  throw new IllegalStateException("original owner guard unexpectedly accepted inventory");
 }
 static String ownerGuardRow(String kind,NativeDeItemApplication.Owner owner,ClientSession session,
     RuntimeInventory inventory,String observation,String rejection,Long foreignOwner) {
  var row=new LinkedHashMap<String,Object>();
  row.put("schema","G06_E149_owner_guard_v1");row.put("kind",kind);
  row.put("guard_method","NativeDeItemApplication.Owner.requireActive");
  row.put("observer_owner_id",NativeRecoveryReadonlySnapshot.observationId(owner));
  row.put("session_identity",NativeRecoveryReadonlySnapshot.observationId(session));
  row.put("inventory_observer_id",NativeRecoveryReadonlySnapshot.observationId(inventory));
  row.put("source_qualified_character",owner.source);row.put("source_qualified_account",owner.account);
  row.put("owner_active_observed",owner.active);row.put("owner_matches_session",owner.matches(session));
  row.put("rejection",rejection);row.put("foreign_observer_owner_id",foreignOwner);
  row.put("source_observation",observation);row.put("item_mutation_attempted",false);
  row.put("full_native_owner_qualified",false);return NativeRecoveryReadonlySnapshot.json(row);
 }
 static void continuedOwnerGuards(ClientSession session,String observation,BufferedWriter proofs,
     IdentityHashMap<NativeDeItemApplication.Owner,Object[]> references)throws Exception {
  var owner=(NativeDeItemApplication.Owner)NativePeerHarness.field(session,"sourceItemOwner");
  RuntimeInventory inventory=(RuntimeInventory)NativePeerHarness.field(session,"inventory");
  require(owner!=null&&inventory!=null&&owner.active&&owner.session.get()==session,"continued owner absent");
  require(owner.matches(session),"continued owner identity mismatch");
  owner.requireActive(inventory); // positive control before any negative probe
  proofs.write(ownerGuardRow("active_accept",owner,session,inventory,observation,null,null)+"\n");proofs.flush();
  for(var entry:references.entrySet()) {
   NativeDeItemApplication.Owner foreign=entry.getKey();Object[] reference=entry.getValue();
   if(foreign==owner||foreign.account.equals(owner.account))continue;
   ClientSession foreignSession=(ClientSession)reference[0];RuntimeInventory foreignInventory=(RuntimeInventory)reference[1];
   require(!owner.matches(foreignSession),"owner accepted another actual session identity");
   String rejection=rejectedOwnerGuard(owner,foreignInventory,"source item inventory is staged or inactive");
   owner.requireActive(inventory); // same live owner still accepts its own inventory
   proofs.write(ownerGuardRow("foreign_inventory_reject",owner,session,foreignInventory,observation,rejection,
       NativeRecoveryReadonlySnapshot.observationId(foreign))+"\n");proofs.flush();break;
  }
 }
 static void retiredOwnerGuards(BufferedWriter proofs,
     IdentityHashMap<NativeDeItemApplication.Owner,Object[]> references)throws Exception {
  require(references.size()==4,"expected the four actual selected GAME owner admissions");
  for(var entry:references.entrySet()) {
   NativeDeItemApplication.Owner owner=entry.getKey();Object[] reference=entry.getValue();
   ClientSession session=(ClientSession)reference[0];RuntimeInventory inventory=(RuntimeInventory)reference[1];
   require(!owner.active&&Boolean.TRUE.equals(NativePeerHarness.field(session,"sourceItemDisconnected")),"old owner not actually retired/disconnected");
   String rejection=rejectedOwnerGuard(owner,inventory,"source item owner is stale");
   proofs.write(ownerGuardRow("retired_reject",owner,session,inventory,(String)reference[2],rejection,null)+"\n");proofs.flush();
  }
 }
