use lito_modern::modern_lito::{
    combat_animation::classify_native_cues,
    combat_audio::{BankLimits, SoundBank},
    resources::{SharedArchive, SharedArchiveKind},
    sprite_actions::{ActionKey, ActionLimits, FrameEvent, SpriteActionCatalog},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::atomic::AtomicBool;

fn event(event: &FrameEvent) -> Value {
    match event {
        FrameEvent::Hit => json!({"kind":"Hit"}),
        FrameEvent::Sound(id) => json!({"kind":"Sound","id":id}),
        FrameEvent::Overlay(id) => json!({"kind":"Overlay","id":id}),
        FrameEvent::Effect(id) => json!({"kind":"Effect","id":id}),
        FrameEvent::UninterpretedNumeric { marker, value } => json!({"kind":"UninterpretedNumeric","marker":marker,"value":value,"playback_verified":false}),
        FrameEvent::Skip => json!({"kind":"Skip","playback_verified":false}),
    }
}

fn classification(events: &[FrameEvent]) -> Value {
    match classify_native_cues(events) {
        Ok(value) => json!({"audio":value.audio.as_ref().map(|g|json!({"mode":format!("{:?}",g.mode),"candidates":g.candidates()})),
            "non_audio":value.non_audio.iter().map(|e|format!("{e:?}")).collect::<Vec<_>>() }),
        Err(error) => json!({"error":error.to_string()}),
    }
}

fn main() {
    let bytes = SharedArchive::open(Path::new(r"D:\lineage開發\0-客戶端"), SharedArchiveKind::Text)
        .unwrap().read_entry("list.spr").unwrap();
    let digest: String = Sha256::digest(&bytes).iter().map(|b|format!("{b:02X}")).collect();
    let catalog = SpriteActionCatalog::parse_for_inspection(&bytes, ActionLimits::default()).unwrap();
    let mut rows = Vec::new();
    let mut groups = Vec::new();
    let mut classification_errors = Vec::new();
    for (skill, sprite) in [(14,2170),(31,10701),(32,2173),(49,759),(52,3936),(54,3104),(72,3934),(78,2234),(206,6527)] {
        let mut row = json!({"skill_id":skill,"catalog_sprite_id":sprite,"native_request_implemented_by_probe":false,
            "native_caller_bank_binding_verified":false,"asset_pixels_decoded":false,"actual_cast_verified":false});
        if let Some(source) = catalog.sprites.get(&sprite) {
            row["header_graphics_bank"] = json!(source.graphics_id);
            row["raw_header"] = json!(source.header_bytes);
            row["header_issue"] = json!(source.header_issue.as_ref().map(|e|format!("{e:?}")));
            row["attributes"] = json!(source.attributes.iter().map(|a|json!({"key":format!("{:?}",a.key),"label_bytes":a.label_bytes,"body_bytes":a.body_bytes})).collect::<Vec<_>>());
            match catalog.resolve_action(sprite, ActionKey::new(0)) {
                Ok(resolved) => {
                    let animation = resolved.animation;
                    for frame in &animation.frames {
                        match classify_native_cues(&frame.events) {
                            Ok(value) => if let Some(group) = value.audio {
                                if !groups.contains(&group) { groups.push(group); }
                            },
                            Err(error) => classification_errors.push(error.to_string()),
                        }
                    }
                    row["definition_sprite_id"] = json!(resolved.definition_sprite_id);
                    row["frames_per_second"] = json!(animation.frames_per_second);
                    row["directionality"] = json!(format!("{:?}",animation.directionality));
                    row["has_uninterpreted_events"] = json!(animation.has_uninterpreted_events());
                    row["total_ticks"] = json!(animation.frames.iter().map(|f|u64::from(f.duration_ticks)).sum::<u64>());
                    row["frames"] = json!(animation.frames.iter().enumerate().map(|(index,f)|json!({"index":index,
                        "image_id":f.image_id,"subframe":f.subframe,"duration_ticks":f.duration_ticks,
                        "events":f.events.iter().map(event).collect::<Vec<_>>(),
                        "existing_native_cue_classification":classification(&f.events),
                        "header_bank_asset_candidate":animation.frame_asset(source.graphics_id,index,0).map(|a|a.file_name()).ok()})).collect::<Vec<_>>());
                }
                Err(error) => row["resolution_error"] = json!(format!("{error:?}")),
            }
        } else {
            row["missing_catalog_sprite"] = json!(true);
        }
        rows.push(row);
    }
    let prepared_sound_bank = if classification_errors.is_empty() {
        match SoundBank::load_shared(Path::new(r"D:\lineage開發\0-客戶端"), &groups, BankLimits::default(), &AtomicBool::new(false)) {
            Ok(bank) => json!({"loaded":true,"group_count":groups.len(),"sound_count":bank.len(),
                "file_bytes":bank.file_bytes(),"pcm_bytes":bank.pcm_bytes(),
                "sounds":bank.ids().map(|id| {
                    let sound = bank.get(id).unwrap();
                    let format = sound.format();
                    json!({"id":id,"source_wav_sha256":sound.sha256().iter().map(|b|format!("{b:02X}")).collect::<String>(),
                        "pcm_sha256":Sha256::digest(sound.pcm_bytes()).iter().map(|b|format!("{b:02X}")).collect::<String>(),
                        "file_bytes":sound.file_bytes(),"pcm_bytes":sound.pcm_bytes().len(),
                        "channels":format.channels,"sample_rate":format.sample_rate,"bits_per_sample":format.bits_per_sample,
                        "block_align":format.block_align})
                }).collect::<Vec<_>>(),"audio_device_opened":false,"sound_played":false}),
            Err(error) => json!({"loaded":false,"error":error.to_string(),"audio_device_opened":false,"sound_played":false}),
        }
    } else {
        json!({"loaded":false,"classification_errors":classification_errors,"audio_device_opened":false,"sound_played":false})
    };
    println!("SELECTOR_ONLY_CATALOG={}",json!({"list_spr_sha256":digest,"list_spr_bytes":bytes.len(),"catalog_header":catalog.header,"rows":rows,
        "prepared_sound_bank":prepared_sound_bank,
        "scope":"Read-only original catalog inspection; header bank candidates do not prove native caller bank selection or pixel equivalence.",
        "actual_new_sessions":0,"actual_new_casts":0,"GUI_verified":false}));
}
