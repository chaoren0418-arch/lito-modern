use lito_modern::modern_lito::native_inventory::{Inventory, MAX_ITEMS};
fn p(key: &str) -> Vec<u8> {
    let f: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-inventory-current.json")).unwrap();
    let s = f["packets"][key].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn inventory() -> Inventory {
    let mut i = Inventory::default();
    i.begin(2, "測試角色").unwrap();
    i
}
const ID: u32 = 0xfedcba98;
#[test]
fn formal_records_and_all_deltas_keep_unsigned_fields_and_opaque_details() {
    let mut i = inventory();
    i.accept(2, &p("batch")).unwrap();
    let x = &i.items[&ID];
    assert_eq!(
        (
            x.id,
            x.use_type,
            x.charge,
            x.icon,
            x.blessing,
            x.count,
            x.identified
        ),
        (ID, 255, 254, 65535, 3, u32::MAX, 255)
    );
    assert_eq!(x.name, "魔法寶石");
    assert_eq!(x.detail, [0, 1, 255]);
    assert_eq!(i.items[&7].count, 2);
    i.accept(2, &p("count")).unwrap();
    assert_eq!(i.items[&ID].count, 123);
    i.accept(2, &p("name")).unwrap();
    assert_eq!(i.items[&ID].name, "已裝備");
    i.accept(2, &p("charge")).unwrap();
    assert_eq!(i.items[&ID].charge, 255);
    i.accept(2, &p("blessing")).unwrap();
    assert_eq!(i.items[&ID].blessing, 131);
    i.accept(2, &p("extended")).unwrap();
    let x = &i.items[&ID];
    assert_eq!(x.name, "更新名稱");
    assert_eq!(x.count, u32::MAX);
    assert_eq!(x.detail, [7, 0, 9]);
    assert_eq!(x.blessing, 131);
    i.accept(2, &p("zero_count")).unwrap();
    assert_eq!(i.items[&ID].count, 0);
    assert_eq!(i.items.len(), 2);
    i.accept(2, &p("remove")).unwrap();
    assert!(!i.items.contains_key(&ID));
    for k in ["count", "name", "charge", "blessing", "extended", "remove"] {
        i.accept(2, &p(k)).unwrap();
    }
    assert_eq!(i.orphan_updates, 6);
    assert_eq!(i.items.len(), 1);
}
#[test]
fn every_truncation_and_trailing_field_is_rejected_atomically() {
    let mut i = inventory();
    i.accept(2, &p("batch")).unwrap();
    let before = i.clone();
    for key in [
        "add",
        "batch",
        "empty",
        "replace",
        "duplicate",
        "remove",
        "count",
        "zero_count",
        "name",
        "charge",
        "blessing",
        "extended",
        "limits",
        "formatting",
        "blank_name",
        "max_batch",
    ] {
        let b = p(key);
        for n in 0..b.len() {
            assert!(i.accept(2, &b[..n]).is_err(), "{key} {n}");
            assert_eq!(i, before);
        }
        let mut b = b;
        b.push(0);
        assert!(i.accept(2, &b).is_err(), "{key}");
        assert_eq!(i, before);
    }
    let mut b = p("add");
    b[1..5].fill(0);
    assert!(i.accept(2, &b).is_err());
    let mut b = p("add");
    b[9] = 4;
    assert!(i.accept(2, &b).is_err());
    let mut b = p("add");
    b[10..14].fill(0);
    assert!(i.accept(2, &b).is_err());
    let mut b = p("blessing");
    b[5] = 127;
    assert!(i.accept(2, &b).is_err());
    assert_eq!(i, before);
}
#[test]
fn batches_add_to_existing_objects_and_replacement_does_not_duplicate_rows() {
    let mut i = inventory();
    i.accept(2, &p("blank_name")).unwrap();
    i.accept(2, &p("batch")).unwrap();
    assert_eq!(i.items.len(), 3);
    i.accept(2, &p("empty")).unwrap();
    assert_eq!(i.items.len(), 3);
    assert_eq!(i.batches, 2);
    i.accept(2, &p("duplicate")).unwrap();
    assert_eq!(i.items.len(), 3);
    assert_eq!(i.items[&ID].name, "替換");
    assert_eq!(i.items[&ID].count, 9);
    i.accept(2, &p("max_batch")).unwrap();
    assert_eq!(i.items.len(), 256);
}
#[test]
fn selected_connection_and_reselection_own_inventory_not_world_generation() {
    let mut i = Inventory::default();
    assert!(i.accept(2, &p("batch")).is_err());
    i.begin(2, "甲").unwrap();
    i.accept(2, &p("batch")).unwrap();
    let before = i.clone();
    assert!(i.accept(1, &p("count")).is_err());
    assert_eq!(i, before);
    for b in [[0x28, 0x34, 2, 0].as_slice(), [0xd2].as_slice()] {
        assert!(!i.accept(2, b).unwrap());
        assert_eq!(i, before);
    }
    let old = i.owner.clone().unwrap();
    i.begin(2, "乙").unwrap();
    assert!(i.items.is_empty());
    assert_eq!(i.batches, 0);
    assert_ne!(i.owner.as_ref().unwrap().selection, old.selection);
    i.accept(2, &p("remove")).unwrap();
    assert_eq!(i.orphan_updates, 1);
    assert!(i.items.is_empty());
    i.accept(2, &p("batch")).unwrap();
    i.clear();
    assert!(i.owner.is_none());
    assert!(i.items.is_empty());
    assert!(i.accept(2, &p("count")).is_err());
    i.begin(3, "甲").unwrap();
    assert!(i.accept(2, &p("add")).is_err());
}
#[test]
fn resource_limits_allow_maximum_wire_fields_but_bound_inventory_storage() {
    let mut i = inventory();
    i.accept(2, &p("limits")).unwrap();
    let x = &i.items[&u32::MAX];
    assert_eq!(x.name.len(), 255);
    assert_eq!(x.detail.len(), 255);
    let item = x.clone();
    i.items.clear();
    for id in 1..=MAX_ITEMS as u32 {
        let mut x = item.clone();
        x.id = id;
        i.items.insert(id, x);
    }
    let before = i.clone();
    assert!(i.accept(2, &p("add")).is_err());
    assert_eq!(i, before);
    let mut replacement = p("replace");
    replacement[1..5].copy_from_slice(&1u32.to_le_bytes());
    i.accept(2, &replacement).unwrap();
    assert_eq!(i.items.len(), MAX_ITEMS);
    assert_eq!(i.items[&1].name, "替換");
    let before = i.clone();
    i.revision = u64::MAX;
    let at_limit = i.clone();
    assert!(i.accept(2, &replacement).is_err());
    assert_eq!(i, at_limit);
    i = before;
    let mut b = p("limits");
    b.splice(15..15, std::iter::once(b'a'));
    assert!(i.accept(2, &b).is_err());
}
#[test]
fn labels_preserve_authoritative_names_and_do_not_expose_control_characters() {
    let mut i = inventory();
    i.accept(2, &p("formatting")).unwrap();
    i.accept(2, &p("blank_name")).unwrap();
    assert_eq!(i.items[&8].name, "\u{000c}Y物品\t");
    assert!(!i.items[&8].label().chars().any(char::is_control));
    assert_eq!(i.items[&9].label(), "未命名物品 × 1");
}

#[test]
fn same_character_resume_without_replay_keeps_authority_and_rejected_switch_rolls_back() {
    let mut i = inventory();
    i.confirm_selection();
    i.accept(2, &p("batch")).unwrap();
    let first = i.owner.clone().unwrap();
    i.suspend();
    assert!(i.owner.is_none() && i.items.is_empty());
    // The selected server character can still receive an inventory update
    // while its client selector is open. It must not become a visible owner.
    i.accept(2, &p("count")).unwrap();
    assert!(i.owner.is_none() && i.items.is_empty());
    i.begin(2, "測試角色").unwrap();
    assert_eq!(i.items[&ID].count, 123);
    assert_eq!(i.batches, 1);
    assert_ne!(i.owner.as_ref().unwrap().selection, first.selection);
    i.confirm_selection();
    i.suspend();
    i.begin(2, "另一角色").unwrap();
    assert!(i.items.is_empty());
    i.reject_selection();
    assert!(i.owner.is_none());
    i.begin(2, "測試角色").unwrap();
    assert_eq!(i.items[&ID].count, 123);
    i.confirm_selection();
    i.suspend();
    i.begin(3, "測試角色").unwrap();
    assert!(i.items.is_empty());
    i.clear();
    i.begin(2, "測試角色").unwrap();
    assert!(i.items.is_empty());
}
