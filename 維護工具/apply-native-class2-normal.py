"""Apply qualified native class2/scene fixes with exact source and recovery archives."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,os,re,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';SRC=P/'src/modern_lito'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--release-run',type=Path,required=True);a=ap.parse_args()
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads')
from native1030_broad_gate import require_idle_runtime
require_idle_runtime(ROOT)
base=ROOT/'驗證紀錄/current/native77-lawful/validation-20261017-195458-430865'
inputs=read(base/'compiled-inputs.json');old_source=ROOT/'驗證紀錄/current/native77-lawful/application-20261017-195433-104755/applied-source-inputs.json'
source=read(old_source);assert sha(old_source)==queue['current_product_identity']['source_manifest_sha256']=='E15907AC8DF9A2C84A789034023EB520C600C298D6ED622693D560BFD3B48182'
assert len(source)==1030 and len(inputs)==1031
for name,digest in inputs.items():assert sha(ROOT/name)==digest
prior=read(ROOT/'驗證紀錄/current/native-class2-position-motion-checkpoint/20261018-012941-869179/evidence.json')
core=None;qualifications=[]
for profile,run in [('Debug',ROOT/'驗證紀錄/current/native-class2-same-map/20261018-013647-340296'),('Release',a.release_run.resolve(strict=True))]:
    assert run.is_relative_to(ROOT/'驗證紀錄/current/native-class2-same-map')
    qp=next(run.glob('qualified-*/qualified.json'));q=read(qp)
    assert q['status']=='independently_qualified_class2_WORLD_and_same_numeric_map_retirement' and q['profile']==profile
    assert q['original_exit_code']==0 and q['recorded_owned_all_terminal'] and q['same_map_real109_retirement_verified'] and q['same_numeric_map_transitions']==2
    pins={n:v for n,v in q['candidate_inputs'].items() if n!='diagnostic_probe.rs'}
    assert {n:v['sha256'] for n,v in pins.items()}==prior['core_module_pins'] and len(pins)==13
    for pin in pins.values():assert sha(pin['path'])==pin['sha256']
    if core is None:core=pins
    assert pins==core
    qualifications.append(dict(profile=profile,path=str(qp),sha256=sha(qp)))
payload={(SRC/name).relative_to(ROOT).as_posix():Path(pin['path']).read_bytes() for name,pin in core.items()}
mod=SRC/'mod.rs';text=mod.read_text(encoding='utf-8-sig')
for name in ['call_lightning_assets','native_call_lightning','native_ambient_bridge','native_ambient_override']:
    assert not re.search(r'\bmod\s+'+name+r'\s*;',text);text+='\npub mod '+name+';\n'
payload[mod.relative_to(ROOT).as_posix()]=text.encode('utf-8')
probe=(ROOT/'維護工具/candidates/native77-request/request_probe.rs').read_text(encoding='utf-8-sig');body=probe[probe.index('use modern_lito::{'):]
body=body.replace('use modern_lito::{','use lito_modern::modern_lito::{',1).replace('__WORLD_FIXTURE__','fixtures/native-world-current.json')
body=re.sub(r'\b77\b','34',body).replace('140904c800000002800180','140401c800000002800180').replace('range_ten_','range_six_')
for before,after in [('x + 10','x + 6'),('x + 11','x + 7'),('x - 10','x - 6'),('reuse_millis(34), 5000','reuse_millis(34), 10'),('究極光裂術','極道落雷')]:body=body.replace(before,after)
test=P/'tests/native_attack_skills.rs';text=test.read_text(encoding='utf-8-sig');assert 'mod call_lightning_request_contract' not in text
payload[test.relative_to(ROOT).as_posix()]=(text+'\nmod call_lightning_request_contract {\n'+body+'\n}\n').encode('utf-8')
tail=(ROOT/'維護工具/candidates/native-class2-presentation/world_probe_tail.rs').read_text(encoding='utf-8-sig')
helper=tail[tail.index('fn motion_hurt_preflight()'):tail.index('pub fn run()->Result<()>')]
test=P/'tests/native_animation.rs';text=test.read_text(encoding='utf-8-sig');assert 'mod hurt_queued_after_motion' not in text
addition='\nmod hurt_queued_after_motion {\nuse lito_modern::modern_lito as m;\nuse anyhow::{ensure,Context,Result};\nuse serde_json::{json,Value};\n'+helper+'\n#[test]\nfn eight_directions_hurt_waits_for_walk_without_camera_rewind_and_old_life_replay(){motion_hurt_preflight().unwrap();}\n}\n'
payload[test.relative_to(ROOT).as_posix()]=(text+addition).encode('utf-8')
assert len(payload)==16
meta=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
out=ROOT/'驗證紀錄/current/native-class2-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
existing={n:ROOT/n for n in [*payload,*meta] if (ROOT/n).is_file()}
existing.update({profile+'-normal/LiToModern.exe':P/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe' for profile in ['Debug','Release']})
before={n:sha(p) for n,p in existing.items()}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,p in existing.items():z.write(p,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
after={n:hashlib.sha256(data).hexdigest().upper() for n,data in payload.items()}
with zipfile.ZipFile(out/'proposal.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name,data in payload.items():z.writestr(name,data)
with zipfile.ZipFile(out/'proposal.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in after.items())
state=dict(status='prepared_class2_normal_application',applied=False,before_source_manifest_sha256=sha(old_source),candidate_qualifications=qualifications,
    core_module_pins=prior['core_module_pins'],before_files=before,after_files=after,new_files=[n for n in payload if not (ROOT/n).exists()],before_archive_sha256=sha(out/'before.zip'),proposal_archive_sha256=sha(out/'proposal.zip'),
    normal_validation_complete=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'application.json',state);(out/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
for name,digest in inputs.items():assert sha(ROOT/name)==digest
for name,data in payload.items():
    temp=out/Path(name).name;temp.write_bytes(data);os.replace(temp,ROOT/name);assert sha(ROOT/name)==after[name];source[name]=after[name];inputs[name]=after[name]
source=dict(sorted(source.items()));inputs=dict(sorted(inputs.items()));assert len(source)==1034 and len(inputs)==1035
save(out/'applied-source-inputs.json',source);save(out/'applied-compiled-inputs.json',inputs)
state.update(status='applied_class2_pending_normal_related_and_current_source_runtime_validation',applied=True,source_manifest_sha256=sha(out/'applied-source-inputs.json'),source_files=1034,compiled_input_files=1035)
save(out/'application.json',state);save(out.parent/'active-application.json',dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
work='原生class2落雷與場景退役、目標一次取位、受擊排於步行邊界之13模組候選已套入正常來源，另加四模組宣告與5個請求／選單契約及八方向動畫回歸。候選兩版實際移動、活動亮度傳圖與同圖號S5A／109退役已各自核對；新來源正常兩主、相關測試及當版執行資格尚待。舊E159兩主與default／library只保留封存版本資格，不移記新來源。先沿唯一入口建置兩版、驗相關目標並保持來源及QA凍結，再接當版實際流程與廣域檢查。登入器／正式JAR只讀，GUI／實體輸入音效、完整G06及G01–G12未完成，8／60秒與G11／Release設定不變。'
for name in meta[:5]:
    path=ROOT/name;j=read(path);old=j['current_product_identity'];assert old['source_manifest_sha256']==state['before_source_manifest_sha256']
    j['retained_source_E159_product_identity']=old;j['retained_source_E159_goal_validation']=j.get('current_source_goal_validation')
    j['current_product_identity']=dict(source_manifest_sha256=state['source_manifest_sha256'],source_files=1034,compiled_input_files=1035,source_qualification_pending=True,
        Debug_normal_sha256=old['Debug_normal_sha256'],Release_normal_sha256=old['Release_normal_sha256'],existing_normal_source_manifest_sha256=state['before_source_manifest_sha256'],
        Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
    j['current_native_class2_application']=dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256'],normal_validation_pending=True)
    j['current_source_goal_validation']={key:dict(state='class2_current_source_validation_pending',complete=False) for key in (j.get('current_source_goal_validation') or {})}
    for item in j.get('items',[]):
        if item['id'].startswith(('G01-','G03-')):item.update(state='class2_current_source_validation_pending',complete=False)
    j.update(source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'application.json'),goal_complete=False,whole_goal_complete=False)
    if name==meta[4]:j['next_action']=work
    else:j['current_focus']=dict(item='class2-normal-build-and-related-validation',work=work,applies_to_current_source=True,goal_complete=False)
    save(path,j)
for name in meta[5:]:
    path=ROOT/name;first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'tracker-readback.json',dict(after_files={n:sha(ROOT/n) for n in meta},source_manifest_sha256=state['source_manifest_sha256']))
print(json.dumps(dict(application=str(out),source_manifest_sha256=state['source_manifest_sha256'],changed_files=16,normal_validation_complete=False),ensure_ascii=False))
