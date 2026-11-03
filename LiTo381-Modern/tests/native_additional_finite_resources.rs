//! Resource-only candidate integration. Synthetic S37 inputs are not server/gameplay evidence.
use anyhow::{ensure, Context, Result};
use lito_modern::modern_lito::{native_world::World,native_world_scene::Model};
use serde_json::{json,Value};
use std::{path::{Path,PathBuf},sync::atomic::AtomicBool};

fn current() -> Result<World> {
    let fixture:Value=serde_json::from_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/native-world-current.json")))?;
    let mut world=World::default();world.relations.begin(2)?;
    for key in ["world","start","status","actor","bot","hp","mp"] {
        let text=fixture["packets"][key].as_str().context("fixture packet")?;
        let bytes=(0..text.len()).step_by(2).map(|i|u8::from_str_radix(&text[i..i+2],16)).collect::<std::result::Result<Vec<_>,_>>()?;
        ensure!(world.accept(&bytes)?,"fixture packet ignored");
    }
    ensure!(world.view_ready(),"resource world not ready");Ok(world)
}
fn png(path:&Path,rgba:&[u8])->Result<()> {
    let mut encoder=png::Encoder::new(std::fs::File::create(path)?,800,600);
    encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(rgba)?;Ok(())
}
fn events(receipt:&Value,sequence:u64,sounds:&mut Vec<u32>)->Result<()> {
    for row in receipt["entered_events"].as_array().context("finite events")? {
        if row["sequence"]==sequence {
            if let Some(sound)=row["sound"].as_u64(){sounds.push(u32::try_from(sound)?);}
        }
    }
    Ok(())
}
#[test]
#[ignore="requires original resources and isolated finite-effect output"]
fn candidate_original_finite_bank_draws_and_advances_each_once() -> Result<()> {
    let output=PathBuf::from(std::env::var("LITO_EXTRA_FINITE_RENDER_OUT")?).canonicalize()?;
    ensure!(output.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/native44FE-extra-finite-effects").canonicalize()?),"owned resource output required");
    let spec:Value=serde_json::from_slice(&std::fs::read(output.join("original-effect-definitions.json"))?)?;
    let graphics:Vec<u16>=serde_json::from_slice(&std::fs::read(output.join("candidate-graphics.json"))?)?;
    ensure!(graphics.len()==11,"candidate set changed");
    let cancel=AtomicBool::new(false);let mut world=current()?;
    let own=world.own_actor().context("resource actor")?.id;
    let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&world,&cancel)?;
    let base=model.draw_at(&world,&cancel,0)?;
    ensure!(base.receipt["skill_effects"]["additional_bank"].is_null(),"new bank eagerly loaded");
    let mut now=1_000_000_000u64;let mut cases=Vec::new();
    for gfx in graphics {
        let source=spec["rows"].as_array().context("source definitions")?.iter().find(|r|r["graphics"]==gfx).context("source effect")?;
        let source_frames=source["action0"]["frames"].as_array().context("source frames")?;
        let total_ticks:u64=source_frames.iter().map(|f|f["ticks"].as_u64().unwrap()).sum();
        ensure!(total_ticks<=24*30,"review long finite effect separately");
        let duration=total_ticks.saturating_mul(1_000_000_000)/24+1_000_000_000;
        let mut expected=Vec::new();
        for frame in source_frames {
            for event in frame["events"].as_array().context("source audio events")? {
                let text=event.as_str().context("event text")?;
                expected.push(text.strip_prefix("Sound(").and_then(|s|s.strip_suffix(')')).context("simple finite sound")?.parse::<u32>()?);
            }
        }
        let before=model.draw_at(&world,&cancel,now)?;
        let mut packet=vec![0x37];packet.extend_from_slice(&own.to_le_bytes());packet.extend_from_slice(&gfx.to_le_bytes());
        ensure!(world.accept(&packet)?,"synthetic source effect ignored");
        let sequence=world.effects.last().context("new finite effect")?.sequence;
        let authority=serde_json::to_value(&world)?;
        let mut sounds=Vec::new();let mut first_visible=None;let mut final_receipt=Value::Null;let mut blank_start_without_pixels=false;
        let mut offset=0u64;
        loop {
            let frame=model.draw_at(&world,&cancel,now+offset)?;
            let receipt=&frame.receipt["skill_effects"];
            ensure!(receipt["load_error"].is_null(),"finite bank failed: {}",receipt["load_error"]);
            ensure!(receipt["additional_bank"]["loaded"]==true,"finite bank absent");
            ensure!(receipt["additional_bank"]["load_error"].is_null(),"additional bank error");
            ensure!(receipt["omitted"]==0,"candidate finite effect omitted");
            events(receipt,sequence,&mut sounds)?;
            if gfx==2251&&offset==0 {
                let blank=receipt["effects"].as_array().context("blank source effect")?.iter().find(|e|e["sequence"]==sequence&&e["graphics"]==2251).context("2251 source row")?;
                ensure!(blank["subframe"]==0&&blank["source_frame"]==1&&blank["drawn"]==false&&blank["finished"]==false,"reviewed2251 blank did not retain its active dwell");
                blank_start_without_pixels=true;
            }
            if first_visible.is_none() && receipt["effects"].as_array().context("draw effects")?.iter().any(|e|e["sequence"]==sequence&&e["graphics"]==gfx&&e["drawn"]==true) {
                let path=output.join(format!("effect-{gfx}.png"));png(&path,frame.canvas.rgba())?;
                first_visible=Some(json!({"png":path,"receipt":receipt,"pixels_changed":frame.canvas.rgba()!=before.canvas.rgba()}));
            }
            final_receipt=receipt.clone();
            ensure!(serde_json::to_value(&world)?==authority,"renderer changed authoritative fixture world");
            if offset>=duration {break}
            offset=(offset+100_000_000).min(duration);
        }
        let first_visible=first_visible.context("no visible finite frame")?;
        ensure!(first_visible["pixels_changed"]==true,"finite effect did not change pixels");
        ensure!(final_receipt["active"]==0,"finite effect did not finish");
        let submitted=sounds.len();expected.sort_unstable();sounds.sort_unstable();
        ensure!(sounds==expected,"finite source event count changed for {gfx}: {sounds:?} != {expected:?}");
        if gfx==2251 {ensure!(blank_start_without_pixels&&sounds==[821],"2251 blank lost its once-only source audio");}
        for step in 1..=3 {
            let frame=model.draw_at(&world,&cancel,now+duration+step*100_000_000)?;
            let mut repeated=Vec::new();events(&frame.receipt["skill_effects"],sequence,&mut repeated)?;
            ensure!(repeated.is_empty(),"finished cue re-entered");
        }
        cases.push(json!({"graphics":gfx,"sequence":sequence,"source_sound_events":submitted,"reviewed_blank_start_without_pixels":blank_start_without_pixels,"duration_bound_ns":duration,"first_visible":first_visible,"final_receipt":final_receipt}));
        now+=duration+1_000_000_000;
    }
    // The same map and repeated effect sequence must still have a fresh bank
    // after world retirement. A pre-cancelled transition keeps the old Model.
    let mut fresh=current()?;fresh.generation=world.generation+1;
    let navigation=model.navigation();
    ensure!(model.draw_at(&fresh,&AtomicBool::new(true),0).is_err(),"pre-cancelled transition published");
    ensure!(std::sync::Arc::ptr_eq(&navigation,&model.navigation()),"cancelled transition replaced old navigation");
    let frame=model.draw_at(&world,&cancel,now)?;
    ensure!(frame.receipt["skill_effects"]["additional_bank"]["loaded"]==true,"cancel discarded old effect bank");
    let frame=model.draw_at(&fresh,&cancel,0)?;
    ensure!(frame.receipt["world_generation"]==fresh.generation,"new world not published");
    ensure!(frame.receipt["skill_effects"]["additional_bank"].is_null(),"retired bank survived new generation");
    ensure!(frame.receipt["skill_effects"]["active"]==0,"old finite effect survived new world");
    let mut packet=vec![0x37];packet.extend_from_slice(&own.to_le_bytes());packet.extend_from_slice(&746u16.to_le_bytes());
    ensure!(fresh.accept(&packet)?,"fresh repeated graphics ignored");
    let sequence=fresh.effects.last().context("fresh finite effect")?.sequence;
    ensure!(sequence==cases[0]["sequence"].as_u64().context("old first sequence")?,"fixture did not reuse event identity");
    let authority=serde_json::to_value(&fresh)?;let mut sounds=Vec::new();
    let frame=model.draw_at(&fresh,&cancel,1_000_000_000)?;events(&frame.receipt["skill_effects"],sequence,&mut sounds)?;
    let frame=model.draw_at(&fresh,&cancel,1_200_000_000)?;events(&frame.receipt["skill_effects"],sequence,&mut sounds)?;
    ensure!(sounds==[116],"fresh generation sound not entered exactly once: {sounds:?}");
    let mut repeated=Vec::new();let mut frame=model.draw_at(&fresh,&cancel,10_000_000_000)?;events(&frame.receipt["skill_effects"],sequence,&mut repeated)?;
    // Native positive dwells reschedule from the actual late dispatch. A long
    // gap advances one callback; it must not burst through the remaining clip.
    let late=frame.receipt["skill_effects"]["effects"].as_array().context("late native cue")?.iter().find(|e|e["sequence"]==sequence).context("fresh late cue")?;
    ensure!(late["source_frame"]==3&&late["callback_count"]==3&&late["next_callback_ms"]==9166&&late["finished"]==false,"late native dispatch skipped or shortened dwell");
    let mut previous_ms=9000u64;
    for _ in 0..16 {
        if frame.receipt["skill_effects"]["active"]==0 {break}
        let cue=frame.receipt["skill_effects"]["effects"].as_array().context("native callback row")?.iter().find(|e|e["sequence"]==sequence).context("fresh callback cue")?;
        let due=cue["next_callback_ms"].as_u64().context("next native callback")?;ensure!(due>previous_ms,"native positive dwell did not advance");
        frame=model.draw_at(&fresh,&cancel,1_000_000_000+due*1_000_000)?;events(&frame.receipt["skill_effects"],sequence,&mut repeated)?;previous_ms=due;
    }
    ensure!(repeated.is_empty()&&frame.receipt["skill_effects"]["active"]==0,"fresh effect repeated or failed to finish after its callbacks");
    ensure!(serde_json::to_value(&fresh)?==authority,"new generation renderer changed authority");
    std::fs::write(output.join("resource-result.json"),serde_json::to_vec_pretty(&json!({"status":"candidate_resource_render_and_source_event_progression_passed","cases":cases,
        "lifecycle":{"pre_cancel_preserves_old_model":true,"same_map_new_generation_retires_bank":true,"reused_sequence_fresh_sound_once":true,"fresh_sound":116,"late_native_callback_preserves_dwell":true},
        "normal_product_modified":false,"actual_sessions":0,"actual_casts":0,"synthetic_S37_inputs":true,"fixture_authority_unchanged_by_renderer":true,
        "PCM_decoding_observed_via_bank":true,"audio_device_playback_verified":false,"normal_GUI_verified":false,"gameplay_verified":false,"whole_goal_complete":false}))?)?;
    Ok(())
}
