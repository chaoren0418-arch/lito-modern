"""Apply the validated eight runtime modules and their resource/lifetime regressions."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
old=V/'native-legacy-audio-normal/validation-20261018-104256-823814';source=read(old/'source-inputs.json');inputs=read(old/'compiled-inputs.json');assert(len(source),len(inputs))==(1040,1041)
old_sha=sha(old/'source-inputs.json');q=read(ROOT/'工作佇列.json');assert not q['active_workloads'] and q['source_and_corresponding_QA_frozen'] and q['current_product_identity']['source_manifest_sha256']==old_sha
for n,h in inputs.items():assert sha(ROOT/n)==h
candidate=ROOT/'維護工具/candidates/native8CC5-scoped-catalog-v3';prep=read(candidate/'preparation.json');assert len(prep['candidate_files'])==8
qualified=V/'native8CC5-catalog-audio/run-20261018-131706-111384/qualified-20261018-131859-813082/qualified.json';proof=read(qualified)
assert proof['status']=='independently_qualified_catalog_candidate_nonempty_audio_refused_preflight' and proof['profile']=='Debug' and proof['scenario']=='refused'
assert proof['actual_sessions']==8 and proof['original_exit_code']==0 and proof['recorded_owned_all_terminal'] and proof['source_overlays_used'] and not proof['normal_source_bodies_unchanged']
assert proof['unplayed_source_audio_rejection_verified'] and proof['new_scene_own_sound_events_delivered_once_each']
with zipfile.ZipFile(qualified.parent/'raw.zip') as z:
    assert sha(qualified.parent/'raw.zip')==proof['raw_archive_sha256']
    for n,h in proof['raw_files'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
measurement=V/'native8CC5-scoped-catalog-v3/measurement-20261018-131100-530427/cost-observation.json';observed=read(measurement)
assert observed['recorded_owned_all_terminal'] and observed['observed_wrapper_terminal_exit_code']==0
assert all(r['first_frame_us']<8_000_000 and r['unchanged_normal_baseline_pixels'] for r in observed['rows'])
asset=V/'native8CC5-scoped-catalog-v3/asset-comparison-20261018-131514-909089';ar=read(asset/'receipt.json');ap=read(asset/'comparison.json')
assert ar['status']=='four_full_profiles_and_scoped_reader_guards_passed' and ar['owned_process'] is None and all(j['exit_code']==0 for j in ar['process_history']) and sha(asset/'comparison.json')==ar['comparison_sha256']
assert all(ap[k] for k in ['cancel_clears_reader','invalid_profile_clears_reader','wrong_root_clears_reader','recovery_matches_original'])
component=V/'native8CC5-scoped-catalog-v2/component-20261018-130806-089706';cr=read(component/'receipt.json');assert cr['status']=='both_profiles_catalog_component_13_passed_each' and cr['owned_process'] is None
for profile in ['Debug','Release']:assert '13 passed; 0 failed; 0 ignored' in (component/(profile+'-tests.stdout.txt')).read_text('utf-8-sig')
payload={}
for n,pin in prep['candidate_files'].items():
    assert sha(pin['path'])==pin['sha256'] and sha(prep['original_files'][n]['path'])==prep['original_files'][n]['sha256']
    payload[Path(prep['original_files'][n]['path']).relative_to(ROOT).as_posix()]=Path(pin['path']).read_bytes()
catalog_key='LiTo381-Modern/src/modern_lito/shared_action_catalog.rs'
catalog=(component/'shared_action_catalog.tests.rs').read_text('utf-8-sig');body=payload[catalog_key].decode('utf-8');cut=body.rfind('\n}')
assert catalog.startswith(body[:cut]) and catalog.endswith(body[cut:])
assert catalog.count('fn model_slot_')==4 and catalog.count('fn renderer_')==2
catalog=catalog.replace('target/modern-lito-shared-catalog-tests','target/current/modern-lito-shared-catalog-tests')
payload[catalog_key]=catalog.encode('utf-8')
light_key='LiTo381-Modern/src/modern_lito/light_arrow_effect.rs';body=payload[light_key].decode('utf-8');cut=body.rfind('\n}')
test=r'''
    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; exact four profiles and scoped reader lifecycle, memory only"]
    fn actual_shared_four_profiles_preserve_all_assets_and_discard_failed_reader() {
        let root=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
        let cancel=AtomicBool::new(false);let context=SharedActionCatalog::load(Path::new(&root),&cancel).unwrap();let mut slot=None;
        for gfx in [167,1797,1799,1583] {
            let original=LightArrowAssets::load_selected_profile(&context,BTreeSet::new(),gfx,&cancel).unwrap();
            let reused=LightArrowAssets::load_effect_profile_with_reader(&context,gfx,&cancel,&mut slot).unwrap();
            assert_eq!(original.catalog_sha256,reused.catalog_sha256);assert_eq!(original.encoded,reused.encoded);assert_eq!(original.rgba,reused.rgba);
            assert_eq!(original.effect.gfx,reused.effect.gfx);assert_eq!(original.impact.gfx,reused.impact.gfx);
            for (a,b) in [(&original.effect,&reused.effect),(&original.impact,&reused.impact)] {
                assert_eq!(a.sequence.animation,b.sequence.animation);assert_eq!(a.sequence.starts,b.sequence.starts);assert_eq!(a.sequence.ends,b.sequence.ends);assert_eq!(a.sequence.total,b.sequence.total);
            }
            assert!(original.casters.is_empty()&&reused.casters.is_empty());assert_eq!(original.sprites.len(),reused.sprites.len());
            for (key,a) in &original.sprites {let b=&reused.sprites[key];assert_eq!(a.decoded,b.decoded);assert_eq!(format!("{:?}",a.provenance),format!("{:?}",b.provenance));}
            assert_eq!(format!("{:?}",original.group),format!("{:?}",reused.group));
            assert_eq!(original.sound.ids().collect::<Vec<_>>(),reused.sound.ids().collect::<Vec<_>>());
            for id in original.sound.ids() {let a=original.sound.get(id).unwrap();let b=reused.sound.get(id).unwrap();assert_eq!(a.sha256(),b.sha256());assert_eq!(a.pcm_bytes(),b.pcm_bytes());assert_eq!(a.file_bytes(),b.file_bytes());assert_eq!(a.format(),b.format());}
        }
        assert!(slot.is_some());assert!(matches!(LightArrowAssets::load_effect_profile_with_reader(&context,u32::MAX,&cancel,&mut slot),Err(LightArrowError::Unsupported(_))));assert!(slot.is_none());
        LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot).unwrap();cancel.store(true,Ordering::Release);
        assert!(matches!(LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot),Err(LightArrowError::Cancelled)));assert!(slot.is_none());cancel.store(false,Ordering::Release);
        LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot).unwrap();slot.as_mut().unwrap().root=PathBuf::from("deliberately-wrong-scope");
        assert!(matches!(LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot),Err(LightArrowError::Resource(ResourceError::UnsafePath(_)))));assert!(slot.is_none());
    }
'''
payload[light_key]=(body[:cut]+test+body[cut:]).encode('utf-8')
assert len(payload)==8
previous=V/'native8CC5-scene-cost/checkpoint-20261018-130921-398700/readback.json';meta=read(previous)['after_files'];assert len(meta)==8 and all(sha(ROOT/n)==h for n,h in meta.items())
out=V/'native-scoped-catalog-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
before={n:sha(ROOT/n) for n in [*payload,*meta]}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
after={n:hashlib.sha256(b).hexdigest().upper() for n,b in payload.items()}
with zipfile.ZipFile(out/'proposal.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,b in payload.items():z.writestr(n,b)
with zipfile.ZipFile(out/'proposal.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in after.items())
pins={str(path):sha(path) for path in [qualified,measurement,asset/'receipt.json',asset/'comparison.json',component/'receipt.json',component/'shared_action_catalog.tests.rs',candidate/'preparation.json']}
state=dict(status='prepared_scoped_catalog_normal_application',applied=False,before_source_manifest_sha256=old_sha,before_files=before,after_files=after,new_files=[],qualification_pins=pins,before_archive_sha256=sha(out/'before.zip'),proposal_archive_sha256=sha(out/'proposal.zip'),normal_validation_complete=False,diagnostic_request_API_applied=False,resource_instrumentation_applied=False,normal_GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'application.json',state);(out/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
for n,h in inputs.items():assert sha(ROOT/n)==h
for n,b in payload.items():
    target=ROOT/n;assert target.resolve(strict=True).is_relative_to(P);tmp=out/Path(n).name;tmp.write_bytes(b);os.replace(tmp,target);assert sha(target)==after[n];source[n]=inputs[n]=after[n]
save(out/'applied-source-inputs.json',dict(sorted(source.items())));save(out/'applied-compiled-inputs.json',dict(sorted(inputs.items())))
state.update(status='applied_scoped_catalog_pending_normal_build_and_current_source_validation',applied=True,source_manifest_sha256=sha(out/'applied-source-inputs.json'),source_files=1040,compiled_input_files=1041)
save(out/'application.json',state);save(out.parent/'active-application.json',dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
work='已將同Model共用動作目錄及同次法術準備共用Sprite讀取器整合正常來源八檔；保留繪圖器32MiB／效果8MiB、原取消與資源界限，加入六個目錄生命週期回歸與一個四法術完整原資源回歸。候選Debug拒絕分支8sessions已獨立核對舊音效零交付與恢復各一次；固定場景三次首Frame7.087／7.415／7.995秒僅為候選成本量測。正常新來源兩版唯一入口建置、17整合與4選定library群組待驗，之後須重驗當版G06。8CC5正常130／控制器兩版96及Release音效48sessions只保留原來源資格；正常GUI／實體音效、完整G06與G01–G12仍未完成。診斷API及資源計時未套入；相鄰只讀且產物獨立。'
for n in meta:
    path=ROOT/n
    if path.suffix=='.json':
        x=read(path);old_identity=x['current_product_identity'];assert old_identity['source_manifest_sha256']==old_sha
        x['retained_source_8CC5_product_identity']=old_identity;x['retained_source_8CC5_goal_validation']=x.get('current_source_goal_validation')
        for key,item in x.items():
            if key.startswith('current_native') and isinstance(item,dict) and item.get('source_manifest_sha256')==old_sha:item.update(applies_to_current_source=False,retained_source_version_only=True)
        x['current_product_identity']=dict(source_manifest_sha256=state['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,source_qualification_pending=True,Debug_normal_sha256=old_identity['Debug_normal_sha256'],Release_normal_sha256=old_identity['Release_normal_sha256'],existing_normal_source_manifest_sha256=old_sha,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        x['current_native_scoped_catalog_application']=dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256'],normal_validation_pending=True)
        x['current_source_goal_validation']={key:dict(state='scoped_catalog_current_source_validation_pending',complete=False) for key in (x.get('current_source_goal_validation') or {})}
        for item in x.get('items',[]):
            if item['id'].startswith(('G01-','G03-','G06-')):item.update(state='scoped_catalog_current_source_validation_pending',complete=False)
        x.update(source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'application.json'),goal_complete=False,whole_goal_complete=False)
        x['current_focus']=dict(item='scoped-catalog-normal-build-and-final-source-G06',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(path,x)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep;path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'tracker-readback.json',dict(after_files={n:sha(ROOT/n) for n in meta},source_manifest_sha256=state['source_manifest_sha256']))
print(json.dumps(dict(application=str(out),source_manifest_sha256=state['source_manifest_sha256'],changed_files=8,normal_validation_complete=False),ensure_ascii=False))
