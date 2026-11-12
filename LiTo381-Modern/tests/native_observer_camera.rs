use lito_modern::modern_lito::native_world::World;

    fn packet(key:&str)->Vec<u8>{let v:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let s=v["packets"][key].as_str().unwrap();(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
    fn viewer()->World{let mut w=World::default();w.accept(&[0x28,0x34,2,0]).unwrap();w.accept(&[0xd2]).unwrap();w.accept(&packet("status")).unwrap();w.accept(&[0xe2,0xc0,0x7f,0,0x80,0,0x80,0x40,0x80]).unwrap();w}
    #[test]
    fn observer_camera_bounds_do_not_create_role_or_actor(){let mut w=viewer();assert!(!w.observer_mode&&w.observer_camera.is_none()&&!w.view_ready());w.set_observer_mode(true);assert_eq!(w.observer_camera,Some([32736,32800]));assert!(w.view_ready()&&!w.ready());assert!(w.actors.is_empty());assert!(w.move_request(0).is_err()&&w.attack_request(3).is_err());}
    #[test]
    fn observer_camera_half_open_edges_and_invalid_direction_preserve_authority(){let mut w=viewer();w.set_observer_mode(true);let before=serde_json::to_value(&w.actors).unwrap();for _ in 0..32 {assert!(w.pan_observer(0).unwrap());}assert_eq!(w.observer_camera.unwrap()[1],32768);let revision=w.visual_revision;assert!(!w.pan_observer(0).unwrap());assert_eq!(w.visual_revision,revision);assert!(w.pan_observer(8).is_err());assert_eq!(serde_json::to_value(&w.actors).unwrap(),before);}
    fn fought_then_new_scene()->World{let mut w=World::default();for k in ["world","start","status","actor"]{w.accept(&packet(k)).unwrap();}let own=w.own_actor().map(|a|[a.x,a.y]);assert_eq!(own,Some([32720,32780]));assert_eq!(w.native_anchor_cell(),own);w.accept(&[0x28,0xf4,1,0]).unwrap();w.accept(&[0xd2]).unwrap();assert!(w.actors.is_empty()&&w.own_actor().is_none());assert_eq!(w.native_view_anchor,own);w}
    #[test]
    fn observer_initial_camera_keeps_native_retained_anchor_inside_bounds(){
        // Native sub_4E7A40 is the only producer of the anchor on entry; S_WORLD/S_E2/S_E5 leave it.
        assert_eq!(lito_modern::modern_lito::native_world::NATIVE_CAMERA_ANCHOR_OFFSET,[6,-18]);
        let mut w=fought_then_new_scene();w.set_observer_mode(true);assert!(w.observer_camera.is_none());
        w.accept(&[0xe2,0xc0,0x7f,0,0x80,0,0x80,0x40,0x80]).unwrap();assert_eq!(w.observer_camera,Some([32720,32780]));assert!(w.actors.is_empty());
        let mut v=fought_then_new_scene();v.accept(&[0xe2,0xc0,0x7f,0,0x80,0,0x80,0x40,0x80]).unwrap();assert!(v.observer_camera.is_none());v.set_observer_mode(true);assert_eq!(v.observer_camera,Some([32720,32780]));
        // Local panning is not a native anchor write.
        assert!(v.pan_observer(0).unwrap());assert_eq!(v.native_view_anchor,Some([32720,32780]));
        v.clear_preserving_generation();assert_eq!(v.native_view_anchor,Some([32720,32780]));assert!(v.observer_camera.is_none()&&!v.observer_mode);}
    #[test]
    fn observer_initial_camera_falls_back_to_centre_when_native_anchor_is_outside_bounds(){
        let mut w=fought_then_new_scene();w.set_observer_mode(true);w.accept(&[0xe2,0,0x81,0,0x81,0x40,0x81,0x40,0x81]).unwrap();assert_eq!(w.observer_camera,Some([33056,33056]));assert_eq!(w.native_view_anchor,Some([32720,32780]));
        let mut fresh=viewer();assert!(fresh.native_view_anchor.is_none());fresh.set_observer_mode(true);assert_eq!(fresh.observer_camera,Some([32736,32800]));}
    #[test]
    fn observer_role_change_map_replacement_and_clear_retire_camera(){let mut w=viewer();w.set_observer_mode(true);w.pan_observer(0).unwrap();w.accept(&[0x28,0xf4,1,0]).unwrap();assert!(w.observer_mode&&w.observer_camera.is_none()&&!w.view_ready());w.accept(&[0xd2]).unwrap();w.accept(&[0xe2,0,0x80,0,0x80,0x40,0x80,0x40,0x80]).unwrap();assert_eq!(w.observer_camera,Some([32800,32800]));w.set_observer_mode(false);assert!(w.observer_camera.is_none());w.set_observer_mode(true);let generation=w.generation;w.clear_preserving_generation();assert_eq!(w.generation,generation);assert!(!w.observer_mode&&w.observer_camera.is_none()&&!w.view_ready());}
