"""Record the reviewed normal-GUI expiry failure without promoting runtime exit 0."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,re,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('evidence',type=Path);a=ap.parse_args()
ep=a.evidence.resolve(strict=True);assert ep.is_relative_to(V/'native-source-handoff-expiry')
e=read(ep);source=e['source_manifest_sha256'];assert e['selection_to_GAME_version_seconds']>30 and not e['GUI_qualification_granted']
assert source=='4F2F911B4B70BDA16C4EE8C3EDD60223F3E72A22C8A3B1B808C3F5EABA60E6A2'
for n,h in e['raw_files'].items():
    with zipfile.ZipFile(ep.parent/'raw.zip')as z:assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
assert sha(ep.parent/'raw.zip')==e['raw_archive_sha256']
raw=(ep.parent/'ClientSession.stdout.txt').read_text('utf-8-sig').splitlines()
starts=[i for i,l in enumerate(raw)if re.match(r'^  \S.*\(.*\).*;$',l)]
idx=next(i for i,l in enumerate(raw)if 'private void requestLitoCharacterInfo(byte[])'in l)
end=next((i for i in starts if i>idx),len(raw));method='\n'.join(raw[idx:end])+'\n'
assert all(s in method for s in ['pending', 'AUTHENTICATED','LOBBY','stageClassicSourceHandoff','loadCharacterDirectory'])
hub=read(ep.parent/'selected-methods.json')['GameHub']
stage=next(v for k,v in hub.items()if 'stageClassicSourceHandoff'in k)
pending=next(v for k,v in hub.items()if 'pendingClassicSourceHandoff'in k)
assert 'long 30000000000l'in stage and 'expiresAtNanos'in pending and 'java/util/Map.remove'in pending and 'aconst_null'in pending
w=read(R/'工作佇列.json');assert not w['active_workloads']and w['current_product_identity']['source_manifest_sha256']==source
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=ep.parent/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'requestLitoCharacterInfo.txt').write_text(method,encoding='utf-8')
record=dict(status='normal_GUI_Release_source_handoff_expiry_failure_preserved_fix_required',source_manifest_sha256=source,
    case=e['case'],original_session=e['original_session'],terminal_tool_chunk=e['terminal_tool_chunk'],original_exit_code=0,
    qualification_granted=False,requested_source=e['requested_source'],actual_GAME_source=e['actual_GAME_source'],
    evidence=str(ep),evidence_sha256=sha(ep),selection_to_GAME_version_seconds=e['selection_to_GAME_version_seconds'],
    formal_jar_sha256=e['formal_jar_sha256'],formal_classic_source_handoff_TTL_seconds=30,
    reviewed_cause='The selected-source C8 staged a 30s handoff. The GAME version packet was sent 30.644957s later, so the later GAME authentication had no live staged source and kept the default source. The normal client published that first directory.',
    current_protocol_C8_contract=dict(method=str(out/'requestLitoCharacterInfo.txt'),sha256=sha(out/'requestLitoCharacterInfo.txt'),
        LOGIN_replay_after_lobby_ignored=True,GAME_requires_LOBBY_and_completed_news_readiness=True,
        requested_name_must_be_canonical_and_authorized=True,live_GAME_C8_rebind_supported_by_bytecode=True,
        new_GAME_C8_runtime_not_yet_verified=True),
    intended_fix='Confirm the chosen source with public GAME C8 after the initial directory, withholding CharactersReady until the confirmed directory completes; keep staged rental teleport/recovery bootstrap separate.',
    required_validation=['natural >30s initial LOGIN-to-GAME delay, unchanged server TTL',
        'no first wrong-source directory exposed or selectable', 'exact original C8/readiness sequence and requested-source character/inventory',
        'quick initial bootstrap, staged rental destination/recovery and duplicate-C9 rejection',
        'current normal Debug/Release, full library and normal GUI after source change'],
    native_8s_and_peer_60s_unchanged=True,formal_JAR_modified=False,product_source_modified=False,whole_goal_complete=False)
(out/'evidence.json').write_bytes(data(record))
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='當版完整library兩版1969過0敗484忽略與原資源各10過已核對；Debug限定入村GUI通過。Release49194原始exit0但B來源接續已超正式30秒而落到測試甲，已在選角前停止，GUI不授予通過。先修正初次GAME來源確認，再重驗來源、原始封包、回歸與正常GUI；不改JAR或30秒／8秒／60秒時限，不請使用者操作。'
staged={}
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);assert j['current_product_identity']['source_manifest_sha256']==source
        j['current_normal_GUI_source_handoff_failure']=record
        j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='initial_GAME_source_handoff_expiry_fix',work=work,applies_to_current_source=True,goal_complete=False)
        j['current_source_goal_validation'].setdefault('G08-player-gui',{}).update(complete=False,state='Release_initial_source_handoff_expiry_failure_fix_required')
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
        datetime.strptime(first[:10],'%Y-%m-%d')
        staged[n]=((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},evidence_sha256=sha(out/'evidence.json'),before_archive_sha256=sha(out/'before.zip'))))
assert all(sha(R/n)==h for n,h in read(out/'readback.json')['after_files'].items())
print(json.dumps(dict(checkpoint=str(out),GUI_qualification_granted=False,fix_required=True),ensure_ascii=False))
