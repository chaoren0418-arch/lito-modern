use lito_modern::modern_lito::native_npc_document::{self as d,Action};
#[test]
fn original_chinese_documents_preserve_text_and_received_actions(){
    for (bytes,title) in [(include_bytes!("fixtures/154-love_start-c.html").as_slice(),"勒夫:"),(include_bytes!("fixtures/152-love_aid-c.html").as_slice(),"飾品輔助商人:")]{
        let doc=d::parse(bytes).unwrap();assert!(doc.text.starts_with(title));assert!(doc.text.contains('\n'));assert!(!doc.text.contains('<'));
        assert_eq!(doc.actions,vec![Action{command:"buy".into(),label:"租用道具".into(),navigate:false},Action{command:"sell".into(),label:"退還道具".into(),navigate:false}]);
    }
    assert!(d::parse(include_bytes!("fixtures/154-love_start-c.html")).unwrap().text.contains("有啥需要幫忙嗎?"));
    assert_eq!(d::member("Love_Start").unwrap(),"love_start-c.html");
    for name in ["../Love_Start","Love_Start-k","love_start-c.html","love_start\0","unknown",""]{assert!(d::member(name).is_err());}
    assert_eq!(d::parse(b"<body><p>Hello  world</p></body>").unwrap().text,"Hello world");
}
#[test]
fn truncated_unqualified_and_oversized_markup_never_produces_action_buttons(){
    let bytes=include_bytes!("fixtures/154-love_start-c.html");let end=bytes.windows(7).position(|v|v==b"</body>").unwrap()+7;
    for n in 0..end{assert!(d::parse(&bytes[..n]).is_err(),"prefix {n}");}
    for bad in [b"<body><script>buy</script></body>".as_slice(),b"<body><a href=buy>Buy</a></body>",b"<body><a action=buy action=sell>Buy</a></body>",b"<body><a action=buy><a action=sell>x</a></a></body>",b"<body><a action=buy></a></body>",b"<body><a action=teleport>x</a></body>",b"<body>&bogus;</body>",b"<body>\x81</body>",b"<body>\0</body>",b"<body></body><p>x</p>"]{assert!(d::parse(bad).is_err(),"{bad:?}");}
    assert!(d::parse(&vec![b'a';d::MAX_BYTES+1]).is_err());
    assert!(d::parse(format!("<body>{}</body>","<br>".repeat(257)).as_bytes()).is_err());
}
#[test]
fn original_archive_loader_matches_fixture_hashes_and_obeys_cancellation(){
    use std::{path::Path,sync::atomic::AtomicBool};use sha2::{Digest,Sha256};
    let root=Path::new(r"D:\lineage開發\0-客戶端");let c=d::Catalog::load(root,&AtomicBool::new(false)).unwrap();
    for (name,bytes) in [("love_start-c.html",include_bytes!("fixtures/154-love_start-c.html").as_slice()),("love_aid-c.html",include_bytes!("fixtures/152-love_aid-c.html").as_slice())]{
        let source=c.sources.iter().find(|v|v.member==name).unwrap();assert_eq!(source.bytes,bytes.len());assert_eq!(source.sha256,format!("{:X}",Sha256::digest(bytes)));
    }
    assert!(c.get("Love_Start").is_ok());assert!(c.get("unqualified").is_err());assert!(d::Catalog::load(root,&AtomicBool::new(true)).is_err());
}

