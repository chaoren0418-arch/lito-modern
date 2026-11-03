#[cfg(test)]
mod static_effect_ingress_development_tests {
    use super::*;
    use crate::modern_lito::{native_world::World, native_skills::Effect};

    fn world() -> World {
        let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        let mut world = World::default();
        for key in ["world", "start", "status", "actor", "bot"] {
            let hex = fixture["packets"][key].as_str().unwrap();
            let bytes = (0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect::<Vec<_>>();
            world.accept(&bytes).unwrap();
        }
        world
    }
    fn packet(world: &World, graphics: u16) -> Vec<u8> {
        let mut body = vec![0x37];
        body.extend(world.own_actor().unwrap().id.to_le_bytes());
        body.extend(graphics.to_le_bytes());
        body
    }

    #[test]
    fn worker_receipt_survives_event_move_and_world_admission_without_resampling() {
        let mut world = world();
        let receipt = IngressReceipt::decoded(9, 31);
        let event = Event::Packet { epoch: 9, body: packet(&world, 2941), rx_counter: 31, receipt };
        let Event::Packet { epoch, body, rx_counter, receipt: moved } = event else { panic!("packet") };
        assert_eq!((moved.connection_epoch(),moved.rx_counter()),(epoch,rx_counter));
        world.accept_received(&body,moved).unwrap();
        let cue = world.effects.last().unwrap();
        let bound = &world.static_effect_receipts[&cue.sequence];
        assert_eq!(bound.ingress(),receipt);
        assert_eq!(bound.ingress().decoded_at(),receipt.decoded_at());
        assert!(bound.matches(world.generation,9,cue));
        assert!(!bound.matches(world.generation,10,cue));
        assert!(!bound.matches(world.generation+1,9,cue));
        let mut wrong: Effect = cue.clone(); wrong.position[0] ^= 1;
        assert!(!bound.matches(world.generation,9,&wrong));
    }

    #[test]
    fn malformed_or_orphan_or_other_graphics_never_borrow_a_previous_static_receipt() {
        let mut world = world();
        world.accept_received(&packet(&world,2941),IngressReceipt::decoded(9,1)).unwrap();
        let before = world.static_effect_receipts.clone();
        let visible = serde_json::to_value(&world).unwrap();
        let mut malformed = packet(&world,2941);malformed.pop();
        assert!(world.accept_received(&malformed,IngressReceipt::decoded(9,2)).is_err());
        assert_eq!(serde_json::to_value(&world).unwrap(),visible);
        assert_eq!(world.static_effect_receipts,before);
        let mut orphan = vec![0x37];orphan.extend(u32::MAX.to_le_bytes());orphan.extend(2941u16.to_le_bytes());
        world.accept_received(&orphan,IngressReceipt::decoded(9,3)).unwrap();
        assert_eq!(world.static_effect_receipts,before);
        world.accept_received(&packet(&world,2942),IngressReceipt::decoded(9,4)).unwrap();
        assert_eq!(world.static_effect_receipts,before);
        assert!(!world.static_effect_receipts.contains_key(&world.effects.last().unwrap().sequence));
    }

    #[test]
    fn ring_eviction_and_world_retirement_remove_receipts_and_untimed_accept_has_no_fallback_clock() {
        let mut world = world();
        for counter in 1..=130 {world.accept_received(&packet(&world,2941),IngressReceipt::decoded(9,counter)).unwrap();}
        assert_eq!(world.effects.len(),128);assert_eq!(world.static_effect_receipts.len(),128);
        assert!(world.static_effect_receipts.keys().all(|s|world.effects.iter().any(|e|e.sequence==*s)));
        world.accept(&packet(&world,2941)).unwrap();
        assert_eq!(world.static_effect_receipts.len(),127);
        assert!(!world.static_effect_receipts.contains_key(&world.effects.last().unwrap().sequence));
        let generation=world.generation;world.clear_preserving_generation();
        assert!(world.static_effect_receipts.is_empty());assert!(world.effects.is_empty());assert_eq!(world.generation,generation);
    }

    #[test]
    fn missing_connection_epoch_rejects_before_world_mutation_and_clock_stays_out_of_json() {
        let mut world = world();let body=packet(&world,2941);
        let before=serde_json::to_value(&world).unwrap();
        assert!(world.accept_received(&body,IngressReceipt::decoded(0,1)).is_err());
        assert_eq!(serde_json::to_value(&world).unwrap(),before);assert!(world.static_effect_receipts.is_empty());
        world.accept_received(&body,IngressReceipt::decoded(9,1)).unwrap();
        assert!(serde_json::to_value(&world).unwrap().get("static_effect_receipts").is_none());
    }
}
