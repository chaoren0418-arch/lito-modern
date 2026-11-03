
#[cfg(test)] mod shape67_controller_development_tests {
    use super::*;
    fn ready()->LoginController {
        let mut c=LoginController::default();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"] {let h=f["packets"][key].as_str().unwrap();c.state.world.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}
        c.state.phase=LoginPhase::BattlePending;c.state.game_connected=true;c.state.selected_source=Some("isolated".into());c.state.selected_character=Some("fixture".into());
        c.state.inventory.owner=Some(super::super::native_inventory::Owner{connection:c.transport.snapshot().epoch,selection:1,character:"fixture".into()});c.state.world.hp=Some([1,100]);c.state.world.learned_skills.clear();c
    }
    fn open(c:&mut LoginController) {let mut b=vec![42];b.extend(c.state.world.own_actor().unwrap().id.to_le_bytes());b.extend(b"monlist\0\0\0\0");c.accept_game(&b).unwrap();}
    #[test] fn noncaster_receives_own_menu_without_rental_and_local_close_or_interrupt_retires() {
        let mut c=ready();open(&mut c);let t=c.shape_prompt().unwrap().token();assert!(c.state.merchant.dialog.is_none());c.dismiss_shape(&t).unwrap();assert!(c.shape_prompt().is_none());open(&mut c);let t=c.shape_prompt().unwrap().token();c.interrupt_world_actions();assert!(c.shape_prompt().is_none());assert!(c.dismiss_shape(&t).is_err());
    }
    #[test] fn authoritative_appearance_invalidates_old_menu_while_effect_alone_does_not_ack() {
        let mut c=ready();open(&mut c);let old=c.shape_prompt().unwrap().token();let own=c.state.world.own_actor().unwrap().clone();let mut effect=vec![55];effect.extend(own.id.to_le_bytes());effect.extend(230_u16.to_le_bytes());c.accept_game(&effect).unwrap();assert_eq!(c.shape_prompt().unwrap().token(),old);
        let hp=c.state.world.hp;let mut body=vec![39];body.extend(own.id.to_le_bytes());body.extend(own.graphics.to_le_bytes());body.push(own.walk_action);c.accept_game(&body).unwrap();assert!(c.shape_prompt().is_none());assert!(c.dismiss_shape(&old).is_err());assert_eq!(c.state.world.hp,hp);open(&mut c);assert_ne!(c.shape_prompt().unwrap().token(),old);
    }
}
