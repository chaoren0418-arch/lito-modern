//! Read original candidate SPR members, never launch or apply game support.
use lito_modern::modern_lito::{
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    sprite::{decode_world_spr_with_limits, SpriteLimits},
    sprite_actions::{ActionKey, ActionLimits, SpriteActionCatalog},
};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,path::Path};
fn digest(bytes:&[u8])->String {format!("{:X}",Sha256::digest(bytes))}
fn main() {
    let root=Path::new(r"D:\lineage開發\0-客戶端");
    let bytes=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let catalog=SpriteActionCatalog::parse_for_inspection(&bytes,ActionLimits::default()).unwrap();
    let mut base=SharedArchive::open(root,SharedArchiveKind::Sprite).unwrap();
    let mut shards=BTreeMap::new();let mut rows=Vec::new();
    let mut encoded_total=0usize;let mut rgba_total=0usize;
    for (skill,sprite) in [(14,2170),(31,10701),(32,2173),(49,759),(52,3936),(54,3104),(72,3934),(78,2234),(206,6527)] {
        let source=&catalog.sprites[&sprite];
        let animation=catalog.resolve_action(sprite,ActionKey::new(0)).unwrap().animation;
        let names:std::collections::BTreeSet<_>=(0..animation.frames.len())
            .map(|i|animation.frame_asset(source.graphics_id,i,0).unwrap().file_name()).collect();
        assert_eq!(names.len(),1,"observed nine single-image definitions");
        let name=names.first().unwrap();
        let shard=(name.bytes().map(u32::from).sum::<u32>()&15) as u8;
        if !shards.contains_key(&shard) {shards.insert(shard,SharedArchive::open(root,SharedArchiveKind::SpriteShard(shard)).unwrap());}
        let archive=match shards[&shard].entry(name) {
            Ok(_)=>shards.get_mut(&shard).unwrap(),
            Err(ResourceError::MissingEntry(_))=>{base.entry(name).unwrap();&mut base},
            Err(e)=>panic!("SPR index error: {e}"),
        };
        let entry=archive.entry(name).unwrap().clone();
        let raw=archive.read_entry(name).unwrap();
        let files=archive.kind().file_names().unwrap();
        let decoded=match decode_world_spr_with_limits(&raw,SpriteLimits::default()) {
            Ok(v)=>v,
            Err(error)=>{
                rows.push(json!({"skill_id":skill,"catalog_sprite_id":sprite,"header_graphics_bank":source.graphics_id,
                    "header_bank_binding_is_candidate":true,"native_caller_bank_binding_verified":false,
                    "member":name,"index_file":files.0,"payload_file":files.1,
                    "entry_offset":entry.offset,"entry_size":entry.size,
                    "encoded_bytes":raw.len(),"encoded_sha256":digest(&raw),
                    "encoded_prefix":&raw[..raw.len().min(96)],"decode_error":error.to_string(),
                    "asset_pixels_decoded":false,"pixel_equivalence_verified":false}));
                continue;
            }
        };
        let rgba_bytes=decoded.frames.iter().map(|f|f.rgba.len()).sum::<usize>();
        encoded_total+=raw.len();rgba_total+=rgba_bytes;
        assert!(encoded_total<=32<<20 && rgba_total<=64<<20);
        let mut frames=Vec::<Value>::new();
        for (index,f) in animation.frames.iter().enumerate() {
            let pixels=decoded.frames.get(f.subframe as usize).expect("referenced original subframe");
            frames.push(json!({"source_frame":index,"subframe":f.subframe,"duration_ticks":f.duration_ticks,
                "events":f.events.iter().map(|e|format!("{e:?}")).collect::<Vec<_>>(),
                "offset":[pixels.x_offset,pixels.y_offset],"size":[pixels.width,pixels.height],
                "RGBA_bytes":pixels.rgba.len(),"RGBA_sha256":digest(&pixels.rgba),
                "nontransparent_pixels":pixels.rgba.chunks_exact(4).filter(|p|p[3]!=0).count()}));
        }
        rows.push(json!({"skill_id":skill,"catalog_sprite_id":sprite,"header_graphics_bank":source.graphics_id,
            "header_bank_binding_is_candidate":true,"native_caller_bank_binding_verified":false,
            "member":name,"index_file":files.0,"payload_file":files.1,
            "entry_offset":entry.offset,"entry_size":entry.size,"encoded_bytes":raw.len(),"encoded_sha256":digest(&raw),
            "decoded_frames":decoded.frames.len(),"decoded_RGBA_bytes":rgba_bytes,
            "frames":frames,"asset_pixels_decoded":true,"pixel_equivalence_verified":false}));
    }
    println!("SELECTOR_SPR_READINESS={}",json!({"list_spr_sha256":digest(&bytes),"rows":rows,
        "encoded_total_bytes":encoded_total,"decoded_total_RGBA_bytes":rgba_total,
        "native_caller_bank_binding_verified":false,"native_skill_support_added":false,
        "actual_new_sessions":0,"actual_new_casts":0,"GUI_verified":false}));
}
