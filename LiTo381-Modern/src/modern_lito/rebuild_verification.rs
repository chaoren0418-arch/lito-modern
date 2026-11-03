//! Current same-executable, offline resource/input verification. No SQL or sockets.
use super::{compositor::Viewport, scene_model::SceneModel};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{fs::File, path::Path, sync::atomic::AtomicBool, time::Duration};

pub fn run(root: &Path, output: &Path) -> Result<serde_json::Value> {
    ensure!(root.is_absolute(), "resource root must be absolute");
    ensure!(output.is_absolute(), "diagnostic output must be absolute");
    let mut model = SceneModel::load(root, 0, &AtomicBool::new(false), |_| {})?;
    let before = Sha256::digest(model.canvas().rgba());
    // Same inverse transform and placement method as WM_LBUTTONDOWN; 150% view.
    let view = Viewport::fit(1200, 900, 800, 600).context("viewport")?;
    let point = view.canvas_point(900, 600, 800, 600).context("input hit")?;
    ensure!(point == (600, 400), "scaled input mapping");
    ensure!(view.canvas_point(1200, 900, 800, 600).is_none(), "edge must miss");
    model.place_actor(point.0, point.1)?;
    ensure!(model.advance(Duration::ZERO)?.changed, "placement must invalidate pixels");
    ensure!(Sha256::digest(model.canvas().rgba()) != before, "placement must change frame");
    ensure!(model.place_actor(-1, 400).is_err(), "negative placement must reject");
    ensure!(model.actor_anchor() == (600, 400), "rejected input must not mutate");
    model.trigger_action(Duration::ZERO)?;
    model.advance(Duration::ZERO)?;
    let sound = model.advance(Duration::from_millis(375))?;
    ensure!(sound.sounds == [1539], "source animation sound cue");
    ensure!(model.advance(Duration::from_millis(375))?.sounds.is_empty(), "cue cannot repeat");
    model.advance(Duration::from_millis(600))?;
    ensure!(model.hit_cues == 1, "source animation hit cue");
    let mut existing = output;
    while !existing.try_exists()? {
        existing = existing.parent().context("diagnostic output has no existing ancestor")?;
    }
    super::resources::reject_reparse_chain(existing)?;
    std::fs::create_dir_all(output).context("create diagnostic output directory")?;
    super::resources::reject_reparse_chain(output)?;
    let png_path = output.join("resource-input-slice.png");
    if png_path.try_exists()? { super::resources::reject_reparse_chain(&png_path)?; }
    let mut encoder = png::Encoder::new(File::create(&png_path)?, 800, 600);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(model.canvas().rgba())?;
    let png_rgba_sha256 = format!("{:x}", Sha256::digest(model.canvas().rgba()));
    model.advance(Duration::from_secs(60))?;
    ensure!(model.status().contains("Idle"), "action must return to idle");
    Ok(serde_json::json!({
        "schema": 1, "status": "offline_slice_verified", "pointer_bits": usize::BITS,
        "resource_root": root, "canvas": [800,600], "graphics_id": model.graphics_id,
        "input_scale": 1.5, "actor_anchor": model.actor_anchor(),
        "sound_requests": model.sound_requests, "hit_cues": model.hit_cues,
        "png": png_path, "png_rgba_sha256": png_rgba_sha256,
        "final_idle_frame_sha256": format!("{:x}", Sha256::digest(model.canvas().rgba())),
        "native_bin_started": false, "network_connected": false, "playable_game": false
    }))
}

/// Compare existing native RX/TX codecs with vectors produced by the current
/// deployed server CipherState. Does not install a gameplay owner or connect.
pub fn verify_formal_cipher(raw: &[u8]) -> Result<usize> {
    use super::native_city_runtime::{source_send::{Sender,WriteOutcome,Progress}, source_stream::{Receiver,Frame,SEED_HIGH_WORD}};
    #[derive(serde::Deserialize)] struct Packet { plain:String,cipher:String,before:String,after:String }
    #[derive(serde::Deserialize)] struct Sequence { seed:u32,packets:Vec<Packet> }
    #[derive(serde::Deserialize)] struct Vectors { schema:u32,jar_sha256:String,sequences:Vec<Sequence> }
    fn hex(s:&str) -> Result<Vec<u8>> {
        ensure!(s.is_ascii() && s.len()%2==0, "invalid vector hex");
        (0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).map_err(Into::into)).collect()
    }
    ensure!(raw.len() <= 32*1024, "vector budget");
    let data:Vectors=serde_json::from_slice(raw)?;
    ensure!(data.schema==1 && data.jar_sha256=="bbc59308c43df89ee5029d3b3c802be8ad55fee65c41faca99a76912dd5dce43", "formal JAR identity");
    ensure!(data.sequences.len()==4, "four independent seeds required");
    let mut verified=0;
    for seq in data.sequences {
        let seed=(u64::from(SEED_HIGH_WORD)<<32)|u64::from(seq.seed);
        let mut sender=Sender::default(); sender.seed_from_source(seed)?;
        let mut receiver=Receiver::default();
        let mut handshake=vec![7,0,1]; handshake.extend_from_slice(&seq.seed.to_le_bytes());
        let (used,result)=receiver.next(&handshake);
        ensure!(used==7 && matches!(result,Ok(Some(Frame::Seed{counter})) if counter==seed), "source handshake");
        ensure!(seq.packets.len()==6, "six sequential packets required");
        for packet in seq.packets {
            let plain=hex(&packet.plain)?; let cipher=hex(&packet.cipher)?;
            let before:u64=packet.before.parse()?; let after:u64=packet.after.parse()?;
            ensure!(sender.snapshot().counter==Some(before), "TX before counter");
            let reservation=sender.commit(sender.prepare_fixture_body(plain.clone())?);
            let initial=sender.view(&reservation.ticket)?;
            let mut frame=(u16::try_from(cipher.len()+2)?).to_le_bytes().to_vec(); frame.extend_from_slice(&cipher);
            ensure!(initial.bytes()?==frame, "formal Java/Rust ciphertext mismatch");
            // Force writes into three-byte chunks; no re-encryption/re-key.
            while sender.snapshot().pending.is_some() {
                let view=sender.view(&reservation.ticket)?;
                let n=view.bytes()?.len().min(3);
                let progress=sender.report(&reservation.ticket,view.offset(),WriteOutcome::Written(n))?;
                ensure!(matches!(progress,Progress::Partial{..}|Progress::Complete), "partial write state");
            }
            ensure!(initial.bytes().is_err() && sender.snapshot().counter==Some(after), "completed TX lifecycle");
            let mut decoded=None;
            for byte in frame {
                let (used,result)=receiver.next(&[byte]);
                ensure!(used==1,"fragment consumed once");
                match result.map_err(|e|anyhow::anyhow!("RX terminal: {e:?}"))? {
                    Some(Frame::Body{bytes,step})=>{
                        ensure!(decoded.is_none()&&bytes==plain&&step.before==before&&step.after==after,"formal RX mismatch");
                        decoded=Some(bytes);
                    },None=>{},_=>anyhow::bail!("unexpected repeated handshake")
                }
            }
            ensure!(decoded.is_some(),"fragmented frame completed");
            verified+=1;
        }
    }
    Ok(verified)
}
