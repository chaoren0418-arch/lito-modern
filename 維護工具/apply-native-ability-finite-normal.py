"""Apply only the two-profile qualified ability/finite-effect payload within ROOT."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,os,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def archive(path,files):
    pins={n:hashlib.sha256(b).hexdigest().upper()for n,b in files.items()}
    with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED)as z:
        for n,b in files.items():z.writestr(n,b)
    with zipfile.ZipFile(path)as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
    return pins
ap=argparse.ArgumentParser();ap.add_argument('--debug-live',type=Path,required=True);ap.add_argument('--release-live',type=Path,required=True);a=ap.parse_args()
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads']and not queue['source_and_corresponding_QA_frozen']
old=V/'native-model-generation-normal/validation-20261018-160708-195207';source=read(old/'source-inputs.json');inputs=read(old/'compiled-inputs.json');old_sha=sha(old/'source-inputs.json')
assert old_sha==queue['current_product_identity']['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
for n,h in inputs.items():assert sha(ROOT/n)==h,n
proofs={};builds={};runtime_files=None
for profile,path in [('Debug',a.debug_live),('Release',a.release_live)]:
    path=path.resolve(strict=True);assert path.is_relative_to(V/'native44FE-ability-buff-requests')
    q=read(path);assert q['status']=='isolated_current75_ability_self_and_nonclan_scope_qualified'and q['profile']==profile and q['source_manifest_sha256']==old_sha
    assert q['actual_sessions']==q['actual_cast_requests']==4 and q['successful_self_casts']==2 and q['MP_rejections']==q['nonclan_target_rejections']==1 and q['recorded_original_owned_processes_terminal']
    assert not q['normal_product_modified']and q['source_overlays_used']and sha(path.parent/'raw.zip')==q['raw_archive_sha256']
    run=read(path.parent.parent/'receipt.json');build=Path(run['compile']);b=read(build/'receipt.json');assert sha(build/'receipt.json')==run['compile_receipt_sha256']
    rqpath=Path(run['resource_qualification']);rq=read(rqpath);assert sha(rqpath)==run['resource_qualification_sha256']and rq['status']=='isolated_candidate_original_resource_scope_qualified'
    assert [rq[k]for k in ['scene_regressions_passed','new_graphics_verified','ability_packet_component_passed','ability_world_cases_passed']]==[23,11,5,2]
    manifest=read(build/'candidate-inputs.json');assert sha(build/'candidate-inputs.json')==q['candidate_manifest_sha256']==rq['candidate_manifest_sha256']==b['candidate_manifest_sha256']
    for n,h in manifest['candidate_files'].items():assert sha(build/n)==h,n
    names=b['runtime_overrides'];assert len(names)==7
    current={n:sha(build/n)for n in names+['mod.rs','resource_probe.rs','ability_buff_component.rs','ability_world_probe.rs','live_probe.rs','ability-codecs.json']}
    if runtime_files is None:runtime_files=current
    else:assert runtime_files==current,'Debug and Release payload bytes differ'
    for info in b['products'].values():assert sha(info['path'])==info['sha256']
    proofs[profile]=dict(live=str(path),live_sha256=sha(path),resources=str(rqpath),resources_sha256=sha(rqpath),compile=str(build),compile_receipt_sha256=sha(build/'receipt.json'))
    builds[profile]=build
build=builds['Debug'];payload={'LiTo381-Modern/src/modern_lito/'+n:(build/n).read_bytes()for n in read(build/'receipt.json')['runtime_overrides']+['mod.rs']}
component=(build/'ability_buff_component.rs').read_text('utf-8-sig')
for name in ['native_skills','native_skill_selection','native_ability_buffs']:
    oldline=f'#[path="{name}.rs"]mod {name};';assert component.count(oldline)==1;component=component.replace(oldline,f'use lito_modern::modern_lito::{name};')
assert component.count('include_bytes!("ability-codecs.json")')==1
component=component.replace('include_bytes!("ability-codecs.json")','include_bytes!("fixtures/native-ability-codecs.json")')
resource=(build/'resource_probe.rs').read_text('utf-8-sig');assert resource.count('#[test]')==1
resource=resource.replace('#[test]','#[test]\n#[ignore="requires original resources and isolated finite-effect output"]',1)
payload.update({'LiTo381-Modern/tests/native_ability_buffs.rs':component.encode('utf-8'),'LiTo381-Modern/tests/native_ability_world.rs':(build/'ability_world_probe.rs').read_bytes(),'LiTo381-Modern/tests/native_additional_finite_resources.rs':resource.encode('utf-8'),'LiTo381-Modern/tests/native_ability_live.rs':(build/'live_probe.rs').read_bytes(),'LiTo381-Modern/tests/fixtures/native-ability-codecs.json':(build/'ability-codecs.json').read_bytes()})
assert len(payload)==13
new=[n for n in payload if not(ROOT/n).exists()];assert len(new)==7
meta=read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(meta)==8 and all(sha(ROOT/n)==h for n,h in meta.items())
out=V/'native-ability-finite-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
before={n:(ROOT/n).read_bytes()for n in set(payload)|set(meta)if(ROOT/n).is_file()};beforepins=archive(out/'before.zip',before);afterpins=archive(out/'proposal.zip',payload)
for n,h in inputs.items():assert sha(ROOT/n)==h,n
state=dict(status='prepared_ability_finite_application',applied=False,before_source_manifest_sha256=old_sha,before_files=beforepins,after_files=afterpins,new_files=new,candidate_qualifications=proofs,before_archive_sha256=sha(out/'before.zip'),proposal_archive_sha256=sha(out/'proposal.zip'),normal_validation_complete=False,launcher_artifacts_modified=False,normal_GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'application.json',state);(out/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
for n,data in payload.items():
    temp=out/Path(n).name;temp.write_bytes(data);os.replace(temp,ROOT/n);assert sha(ROOT/n)==afterpins[n];source[n]=inputs[n]=afterpins[n]
save(out/'applied-source-inputs.json',dict(sorted(source.items())));save(out/'applied-compiled-inputs.json',dict(sorted(inputs.items())))
assert(len(source),len(inputs))==(1047,1048)
state.update(status='applied_ability_finite_pending_normal_validation',applied=True,source_manifest_sha256=sha(out/'applied-source-inputs.json'),source_files=len(source),compiled_input_files=len(inputs));save(out/'application.json',state)
save(out.parent/'active-application.json',dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
work='已將兩版各4實際sessions核對的26敏捷／42力量請求、S6B-S6C權威接收與11原資源有限特效整合正常來源。新增7個來源／測試／fixture檔，共1047source／1048compiled inputs；兩版正常入口建置及當版測試待驗，舊44FE廣域／library／G06資格只留原版本。8秒／60秒與Release／G11界限不變，相鄰只讀且登入器產物獨立。正常GUI、實體輸入／音效及完整G01–G12仍未完成。'
for n in meta:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);identity=value['current_product_identity'];assert identity['source_manifest_sha256']==old_sha
        value['retained_44FE_product_identity']=identity;value['retained_44FE_goal_validation']=value.get('current_source_goal_validation')
        for key,item in value.items():
            if key.startswith('current_native')and isinstance(item,dict)and item.get('source_manifest_sha256')==old_sha:item.update(applies_to_current_source=False,retained_source_version_only=True)
        value['current_product_identity']=dict(source_manifest_sha256=state['source_manifest_sha256'],source_files=len(source),compiled_input_files=len(inputs),source_qualification_pending=True,Debug_normal_sha256=identity['Debug_normal_sha256'],Release_normal_sha256=identity['Release_normal_sha256'],existing_normal_source_manifest_sha256=old_sha,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        value['current_native_ability_finite_application']=dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256'],normal_validation_pending=True,candidate_qualifications=proofs)
        value['current_source_goal_validation']={key:dict(state='ability_finite_current_source_validation_pending',complete=False)for key in(value.get('current_source_goal_validation')or{})}
        for item in value.get('items',[]):
            if item['id'].startswith(('G01-','G03-','G06-','G08-')):item.update(state='ability_finite_current_source_validation_pending',complete=False)
        value.update(source_and_corresponding_QA_frozen=True,candidate_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'application.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='ability-finite-normal-validation',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');path.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(after_files={n:sha(ROOT/n)for n in meta},source_manifest_sha256=state['source_manifest_sha256']))
print(json.dumps(dict(application=str(out),source_manifest_sha256=state['source_manifest_sha256'],changed_files=len(payload),normal_validation_complete=False),ensure_ascii=False))
