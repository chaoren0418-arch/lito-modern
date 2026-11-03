"""Prepare four source overlays for scoped legacy delivery; never edit the product."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=V/'native-retired-join-normal/validation-20261018-063154-317211';inputs=read(normal/'compiled-inputs.json')
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads')and not queue.get('source_and_corresponding_QA_frozen')
for name,h in inputs.items():assert sha(ROOT/name)==h
component=ROOT/'維護工具/candidates/native-legacy-audio-delivery/legacy_audio_delivery.rs'
qp=V/'native-legacy-audio-delivery/20261018-095326-629336/qualified.json';q=read(qp)
assert q['component_only']and q['recorded_owned_all_terminal']and all(p['passed']==8 and p['failed']==0 for p in q['profiles'].values())
assert sha(component)==q['candidate_pins'][component.name]
out=ROOT/'維護工具/candidates/native-legacy-audio-lifecycle'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
names=['native_audio_delivery.rs','native_world_scene.rs','native_world_view.rs','native_login_window.rs']
original={name:(P/'src/modern_lito'/name).read_bytes()for name in names}
text={name:data.decode('utf-8-sig')for name,data in original.items()}
def replace(name,old,new,count=1):
    assert text[name].count(old)==count,(name,old,text[name].count(old))
    text[name]=text[name].replace(old,new)
text['native_audio_delivery.rs']+='\n// Scoped ordinary WAV payload; voice policy remains play_legacy.\npub mod legacy {\n'+component.read_text('utf-8-sig')+'\n}\n'
n='native_world_scene.rs'
replace(n,'    pub sounds: Vec<super::combat_audio::PreparedSound>,','    pub sounds: Vec<super::combat_audio::PreparedSound>,\n    pub legacy_audio: Vec<super::native_audio_delivery::legacy::PreparedLegacyCue>,')
replace(n,'        let mut sounds = Vec::new();','        let mut sounds = Vec::new();\n        let mut legacy_audio = Vec::new();')
replace(n,'"event":event,"prepared":true,"bytes":sound.file_bytes()','"event":event,"prepared":true,"delivery_kind":"legacy_scoped","bytes":sound.file_bytes()')
replace(n,'audio_receipts.push(row);if !suppressed {sounds.push(sound);}','audio_receipts.push(row);if !suppressed {legacy_audio.push(super::native_audio_delivery::legacy::PreparedLegacyCue::from_event(sound.clone(),event)?);sounds.push(sound);}')
replace(n,'            sounds,\n            native_audio,','            sounds,\n            legacy_audio,\n            native_audio,')
n='native_world_view.rs'
replace(n,'    deferred_sounds: Vec<super::combat_audio::PreparedSound>,','    deferred_sounds: Vec<super::combat_audio::PreparedSound>,\n    deferred_legacy_audio: Vec<super::native_audio_delivery::legacy::PreparedLegacyCue>,')
replace(n,'self.deferred_sounds.clear();','for cue in &self.deferred_legacy_audio {cue.discard();}\n        self.deferred_legacy_audio.clear();\n        self.deferred_sounds.clear();',2)
replace(n,'self.deferred_sounds.append(&mut frame.sounds);','self.deferred_sounds.append(&mut frame.sounds);self.deferred_legacy_audio.append(&mut frame.legacy_audio);')
replace(n,'for cue in &frame.native_audio {cue.discard();}','for cue in &frame.legacy_audio {cue.discard();}for cue in &frame.native_audio {cue.discard();}')
replace(n,'for cue in &done.frame.native_audio {cue.discard();}','for cue in &done.frame.legacy_audio {cue.discard();}for cue in &done.frame.native_audio {cue.discard();}')
replace(n,'let bound=done.frame.native_audio.iter().try_for_each(|cue|cue.bind(scope.clone())).and_then(|_| {',
    'let bound=(||->anyhow::Result<()> {\n                                anyhow::ensure!(done.frame.sounds.len()==done.frame.legacy_audio.len(),"legacy frame payload alignment");\n                                anyhow::ensure!(self.deferred_sounds.len()==self.deferred_legacy_audio.len(),"deferred legacy payload alignment");\n                                for cue in &done.frame.legacy_audio {cue.bind(scope.clone())?;}\n                                Ok(())\n                            })().and_then(|_|done.frame.native_audio.iter().try_for_each(|cue|cue.bind(scope.clone()))).and_then(|_| {')
replace(n,'done.frame.sounds = sounds;','done.frame.sounds = sounds;\n                                    let mut legacy=std::mem::take(&mut self.deferred_legacy_audio);\n                                    legacy.append(&mut done.frame.legacy_audio);done.frame.legacy_audio=legacy;')
replace(n,'self.deferred_sounds.append(&mut done.frame.sounds);','self.deferred_sounds.append(&mut done.frame.sounds);\n                                    self.deferred_legacy_audio.append(&mut done.frame.legacy_audio);')
n='native_login_window.rs'
old='''                if self.audio_error.is_none() && GetForegroundWindow() == parent {
                    for sound in &frame.sounds {
                        if self.audio_error.is_none() {
                            if let Err(e)=self.voices.play_legacy(sound) {self.audio_error=Some(e.to_string());}
                        }
                    }
                }
'''
new='''                for cue in &frame.legacy_audio {
                    if self.audio_error.is_none() && GetForegroundWindow()==parent {
                        match cue.submit(&native_scope,&mut self.voices) {
                            Err(e)=>self.audio_error=Some(e.to_string()),
                            Ok(super::native_audio_delivery::DeliveryOutcome::Unbound)=>self.audio_error=Some("一般音效尚未綁定場景".into()),
                            _=>{},
                        }
                    } else {cue.discard();}
                }
'''
replace(n,old,new)
for name,value in text.items():(out/name).write_text(value,encoding='utf-8')
for name,data in original.items():assert(P/'src/modern_lito'/name).read_bytes()==data
record=dict(status='prepared_uncompiled_legacy_audio_lifecycle_overlays',normal_source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],
    component_qualification=dict(path=str(qp),sha256=sha(qp)),component=dict(path=str(component),sha256=sha(component)),
    source_overlays={name:dict(original_path=str(P/'src/modern_lito'/name),original_sha256=hashlib.sha256(original[name]).hexdigest().upper(),candidate_path=str(out/name),candidate_sha256=sha(out/name))for name in names},
    behavior='Mirror ordinary prepared WAVs into scoped event payloads, bind only at accepted publication, preserve environment deferral, discard retired/unfocused events, and keep existing play_legacy voice policy.',
    normal_product_source_modified=False,normal_integration_executed=False,candidate_compiled=False,C02_executed=False,
    Debug_source_readiness_failure_still_pending=True,new_sessions=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,
    pending=['Compile overlays with the separately pinned diagnostic C1F request extension. Record both baseline and effective candidate identities; never call the changed module bodies unchanged.',
        'Update the isolated probe to handle legacy payloads and native finite cues separately, taking retained event provenance from the legacy payload.',
        'Verify publication, environment deferral/retirement/focus policy and then real C1F/S20/C72 both profiles without relaxing existing deadlines.',
        'Only after candidate validation apply the final change to normal sources and revalidate the resulting normal products and required G01-G12 scope.'])
save(out/'preparation.json',record)
pins={p.name:sha(p)for p in out.iterdir()if p.is_file()}
with zipfile.ZipFile(out/'candidate.zip','x',zipfile.ZIP_DEFLATED)as archive:
    for name in pins:archive.write(out/name,name)
with zipfile.ZipFile(out/'candidate.zip')as archive:assert all(hashlib.sha256(archive.read(name)).hexdigest().upper()==h for name,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'candidate.zip'),normal_product_source_modified=False))
for name,h in inputs.items():assert sha(ROOT/name)==h
print(json.dumps(dict(candidate=str(out),overlays=4,compiled=False,normal_product_source_modified=False),ensure_ascii=False))
