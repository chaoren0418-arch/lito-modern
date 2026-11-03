    // Mirrors only the original pre-copy reads. Does not call copy, a timer,
    // Future status/cancellation, a game getter, or any gameplay setter.
    private static LinkedHashMap<String,Object> stateCopyPreconditions(Object actor)
            throws ReflectiveOperationException {
        var expected=new LinkedHashMap<String,String>();
        /*EXPECTED_FIELDS*/
        var names=new ArrayList<String>();var rows=new ArrayList<Object>();
        for(var slot:NativeDeWorldKinds.Item.class.getDeclaredFields()) {
            if(java.lang.reflect.Modifier.isStatic(slot.getModifiers()))continue;
            String name=slot.getName(),type=slot.getType().getName();
            if(!type.equals(expected.remove(name)))throw new IllegalStateException("state-copy field manifest differs");
            if(names.size()>=128)throw new IllegalStateException("state-copy declared field scan exceeds bound");
            int index=names.size();names.add(name);slot.setAccessible(true);Object value=slot.get(actor);
            boolean future=value instanceof java.util.concurrent.ScheduledFuture<?>;
            boolean declaredFuture=java.util.concurrent.ScheduledFuture.class.isAssignableFrom(slot.getType());
            boolean lease=slot.getType().getSimpleName().equals("EquipmentTimerLease");
            boolean flag=name.equals("_magicStoneExpirySubmissionInFlight")||name.equals("_magicStoneExpiryRetryScheduled");
            boolean generation=name.equals("_pendingMagicStoneExpiryGeneration");
            if(declaredFuture||future||lease||flag||generation) {
                if(flag&&!(value instanceof Boolean)||generation&&!(value instanceof Long))
                    throw new IllegalStateException("state-copy pending scalar type differs");
                rows.add(object("field",name,"scan_index",index,"declared_type",type,
                    "present",value!=null,"scheduled_future_instance",future,
                    "reference_observer_id",(future||lease)&&value!=null?id(value):null,
                    "pending_value",flag||generation?value:null));
            }
        }
        if(!expected.isEmpty())throw new IllegalStateException("state-copy declared field scan incomplete");
        return object("schema","G06_E149_state_copy_preconditions_v1","actor_observer_id",id(actor),
            "declared_class","local.lito381.server.NativeDeWorldKinds$Item",
            "declared_fields_scanned",names,"entries",rows,"snapshot_is_copy_authorization",false);
    }
