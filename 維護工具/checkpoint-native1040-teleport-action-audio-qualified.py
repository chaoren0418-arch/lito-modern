"""Publish only an independently qualified terminal audio profile; preserve full goal."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, zipfile

ROOT = Path(__file__).resolve().parents[1]

def read(path):
    return json.loads(Path(path).read_text('utf-8-sig'))

def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

def save(path, value):
    Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

parser = argparse.ArgumentParser()
parser.add_argument('run', type=Path)
parser.add_argument('--qualification', type=Path, required=True)
parser.add_argument('--session', type=int, required=True)
parser.add_argument('--terminal-chunk', required=True)
args = parser.parse_args()
run = args.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native1040-teleport-action-audio')
receipt = read(run/'receipt.json')
assert receipt['status']=='executed_current_Presentation_handoff_diagnostic' and receipt['owned_process'] is None
qp = args.qualification.resolve(strict=True)
assert qp.is_relative_to(run) and qp.name=='qualified.json'
q = read(qp)
assert q['source_audio_kind']=='received_S20_action68_sound1534' and q['diagnostic_request_API_extension'] and q['normal_source_bodies_unchanged'] and not q['normal_GUI_API_permission_inferred']
assert q['status']=='independently_qualified_current_C02_Presentation_audio_handoff_diagnostic_scope'
assert q['profile']==receipt['profile'] and q['source_manifest_sha256']==receipt['source_manifest_sha256']
assert q['original_exit_code']==0 and q['recorded_owned_all_terminal'] and q['actual_sessions']==6
assert q['original_session_id']==args.session and q['terminal_tool_chunk']==args.terminal_chunk
assert q['unplayed_source_audio_rejection_verified'] and q['new_destination_audio_delivered_once_each']
assert len(q['authentic_source_audio_wire_proofs'])==2 and q['retained_source_cues']==2
assert not q['normal_EXE_runtime_inferred'] and not q['physical_audio_verified'] and not q['whole_G06_complete']
assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
with zipfile.ZipFile(qp.parent/'raw.zip') as archive:
    assert set(archive.namelist())==set(q['raw_files'])
    for name, digest in q['raw_files'].items():
        with archive.open(name) as stream:
            assert hashlib.file_digest(stream, 'sha256').hexdigest().upper()==digest
active = list(run.glob('running-checkpoint-*/readback.json'))
assert len(active)==1
before = read(active[0])['after_files']
assert len(before)==8 and all(sha(ROOT/name)==digest for name,digest in before.items())
queue = read(ROOT/'工作佇列.json')
assert queue['current_native1040_G06_audio_active']['original_session_id']==args.session
assert queue['current_native1040_G06_audio_active']['qualification_pending']
prior = queue.get('current_native1040_authentic_audio', {})
if prior:
    assert prior['source_manifest_sha256']==q['source_manifest_sha256']
profiles = dict(prior.get('profiles', {}))
assert q['profile'] not in profiles
profiles[q['profile']] = dict(qualification=str(qp), qualification_sha256=sha(qp), actual_sessions=6,
    actual_product_sha256=q['actual_product_sha256'], original_session_id=args.session,
    terminal_tool_chunk=args.terminal_chunk, recorded_owned_identities=q['recorded_owned_identities'],
    unplayed_source_audio_rejection_verified=True, new_destination_audio_delivered_once_each=True)
for profile, info in profiles.items():
    p = Path(info['qualification']); assert sha(p)==info['qualification_sha256']
    assert read(p)['profile']==profile and read(p)['source_manifest_sha256']==q['source_manifest_sha256']
both = set(profiles)=={'Debug','Release'}
record = dict(source_audio_kind='received_S20_action68_sound1534',diagnostic_request_API_extension=True,normal_source_bodies_unchanged=True,normal_GUI_API_permission_inferred=False,status='qualified_both_profiles_diagnostic_audio_scope' if both else 'qualified_one_profile_other_profile_pending',
    source_manifest_sha256=q['source_manifest_sha256'], profiles=profiles,
    actual_sessions=sum(p['actual_sessions'] for p in profiles.values()), both_profiles_qualified=both,
    normal_EXE_runtime_inferred=False, GUI_verified=False, physical_audio_verified=False,
    same_endpoint_verified=False, whole_G06_complete=False, whole_goal_complete=False)
out = run/('qualified-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
with zipfile.ZipFile(out/'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in before:
        archive.write(ROOT/name, name)
with zipfile.ZipFile(out/'before.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper()==digest for name,digest in before.items())
save(out/'evidence.json', record)
work = '來源'+q['source_manifest_sha256'][:8]+'真實C1F揮手／S20來源音效跨端點診斷已獨立核對'+('兩版' if both else q['profile']+'；另一版待驗')+'，累計'+str(record['actual_sessions'])+' LOGIN／GAME sessions。每端待播舊cue為Stale、同cue重複為AlreadyConsumed且零交付；目的地真實heal1／S37之sound109各交付一次。來源／映像／raw及記錄程序身分終止均已核對，只授予此原核心本文加診斷請求入口／假Player範圍；正常UI動作請求權限不推定；正常EXE／GUI與實體音效、同端點與其他服務及完整G06／G01–G12仍待。同來源兩版廣域與完整library限定資格維持。下一步'+('必要資源及正常可見性驗證' if both else '另一版相同音效跨端點案例')+'；8／60秒、Release／G11預算不變，相鄰只讀且登入器與重建版獨立。'
for name in before:
    path = ROOT/name
    if path.suffix=='.json':
        value = read(path)
        assert value['current_product_identity']['source_manifest_sha256']==q['source_manifest_sha256']
        value['current_native1040_authentic_audio']=record
        value['current_native1040_G06_audio_active'].update(status='completed_qualified_profile_audio_scope',qualification_pending=False,qualification=str(qp),qualification_sha256=sha(qp),terminal_tool_chunk=args.terminal_chunk,observed_terminal_exit_code=0)
        value.update(active_workloads={}, active_workload_sessions={}, source_and_corresponding_QA_frozen=False,
            source_freeze_while_Cargo=False, next_task=work, next_focus=work, last_goal_turn_classification='progress',
            last_goal_turn_evidence=str(out/'evidence.json'), goal_complete=False, whole_goal_complete=False)
        value['current_focus']=dict(item='current1040-authentic-audio-and-resource-acceptance',work=work,applies_to_current_source=True,goal_complete=False)
        if name.endswith('next-validation-action.json'):
            value['next_action']=work
        save(path, value)
    else:
        first, separator, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert first.startswith('2026-10-18') and separator
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json', dict(before_files=before, after_files={name:sha(ROOT/name) for name in before},
    before_archive_sha256=sha(out/'before.zip'), evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/name)==digest for name,digest in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json', dict(**record,checkpoint=str(out),must_poll_original_session_before_restarting=False,next_action='Other profile authentic audio handoff' if not both else 'Necessary resources and normal visibility acceptance; physical/GUI scope remains unqualified.'))
print(json.dumps(dict(checkpoint=str(out),profiles=list(profiles),actual_sessions=record['actual_sessions'],whole_goal_complete=False),ensure_ascii=False))
