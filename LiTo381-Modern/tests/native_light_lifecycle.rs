//! Requires the applied candidate World/actor state; excluded from the isolated
//! ten-core shim which deliberately links the unchanged qualified978 library.
use lito_modern::modern_lito::native_world::World;

fn put(id: u32) -> Vec<u8> {
    let mut b = vec![0x0b];
    for value in [32760u16, 32760] {b.extend_from_slice(&value.to_le_bytes());}
    b.extend_from_slice(&id.to_le_bytes()); b.extend_from_slice(&734u16.to_le_bytes());
    b.extend_from_slice(&[0, 0, 5, 0]); // walk, heading, light, speed
    b.extend_from_slice(&0u32.to_le_bytes()); b.extend_from_slice(&0i16.to_le_bytes());
    b.extend_from_slice(b"NativeLightActor\0\0"); b.push(0);
    b.extend_from_slice(&0u32.to_le_bytes()); b.extend_from_slice(&[0, 0]); // clan/master
    b.extend_from_slice(&[0, 255, 0, 1, 0]); // unknown, HP, companion, level, aux name
    b.extend_from_slice(&0u16.to_le_bytes()); b.push(0); b
}
fn action(id: u32, value: u8) -> Vec<u8> {
    let mut b = vec![0x20]; b.extend_from_slice(&id.to_le_bytes()); b.push(value); b
}

#[test]
fn existing_put_preserves_native_death_marker_while_resurrection_and_new_identity_clear_it() {
    let mut world = World::default(); let id = 7u32;
    assert!(world.accept(&[0x28, 0x34, 0x02, 0]).unwrap());
    assert!(world.accept(&[0xd2]).unwrap());
    assert!(world.accept(&put(id)).unwrap());
    assert!(!world.actors[&id].draw_state.native_light_dead);
    world.accept(&action(id, 8)).unwrap();
    assert!(world.actors[&id].dead && world.actors[&id].draw_state.native_light_dead);
    world.accept(&put(id)).unwrap();
    assert!(!world.actors[&id].dead);
    assert!(world.actors[&id].draw_state.native_light_dead);
    let mut resurrect = vec![0x11]; resurrect.extend_from_slice(&id.to_le_bytes());
    resurrect.push(0); resurrect.extend_from_slice(&9u32.to_le_bytes());
    resurrect.extend_from_slice(&734u16.to_le_bytes());
    world.accept(&resurrect).unwrap();
    assert!(!world.actors[&id].draw_state.native_light_dead);
    world.accept(&action(id, 8)).unwrap();
    let mut remove = vec![0x15]; remove.extend_from_slice(&id.to_le_bytes());
    world.accept(&remove).unwrap(); world.accept(&put(id)).unwrap();
    assert!(!world.actors[&id].draw_state.native_light_dead);
}
