"""Qualify the isolated proposal without claiming a live detection cast or GUI."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--exec-chunk-id',required=True)
ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-detection-skill-candidate') and a.terminal_exit_code==0
r=read(run/'receipt.json');assert r['status']=='unapplied_detection_candidate_passed_pending_independent_readback'
assert r['owned_process'] is None and len(r['processes'])==2 and all(p['exit_code']==0 for p in r['processes'])
archive=read(run/'archive-readback.json');assert sha(run/'raw.zip')==archive['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert len(z.namelist())==len(archive['files']) and set(z.namelist())==set(archive['files'])
    for name,value in archive['files'].items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==value
        if name!='Rust-x64-probe.exe':assert sha(run/name)==value
    exe=z.read('Rust-x64-probe.exe');at=int.from_bytes(exe[60:64],'little')
    assert exe[:2]==b'MZ' and exe[at+4:at+6]==b'\x64\x86' and exe[at+24:at+26]==b'\x0b\x02'
    assert hashlib.sha256(exe).hexdigest().upper()==r['diagnostic_sha256']
candidate=ROOT/'維護工具/candidates/native-detection-skill';meta=read(run/'metadata.json')
for name,value in r['candidate_files'].items():assert sha(run/name)==sha(candidate/name)==value
for name,info in meta['files'].items():assert sha(ROOT/'LiTo381-Modern/src/modern_lito'/name)==info['base_sha256']
extract=read(run/'method-extraction.json');source=(run/'native_skills.rs').read_text(encoding='utf-8-sig')
first=source.index('fn can_cast(');end=source.index('#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]',first)
assert (run/'request-method-extract.rs').read_text(encoding='utf-8-sig')==source[first:end]
assert sha(run/'request-method-extract.rs')==extract['extracted_sha256']
source=(run/'native_skill_effects.rs').read_text(encoding='utf-8-sig')
first=source.index('        // S37\'s explicit11722..11725 selects the reviewed TW data namespace.')
end=source.index('    }\n    pub fn draw(',first)
assert (run/'effect-method-extract.rs').read_text(encoding='utf-8-sig')==source[:first]+source[end:]
assert sha(run/'effect-method-extract.rs')==extract['effect_method_extract_sha256']
assert hashlib.sha256(source[first:end].encode()).hexdigest().upper()==extract['omitted_TW_branch_sha256']
for name in ['tower_assets.rs','shared_action_catalog.rs']:
    assert sha(run/name)==sha(ROOT/'LiTo381-Modern/src/modern_lito'/name)
template=(run/'probe-template.rs').read_text(encoding='utf-8-sig');assert sha(run/'probe-template.rs')==r['probe_sha256']
for token,name in [('@REQUEST_EXTRACT@','request-method-extract.rs'),('@SELECTION@','native_skill_selection.rs'),
    ('@EFFECTS@','effect-method-extract.rs'),('@TOWER@','tower_assets.rs'),('@SHARED_CATALOG@','shared_action_catalog.rs')]:
    assert template.count(token)==1;template=template.replace(token,(run/name).as_posix())
assert template==(run/'probe.rs').read_text(encoding='utf-8-sig')
result=read(run/'candidate-results.json');definition=read(run/'original749-definition.json')
assert result['request']=='140104' and result['selector_codec_compared'] and result['request_does_not_predict_MP']
assert result['request_guard_rejections']==['unlearned','zero_mp','zero_hp','dead','not_started','movement_block']
assert result['source_frames']==len(definition['frames'])==23
assert definition['graphics']==definition['bound_graphics']==749 and definition['image_count']==1
assert definition['directionality']=='Undirected' and definition['frames_per_second']==24
assert all(f['image']==0 and f['subframe']==(22 if i==0 else i-1) for i,f in enumerate(definition['frames']))
assert all(f['events']==('[Sound(108)]' if i==1 else '[]') for i,f in enumerate(definition['frames']))
assert result['SPR_name']=='749-0.spr' and result['SPR_sha256']=='01D399D0D7F4ACC94AFD4A5A03E4373E101C57BDEF078EF8EAFD6492B792A8B7'
assert len(result['samples'])==24 and result['unique_frame_events']==1
due=0;events=[]
for step,row in enumerate(result['samples']):
    assert row['load_error'] is None and row['started']==1 and row['omitted']==0
    assert len(row['effects'])==1;effect=row['effects'][0]
    frame=step+1 if step<22 else 0;finished=step==23
    assert effect['source_frame']==frame and effect['graphics']==749 and effect['callback_count']==step+1
    assert effect['native_clock_elapsed_ms']==due and effect['finished']==finished
    assert row['active']==(0 if finished else 1)
    if not finished:
        due+=definition['frames'][frame]['ticks']*1000//24
        assert effect['next_callback_ms']==due and effect['drawn']
    else:assert effect['next_callback_ms'] is None and not effect['drawn']
    events+=row['entered_events']
assert len(events)==1 and events[0]['sound']==108
assert result['bank_RGBA_bytes']==15263630 and result['bank_encoded_bytes']==1441289
assert result['unrelated_private_TW_prepare_branch_omitted'] and not result['full_candidate_effect_loader_compiled']
pins=read(run/'compiled-inputs.json');assert len(pins)==1025
for name,value in pins.items():assert sha(ROOT/name)==value
for info in r['libraries'].values():assert sha(info['path'])==info['sha256']
failed=ROOT/'驗證紀錄/current/native-detection-skill-candidate/20261017-002015-718185'
assert read(failed/'receipt.json')['status']=='failed_preserved'
assert sha(failed/'raw.zip')==read(failed/'archive-readback.json')['archive_sha256']=='1F9346E1D351FD2DFA189B82C70579D644011D16E2A6010876A4FB67BC4C6DFD'
q=dict(status='independently_read_back_unapplied_detection_request_selection_and_original_effect_candidate',
    original_session_id=None,original_exec_tool_chunk_id=a.exec_chunk_id,observed_terminal_exit_code=0,
    source_manifest_sha256=r['source_manifest_sha256'],candidate_files=r['candidate_files'],skill_id=13,request_hex='140104',
    request_guard_rejections=6,source_frames=23,finite_effect_samples=24,sound108_entry_count=1,
    bank_RGBA_bytes=result['bank_RGBA_bytes'],bank_encoded_bytes=result['bank_encoded_bytes'],
    full_candidate_selection_compiled=True,generic_skill_effect_loader_and_draw_compiled=True,
    unrelated_private_TW_prepare_branch_omitted=True,full_candidate_effect_loader_compiled=False,
    unchanged_probe_dependency_files=['tower_assets.rs','shared_action_catalog.rs'],
    existing_classic_decoder_frame_comparison=True,independent_CSharp_pixel_oracle_executed=False,
    previous_compile_failure=str(failed),previous_failure_archive_sha256=sha(failed/'raw.zip'),
    candidate_applied_to_product=False,product_source_changed=False,actual_formal_sessions=0,actual_casts=0,
    invisible_target_reveal_or_cooldown_runtime_verified=False,GUI_verified=False,audio_device_verified=False,
    raw_archive_sha256=archive['archive_sha256'],raw_files=archive['files'],whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
assert not (run/'independent-readback.json').exists()
(run/'independent-readback.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:q[k] for k in ['status','skill_id','source_frames','finite_effect_samples','candidate_applied_to_product']},ensure_ascii=False))
