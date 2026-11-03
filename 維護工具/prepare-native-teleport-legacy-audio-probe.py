"""Prepare a distinct probe for scoped legacy events; preserve prior failed probes."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
old=ROOT/'維護工具/candidates/native-teleport-action-audio/probe.rs'
s=old.read_text('utf-8-sig')
s=s.replace('//! Diagnostic wave-request extension; all normal module bodies remain unchanged.',
    '//! Candidate legacy-scope overlays plus a separately pinned wave-request extension.')
s=s.replace('//! Current production Presentation across real C02 service handoff; diagnostic only.',
    '//! Candidate Presentation across real C02 handoff; normal product integration is unqualified.')
assert s.count('held: Vec<m::native_audio_delivery::PreparedCue>')==1
s=s.replace('held: Vec<m::native_audio_delivery::PreparedCue>','held: Vec<m::native_audio_delivery::legacy::PreparedLegacyCue>')
at=s.index('struct Peer {')
trait='''impl m::native_audio_delivery::legacy::LegacyPlayer for Sink {
    fn play_legacy(&mut self, sound: &m::combat_audio::PreparedSound)
        -> std::result::Result<m::combat_audio::PlayOutcome,m::combat_audio::PoolError> {
        self.played.push(sound.sound_id());
        Ok(m::combat_audio::PlayOutcome::Started{sound_id:sound.sound_id(),slot:0})
    }
}
'''
s=s[:at]+trait+s[at:]
start=s.index('                let source_effect =');end=s.index('                let outcome = cue.submit',start)
s=s[:start]+s[end:]
s=s.replace('.filter(|row| row["prepared"] == true)', '.filter(|row| row["prepared"] == true && row["native_group"] == true)')
needle='''        if let Some(frame) = self.view.frame.as_ref() {
'''
assert s.count(needle)==1
loop='''            ensure!(frame.sounds.len()==frame.legacy_audio.len(),"legacy frame payload alignment");
            let own=s.world.status.context("legacy audio owner")?.object_id;
            for cue in &frame.legacy_audio {
                let event=cue.event().context("published legacy event provenance")?;
                let sound=cue.sound();
                let row=json!({"event":event,"prepared":true,"delivery_kind":"legacy_scoped",
                    "bytes":sound.file_bytes(),"sha256":sound.sha256().iter().map(|b|format!("{b:02x}")).collect::<String>()});
                if self.defer_audio && event.actor==own && event.action==68 && event.sound==Some(1534)
                    && sound.sound_id()==1534 {
                    if self.held.is_empty() {
                        self.held.push(cue.clone());
                        self.held_meta=json!({"epoch":epoch,"generation":s.world.generation,
                            "row":row,"world_actions":s.world.actors[&own].actions,
                            "own_casts":s.world.own_cast_packets,
                            "delivered109_before":self.sink.played.iter().filter(|&&id|id==109).count()});
                    }
                } else {
                    let outcome=cue.submit(&scope,&mut self.sink)?;
                    ensure!(outcome!=m::native_audio_delivery::DeliveryOutcome::Unbound,
                        "published legacy cue unbound");
                }
            }
'''
s=s.replace(needle,needle+loop)
out=ROOT/'維護工具/candidates/native-teleport-legacy-audio';out.mkdir()
(out/'probe.rs').write_text(s,encoding='utf-8')
api=old.parent/'diagnostic_wave_api.rs';(out/'diagnostic_wave_api.rs').write_bytes(api.read_bytes())
overlay=ROOT/'維護工具/candidates/native-legacy-audio-lifecycle/20261018-095648-877889/preparation.json'
record=dict(status='prepared_uncompiled_legacy_scope_C1F_S20_C72_probe',original_failed_probe=dict(path=str(old),sha256=sha(old)),
    normal_baseline_source_manifest_sha256='4E1F7A4E02DA1201F2254A959A4130D5038C6F0A9393449D4EB30F8AE2883ED4',
    candidate_probe_sha256=sha(out/'probe.rs'),diagnostic_API_sha256=sha(out/'diagnostic_wave_api.rs'),
    source_overlay_manifest=dict(path=str(overlay),sha256=sha(overlay)),candidate_source_overlays_used=True,
    normal_source_bodies_unchanged=False,normal_product_source_modified=False,
    compiled=False,executed=False,new_sessions=0,GUI_verified=False,physical_audio_verified=False,
    Debug_source_readiness_failure_still_pending=True,whole_goal_complete=False,
    pending=['Compile an executable diagnostic library from the four pinned overlays plus the pinned C1F API extension; record an effective candidate identity distinct from the normal baseline.',
        'Compile this probe against that library and execute the original C72 cross-endpoint scenarios on both profiles without changing phase deadlines.',
        'Read actual C1F/S20 provenance, held legacy1534 rejection/duplicate rejection and native destination109 delivery independently.',
        'Qualify environment deferral/cancel/focus integration, then apply and revalidate the final normal products; current normal and GUI scope remains unqualified.'])
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(candidate=str(out),compiled=False,normal_product_source_modified=False),ensure_ascii=False))