#[test]
fn local_page_links_are_distinct_from_wire_requests_and_stay_in_the_catalog(){
    let start=d::parse(include_bytes!("fixtures/322-phan_start-c.html")).unwrap();
    assert_eq!(start.actions.len(),3);assert!(start.actions.iter().all(|a|a.navigate&&d::member(&a.command).is_ok()));
    let citizen=d::parse(include_bytes!("fixtures/312-phan_citizen-c.html")).unwrap();
    assert!(citizen.actions.iter().any(|a|!a.navigate&&a.command=="request LTC light jewel"));
    assert!(citizen.actions.iter().any(|a|a.navigate&&a.command=="Phan_Start"));
    for value in ["../Phan_Start","file:test","https://example.test","unknown"]{
        assert!(d::parse(format!("<body><a link=\"{value}\">open</a></body>").as_bytes()).is_err());
    }
}
#[test]
fn original_hotel_variables_are_plain_text_and_missing_values_fail_closed(){
    use std::{path::Path,sync::atomic::AtomicBool};
    let catalog=d::Catalog::load(Path::new(r"D:\lineage開發\0-客戶端"),&AtomicBool::new(false)).unwrap();
    for name in ["inn","inn2","inn4","inn20"]{
        let doc=catalog.get(name).unwrap();assert!(doc.text_with_args(&[]).is_err());
        let text=doc.text_with_args(&["旅館服務生-席巴".into()]).unwrap();assert!(text.starts_with("旅館服務生-席巴:"));assert!(!text.contains('\u{e000}'));
        assert!(doc.text_with_args(&["<a action=buy>literal</a>".into()]).unwrap().starts_with("<a action=buy>literal</a>:"));
        assert!(doc.text_with_args(&["x\0y".into()]).is_err());
    }
    assert_eq!(catalog.get("inn").unwrap().actions.iter().map(|a|a.command.as_str()).collect::<Vec<_>>(),["room","return","enter"]);
    assert_eq!(d::resolved_arguments("inn",&[],"收到的NPC"),["收到的NPC"]);
    assert_eq!(d::resolved_arguments("inn",&["正式參數".into()],"收到的NPC"),["正式參數"]);
    assert!(d::resolved_arguments("inn2",&[],"收到的NPC").is_empty());
    assert!(d::parse(b"<body><var src=\"#0\">no closing body").is_err());
    for bad in [b"<body><var src=\"#256\"></body>".as_slice(),b"<body><var src=\"file\"></body>",b"<body><a action=room><var src=\"#0\"></a></body>"]{assert!(d::parse(bad).is_err());}
}

#[test]
fn original_teleport_page_keeps_only_source_actions_and_image_references(){
 let bytes=include_bytes!("fixtures/343-teleportme-c.html");let doc=d::parse(bytes).unwrap();
 assert_eq!(d::member("TeleportMe").unwrap(),"teleportme-c.html");
 assert!(doc.text.starts_with("傳送師:"));assert!(doc.text.contains("你想要去哪裡呢？"));
 assert_eq!(doc.actions.iter().map(|a|a.command.as_str()).collect::<Vec<_>>(),["LT_TELEPORT_B","LT_TELEPORT_C"]);
 assert!(doc.actions.iter().all(|a|!a.navigate));assert!(doc.actions[0].label.starts_with("祝福之地"));assert_eq!(doc.actions[1].label,"勇氣之地");assert_eq!(doc.images,[7311,7312]);
 let c=d::Catalog::load(std::path::Path::new(r"D:\lineage開發\0-客戶端"),&std::sync::atomic::AtomicBool::new(false)).unwrap();
 assert_eq!(c.get("TeleportMe").unwrap(),&doc);
 let source=c.sources.iter().find(|s|s.member=="teleportme-c.html").unwrap();
 use sha2::{Digest,Sha256};assert_eq!(source.sha256,format!("{:X}",Sha256::digest(bytes)));assert_eq!(source.bytes,bytes.len());
 for bad in [b"<body><img src=#7313></body>".as_slice(),b"<body><img src=../7311></body>",b"<body><img src=#7311 onclick=buy></body>",b"<body><a action=LT_TELEPORT_A>x</a></body>",b"<body><a action=LT_TELEPORT_D>x</a></body>",b"<body><a action=LT_TELEPORT_B><img src=#7311></a></body>"]{assert!(d::parse(bad).is_err());}
}
