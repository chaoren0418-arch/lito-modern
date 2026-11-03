"""Combine two disjoint, executed current five-body scopes without rerunning tests."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('base',type=Path);ap.add_argument('scene',type=Path);a=ap.parse_args()
scopes={};sources=None;normal=None;names=set();profiles={};histories=[]
expected_leaves={
 'real_explicit_sprite_bank_binds_eight_headings_and_unique_scope_rejects_collision',
 'real_shared_attack_uses_source_fps_ticks_and_once_only_presentation_events',
 'actual_shared_bow_bodies_match_independent_pixels_and_launch_cues',
 'actual_shared_arrow_eight_banks_match_independent_reference','real_shared_sound_1539_matches_pcm_reference',
 'real_honor_effect_roof_cache_matches_csharp_and_counts_full_budget',
 'real_shared_armed_families_preload_and_knight_pixels_match_independent_csharp_goldens',
 'real_shared_combat_assets_match_qualified_one_shots_and_render_attack_hurt_death',
 'real_shared_roofs_fade_and_restore_without_archive_reads_during_render',
 'real_shared_world_all_base_appearances_decode_and_authoritative_camera_moves'}
for kind,path in [('base',a.base),('scene',a.scene)]:
    run=path.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-resource-optin')
    qp=run/'qualified/qualified.json';q=read(qp);c=read(run/'cohort.json')
    assert q['status']=='independently_qualified_current_both_profiles_five_original_resource_bodies'
    assert q['library_kind']=='startup-current' and q['source_files']==1024 and q['compiled_input_files']==1025
    assert q['resource_scope']==kind and q['observed_terminal_exit_code']==0 and q['original_session_id']>0
    assert q['actual_test_invocations']==10 and q['unique_previously_ignored_bodies']==5
    assert q['original_owned_processes_terminal'] and c['owned_process'] is None
    assert len(c['process_history'])==10 and all(v['exit_code']==0 for v in c['process_history'])
    histories+=c['process_history']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:
        assert len(z.namelist())==len(q['raw_files']) and set(z.namelist())==set(q['raw_files'])
        for name,value in q['raw_files'].items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==value
    this_sources=read(run/'compiled-inputs.json');assert len(this_sources)==1025
    this_normal=(c['normal_qualification'],c['normal_qualification_sha256'],c['normal_profiles'])
    if sources is None:sources=this_sources;normal=this_normal
    else:assert sources==this_sources and normal==this_normal
    assert not names.intersection(q['test_names']);names.update(q['test_names'])
    for profile,p in q['profiles'].items():
        assert p['passed']==5 and p['failed']==0
        image=Path(c['profiles'][profile]['immutable_test_exe'])
        assert sha(image)==p['actual_test_sha256']
        if profile not in profiles:profiles[profile]=dict(passed_optin_bodies=0,failed=0,actual_test_sha256=p['actual_test_sha256'],immutable_test_exe=str(image))
        assert profiles[profile]['actual_test_sha256']==p['actual_test_sha256']
        assert profiles[profile]['immutable_test_exe']==str(image)
        profiles[profile]['passed_optin_bodies']+=5
    if kind=='scene':
        assert q['sixth_scene_test_qualified'] is False
        sixth=q['unexecuted_sixth_scene_precondition']
    scopes[kind]=dict(qualification=str(qp),qualification_sha256=sha(qp),original_session_id=q['original_session_id'],
        observed_terminal_exit_code=0,raw_archive_sha256=q['raw_archive_sha256'])
assert len(names)==10 and {name.rsplit('::',1)[-1] for name in names}==expected_leaves
assert set(profiles)=={'Debug','Release'} and all(p['passed_optin_bodies']==10 for p in profiles.values())
for name,value in sources.items():assert sha(ROOT/name)==value
assert sha(normal[0])==normal[1]
for value in normal[2].values():assert sha(value['path'])==value['sha256']
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-resource-optin'/('current1024-ten-bodies-'+stamp);out.mkdir()
(out/'combiner.py').write_bytes(Path(__file__).read_bytes())
for kind,info in scopes.items():(out/(kind+'-qualification.json')).write_bytes(Path(info['qualification']).read_bytes())
save(out/'compiled-inputs.json',sources);save(out/'process-history.json',histories)
files={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in files:z.write(out/name,name)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(files)
    for name,value in files.items():assert hashlib.sha256(z.read(name)).hexdigest().upper()==value
q=dict(status='independently_readback_qualified_current_ten_unique_original_resource_bodies_both_profiles',
    source_files=1024,compiled_input_files=1025,source_manifest_sha256=read(a.base/'qualified/qualified.json')['source_manifest_sha256'],
    scopes=scopes,profiles=profiles,test_names=sorted(names),unique_previously_ignored_bodies=10,
    actual_test_invocations=20,actual_test_executables=2,recorded_test_processes=20,
    unchanged_default_library_ignored_count=473,other_library_ignored_bodies_not_qualified_by_these_scopes=463,
    sixth_scene_test_qualified=False,unexecuted_sixth_scene_precondition=sixth,
    normal_main_identities=normal[2],normal_related_qualification=normal[0],normal_related_qualification_sha256=normal[1],
    full_Sprite_PAKs_metadata_only=True,all_resources_or_ignored_bodies_verified=False,
    current_native_Disint_gameplay_or_failure_inferred_from_old_scene_precondition=False,
    actual_formal_sessions=0,GUI_verified=False,audio_device_verified=False,performance_accepted=False,
    raw_archive_sha256=sha(out/'raw.zip'),raw_files=files,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
save(out/'qualified.json',q)
print(json.dumps(dict(qualification=str(out/'qualified.json'),unique_bodies=10,actual_test_invocations=20,whole_goal_complete=False),ensure_ascii=False))
