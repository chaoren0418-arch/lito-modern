"""Prepare source-body probes with nonempty ordinary audio across every G06 branch."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,os
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current';T=ROOT/'維護工具'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
normal=V/'native-legacy-audio-normal/validation-20261018-104256-823814';c=read(normal/'cohort.json')
assert c['status']=='executed_legacy_audio_normals_related' and c['source_manifest_sha256']=='8CC5A560EFC60C6830658A06B3C4430593493377D433684090B4F46F3427D24E'
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h
v2=T/'candidates/native-teleport-legacy-audio-v2';previous=read(v2/'preparation.json')
assert sha(v2/'probe.rs')==previous['candidate_probe_sha256'] and sha(v2/'diagnostic_wave_api.rs')==previous['diagnostic_API_sha256']
base=T/'candidates/native-teleport-failure-presentation/probe.rs'
out=T/'candidates/native8CC5-teleport-audio';out.mkdir()
success=(v2/'probe.rs').read_text('utf-8-sig');failure=base.read_text('utf-8-sig')
header='//! Current normal module bodies plus an isolated C1F request extension.\n//! Real C02 event provenance and FakePlayer delivery only; no normal GUI/API permission or physical audio inference.\n'
success=header+success[success.index('use anyhow::'):]
failure=header+failure[failure.index('use anyhow::'):]
wave=success[success.index('fn wave_sound('):success.index('#[derive(Default)]')]
failure=failure.replace('#[derive(Default)]',wave+'#[derive(Default)]',1)
legacy_impl=success[success.index('impl m::native_audio_delivery::legacy::LegacyPlayer'):success.index('struct Peer {')]
failure=failure.replace('struct Peer {',legacy_impl+'struct Peer {',1)
# Retain the previously qualified tick and real C1F/S20 helpers, then preserve
# every existing failure/recovery phase bound and authority assertion below.
old_peer=failure[failure.index('struct Peer {'):failure.index('    fn wait(')]
new_peer=success[success.index('struct Peer {'):success.index('    fn wait(')]
failure=failure.replace(old_peer,new_peer,1)
needle='''        p.ready(8)?;
        let before = p.c.snapshot().clone();'''
assert failure.count(needle)==1
failure=failure.replace(needle,'''        p.ready(8)?;
        let source_audio = p.action_audio()?;
        let before = p.c.snapshot().clone();''',1)
needle='        let old_cues = p.view.frame.as_ref().unwrap().native_audio.clone();'
assert failure.count(needle)==1
failure=failure.replace(needle,'''        let old_cues = p.held.clone();
        ensure!(old_cues.len()==1, "nonempty held ordinary source cue required");''',1)
needle='''        for cue in &old_cues {
            old_cue_outcomes.push(format!("{:?}", cue.submit(&scope, &mut p.sink)?));
        }'''
assert failure.count(needle)==1
failure=failure.replace(needle,'''        for cue in &old_cues {
            let first=cue.submit(&scope, &mut p.sink)?;
            let repeat=cue.clone().submit(&scope, &mut p.sink)?;
            ensure!(first==m::native_audio_delivery::DeliveryOutcome::Stale,"old ordinary cue not stale");
            ensure!(repeat==m::native_audio_delivery::DeliveryOutcome::AlreadyConsumed,"old ordinary cue repeated");
            old_cue_outcomes.push(json!({"first":format!("{first:?}"),"repeat":format!("{repeat:?}")}));
        }''',1)
needle='''        drop(old_cues);
        if cancel_expiry || cancel_retry {'''
assert failure.count(needle)==1
failure=failure.replace(needle,'''        std::fs::write(root.join(format!("{role}-source-audio-retirement.json")),
            serde_json::to_vec_pretty(&json!({"source_audio":source_audio,"retirement":failure_retirement,"whole_case_qualified":false}))?)?;
        drop(old_cues);
        if cancel_expiry || cancel_retry {''',1)
needle='''        let recovered_picture = p.picture(root, role, "recovered")?;'''
assert failure.count(needle)==1
failure=failure.replace(needle,'''        p.stage="recovered_wave";
        let recovery_audio=p.action_audio()?;
        ensure!(p.held.len()==1,"recovery must prepare one own ordinary cue");
        let recovered_scope=p.view.audio_scope(p.c.transport().snapshot().epoch,p.c.snapshot().world.generation);
        let delivered_before=p.sink.played.len();
        let recovered_cue=p.held[0].clone();
        let recovered_first=recovered_cue.submit(&recovered_scope,&mut p.sink)?;
        let recovered_repeat=recovered_cue.clone().submit(&recovered_scope,&mut p.sink)?;
        ensure!(matches!(recovered_first,m::native_audio_delivery::DeliveryOutcome::Played(m::combat_audio::PlayOutcome::Started{..})),"new ordinary cue did not play");
        ensure!(recovered_repeat==m::native_audio_delivery::DeliveryOutcome::AlreadyConsumed && p.sink.played.len()==delivered_before+1,"new ordinary cue not once only");
        let recovery_audio=json!({"request":recovery_audio,"first":format!("{recovered_first:?}"),"repeat":format!("{recovered_repeat:?}"),"deliveries_before":delivered_before,"deliveries_after":p.sink.played.len()});
        let recovered_picture = p.picture(root, role, "recovered")?;''',1)
needle='"failure_retirement":failure_retirement,"recovered_picture":recovered_picture'
assert failure.count(needle)==1
failure=failure.replace(needle,'"failure_retirement":failure_retirement,"source_audio":source_audio,"recovery_audio":recovery_audio,"recovered_picture":recovered_picture',1)
needle='"trace":p.trace,"played":p.sink.played,"GUI_acceptance":false'
assert failure.count(needle)==1
failure=failure.replace(needle,'"trace":p.trace,"played":p.sink.played,"stage":p.stage,"last_snapshot":p.c.snapshot(),"last_frame_receipt":p.view.frame.as_ref().map(|f|f.receipt.clone()),"GUI_acceptance":false',1)
for name,body in [('success.rs',success),('failure.rs',failure)]: (out/name).write_text(body,encoding='utf-8',newline='\n')
(out/'diagnostic_wave_api.rs').write_bytes((v2/'diagnostic_wave_api.rs').read_bytes())
rustfmt=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustfmt.exe'
subprocess.run([str(rustfmt),'--edition','2021',str(out/'success.rs'),str(out/'failure.rs')],check=True,cwd=P)
record=dict(status='prepared_uncompiled_current_normal_nonempty_audio_probes',source_manifest_sha256=c['source_manifest_sha256'],normal_cohort=str(normal),
    origins={str(p):sha(p) for p in [v2/'probe.rs',v2/'diagnostic_wave_api.rs',base]},
    files={n:sha(out/n) for n in ['success.rs','failure.rs','diagnostic_wave_api.rs']},
    source_overlays_used=False,normal_source_bodies_unchanged=True,diagnostic_request_API_extension=True,normal_product_source_modified=False,
    normal_GUI_action_request_permission_inferred=False,original_failure_phase_bounds_and_authority_checks_preserved=True,
    source_audio='Actual C1F action68 followed by received S20, own gfx0/sound1534 or gfx61/sound1564, held without delivery.',
    retirement='First old payload Stale; clone AlreadyConsumed; zero old deliveries.',
    recovery='Actual C1F/S20 after fresh LOGIN/GAME/map500 reentry; fresh ordinary payload Played once, clone AlreadyConsumed.',
    compiled=False,executed=False,new_sessions=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(out),compiled=False,normal_product_source_modified=False),ensure_ascii=False))
