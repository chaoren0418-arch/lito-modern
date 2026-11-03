#[cfg(test)]
mod static2941_queue_development_tests {
    use super::*;
    use super::static_effect_ingress_development_tests::world;
    use crate::modern_lito::{native_static2941::Queue,native_world::World};

    fn add(world:&mut World,queue:&mut Queue,counter:u64)->(IngressReceipt,u64) {
        let mut b=vec![0x37];b.extend(world.own_actor().unwrap().id.to_le_bytes());b.extend(2941u16.to_le_bytes());
        let receipt=IngressReceipt::decoded(9,counter);let dispatched=Instant::now();
        world.accept_received(&b,receipt).unwrap();let sequence=world.effects.last().unwrap().sequence;
        assert!(queue.admit_received(world,9,receipt,dispatched).unwrap());(receipt,sequence)
    }

    #[test]
    fn distinct_decode_dispatch_constructor_enqueue_samples_and_zero_deadline_are_bound() {
        let mut w=world();let mut q=Queue::default();let(_,s)=add(&mut w,&mut q,1);
        let lease=w.static_effect_receipts[&s].scheduled_lease().unwrap();
        assert!(lease.decoded_at()<=lease.dispatched_at());assert!(lease.dispatched_at()<=lease.constructed_at());
        assert!(lease.constructed_at()<=lease.enqueued_at());assert_eq!(lease.deadline(),lease.enqueued_at());
        assert!(lease.current(&w,9));assert!(!lease.current(&w,10));
    }

    #[test]
    fn copied_world_observer_is_revoked_by_controller_drain_without_mutating_the_copy() {
        let mut w=world();let mut q=Queue::default();let(_,s)=add(&mut w,&mut q,1);let old=w.clone();
        let lease=old.static_effect_receipts[&s].scheduled_lease().unwrap().clone();let revision=w.visual_revision;
        let now=Instant::now();let report=q.drain_due(&mut w,9,now).unwrap();
        assert_eq!(report.sampled_at,Some(now));assert_eq!(report.retired_sequences,vec![s]);
        assert_eq!(w.visual_revision,revision+1);assert_eq!(old.visual_revision,revision);
        assert!(!lease.active());assert!(!lease.current(&old,9));
        assert!(q.drain_due(&mut w,9,Instant::now()).unwrap().retired_sequences.is_empty());
    }

    #[test]
    fn equal_or_earlier_deadlines_keep_admission_order_in_the_callback_receipt() {
        let mut w=world();let mut q=Queue::default();let(_,a)=add(&mut w,&mut q,1);let(_,b)=add(&mut w,&mut q,2);
        let ra=w.static_effect_receipts[&a].scheduled_lease().unwrap();let rb=w.static_effect_receipts[&b].scheduled_lease().unwrap();
        assert!(ra.deadline()<=rb.deadline());
        assert_eq!(q.drain_due(&mut w,9,Instant::now()).unwrap().retired_sequences,vec![a,b]);
    }

    #[test]
    fn ring_eviction_revokes_only_removed_observers_and_keeps_queue_bounded() {
        let mut w=world();let mut q=Queue::default();let(_,s)=add(&mut w,&mut q,1);
        let old=w.static_effect_receipts[&s].scheduled_lease().unwrap().clone();
        for n in 2..=130 {add(&mut w,&mut q,n);}
        assert!(!old.active());assert_eq!(w.static_effect_receipts.len(),128);
        assert_eq!(q.drain_due(&mut w,9,Instant::now()).unwrap().retired_sequences.len(),128);
    }

    #[test]
    fn owner_change_and_queue_drop_revoke_old_snapshots() {
        let mut w=world();let mut q=Queue::default();let(_,s)=add(&mut w,&mut q,1);
        let old=w.static_effect_receipts[&s].scheduled_lease().unwrap().clone();q.synchronize(&w,10);assert!(!old.active());
        let mut fresh=world();let mut other=Queue::default();let(_,s)=add(&mut fresh,&mut other,1);
        let dropped=fresh.static_effect_receipts[&s].scheduled_lease().unwrap().clone();drop(other);assert!(!dropped.active());
    }

    #[test]
    fn retired_or_already_scheduled_receipt_cannot_create_another_lifetime() {
        let mut w=world();let mut q=Queue::default();let(receipt,s)=add(&mut w,&mut q,1);
        assert!(!q.admit_received(&mut w,9,receipt,Instant::now()).unwrap());q.retire();
        assert!(!q.admit_received(&mut w,9,receipt,Instant::now()).unwrap());
        assert!(!w.static_effect_receipts[&s].scheduled_lease().unwrap().active());
    }

    #[test]
    fn wrong_connection_is_rejected_before_revoking_the_current_owner() {
        let mut w=world();let mut q=Queue::default();let(receipt,s)=add(&mut w,&mut q,1);
        assert!(q.admit_received(&mut w,10,receipt,Instant::now()).is_err());
        assert!(w.static_effect_receipts[&s].scheduled_lease().unwrap().current(&w,9));
    }

    #[test]
    fn untimed_world_accept_does_not_create_a_controller_clock_or_deadline() {
        let mut w=world();let mut q=Queue::default();let mut b=vec![0x37];
        b.extend(w.own_actor().unwrap().id.to_le_bytes());b.extend(2941u16.to_le_bytes());w.accept(&b).unwrap();
        let receipt=IngressReceipt::decoded(9,1);assert!(!q.admit_received(&mut w,9,receipt,Instant::now()).unwrap());
        assert!(q.drain_due(&mut w,9,Instant::now()).unwrap().retired_sequences.is_empty());
    }
}
