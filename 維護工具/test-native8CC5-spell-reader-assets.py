"""Compare complete four-profile assets with unchanged original loader bodies."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current';T=ROOT/'維護工具'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
measurement=V/'native8CC5-scoped-catalog-v3/measurement-20261018-131100-530427';m=read(measurement/'receipt.json');assert m['owned_process'] is None
assert m['status']=='measured_instrumented_resource_open_costs_only'
candidate=T/'candidates/native8CC5-scoped-catalog-v3';prep=read(candidate/'preparation.json')
for pin in [*prep['original_files'].values(),*prep['candidate_files'].values()]:assert sha(pin['path'])==pin['sha256']
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native8CC5-scoped-catalog-v3'/('asset-comparison-'+stamp);out.mkdir()
runtime=P/'target/current/native8CC5-scoped-catalog-v3'/('asset-comparison-'+stamp);runtime.mkdir()
signature=r'''
fn diagnostic_signature(a:&LightArrowAssets)->serde_json::Value {
    let sprites:Vec<_>=a.sprites.iter().map(|(key,s)| {
        let frames:Vec<_>=s.decoded.frames.iter().map(|f|serde_json::json!({
            "x":f.x_offset,"y":f.y_offset,"width":f.width,"height":f.height,"frame_type":f.frame_type,
            "unknown1":f.unknown1,"unknown2":f.unknown2,"rgba_bytes":f.rgba.len(),
            "rgba_sha256":format!("{:X}",Sha256::digest(&f.rgba)),
            "fragments_sha256":format!("{:X}",Sha256::digest(format!("{:?}",f.fragments).as_bytes()))})).collect();
        serde_json::json!({"gfx":key.gfx,"image":key.image,"provenance":format!("{:?}",s.provenance),
            "palette":s.decoded.has_palette,"block_metadata":s.decoded.block_table_metadata,"frames":frames})
    }).collect();
    let sounds:Vec<_>=a.sound.ids().map(|id|{let s=a.sound.get(id).unwrap();serde_json::json!({"id":id,"wav_sha256":format!("{:X?}",s.sha256()),"file_bytes":s.file_bytes(),"pcm_sha256":format!("{:X}",Sha256::digest(s.pcm_bytes())),"format":format!("{:?}",s.format())})}).collect();
    serde_json::json!({"effect_gfx":a.effect.gfx,"effect_animation":format!("{:?}",a.effect.sequence.animation),
        "effect_clock":format!("{:?}/{:?}/{}",a.effect.sequence.starts,a.effect.sequence.ends,a.effect.sequence.total),
        "impact_gfx":a.impact.gfx,"impact_animation":format!("{:?}",a.impact.sequence.animation),
        "impact_clock":format!("{:?}/{:?}/{}",a.impact.sequence.starts,a.impact.sequence.ends,a.impact.sequence.total),
        "caster_count":a.casters.len(),"group":format!("{:?}",a.group),"sprites":sprites,"sounds":sounds,
        "catalog_sha256":format!("{:X?}",a.catalog_sha256),"encoded":a.encoded,"rgba":a.rgba})
}
'''
for name,source in [('baseline',P/'src/modern_lito/light_arrow_effect.rs'),('candidate',candidate/'light_arrow_effect.rs')]:
    prefix=source.read_bytes();(out/(name+'.original.rs')).write_bytes(prefix)
    suffix=signature+'''
pub fn diagnostic_profile_signatures(root:&Path)->std::result::Result<serde_json::Value,String> {
    let cancel=AtomicBool::new(false);let context=SharedActionCatalog::load(root,&cancel).map_err(|e|e.to_string())?;
    let mut results=Vec::new();
'''
    if name=='candidate':suffix+='    let mut slot=None;\n'
    suffix+='    for gfx in [167,1797,1799,1583] {\n'
    suffix+=('        let a=LightArrowAssets::load_effect_profile_with_reader(&context,gfx,&cancel,&mut slot)' if name=='candidate' else '        let a=LightArrowAssets::load_selected_profile(&context,BTreeSet::new(),gfx,&cancel)')+'.map_err(|e|e.to_string())?;\n        results.push(diagnostic_signature(&a));\n    }\n'
    if name=='candidate':suffix+='''
    assert!(slot.is_some());
    assert!(matches!(LightArrowAssets::load_effect_profile_with_reader(&context,u32::MAX,&cancel,&mut slot),Err(LightArrowError::Unsupported(_))));
    assert!(slot.is_none());
    let recovered=LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot).map_err(|e|e.to_string())?;
    assert_eq!(diagnostic_signature(&recovered),results[0]);
    cancel.store(true,Ordering::Release);
    assert!(matches!(LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot),Err(LightArrowError::Cancelled)));
    assert!(slot.is_none());cancel.store(false,Ordering::Release);
    LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot).map_err(|e|e.to_string())?;
    slot.as_mut().unwrap().root=PathBuf::from("deliberately-wrong-scope");
    assert!(matches!(LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot),Err(LightArrowError::Resource(ResourceError::UnsafePath(_)))));
    assert!(slot.is_none());
'''
    suffix+='    Ok(serde_json::Value::Array(results))\n}\n'
    (out/(name+'.rs')).write_bytes(prefix+b'\n'+suffix.encode('utf-8'))
(out/'shared_action_catalog.rs').write_bytes((candidate/'shared_action_catalog.rs').read_bytes())
wrapper='mod modern_lito {\npub use lito_modern::modern_lito::*;\n'+''.join('#[path = '+json.dumps((out/(name+'.rs')).as_posix(),ensure_ascii=False)+']\npub mod '+name+';\n' for name in ['shared_action_catalog','baseline','candidate'])+'}\n'
wrapper+='''fn main()->Result<(),String>{
    let root=std::path::Path::new("D:/lineage開發/0-客戶端");
    let baseline=modern_lito::baseline::diagnostic_profile_signatures(root)?;
    let candidate=modern_lito::candidate::diagnostic_profile_signatures(root)?;
    assert_eq!(baseline,candidate,"complete asset/fragment/animation/PCM matrix changed");
    std::fs::write(std::env::args_os().nth(1).unwrap(),serde_json::to_vec_pretty(&serde_json::json!({"status":"four_full_profiles_identical_and_scoped_failure_guards_passed","profiles":[167,1797,1799,1583],"signatures":candidate,"cancel_clears_reader":true,"invalid_profile_clears_reader":true,"wrong_root_clears_reader":true,"recovery_matches_original":true,"audio_device_opened":false,"network_sessions":0,"normal_product_source_modified":false})).unwrap()).map_err(|e|e.to_string())?;Ok(())
}\n'''
(out/'compare.rs').write_text(wrapper,encoding='utf-8',newline='\n');(out/'runner.py').write_bytes(Path(__file__).read_bytes())
args=list(m['process_history'][1]['arguments']);args[1]=str(out/'compare.rs');args[args.index('--crate-name')+1]='spell_reader_asset_comparison';exe=runtime/'compare.exe';args[args.index('-o')+1]=str(exe)
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='running',runner_identity=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,measurement=str(measurement),measurement_receipt_sha256=sha(measurement/'receipt.json'),candidate_preparation_sha256=sha(candidate/'preparation.json'),source_files={p.name:sha(p) for p in out.iterdir() if p.is_file()},product=str(exe),normal_product_source_modified=False,new_sessions=0)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
print('ASSET_COMPARISON='+str(out),flush=True)
try:
    for label,command in [('compile',args),('compare',[str(exe),str(out/'comparison.json')])]:
        with(out/(label+'.stdout.txt')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
            child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,arguments=command);state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            try:job['exit_code']=child.wait(timeout=180)
            except subprocess.TimeoutExpired:
                assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];child.terminate();job['exit_code']=child.wait();raise
            state['owned_process']=None;save(out/'receipt.json',state);assert job['exit_code']==0,(label,job['exit_code'])
    assert read(out/'comparison.json')['status']=='four_full_profiles_identical_and_scoped_failure_guards_passed'
    for pin in prep['original_files'].values():assert sha(pin['path'])==pin['sha256']
    state.update(status='four_full_profiles_and_scoped_reader_guards_passed',product_sha256=sha(exe),comparison_sha256=sha(out/'comparison.json'))
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(comparison=str(out),profiles=4,full_pixels_fragments_animation_and_PCM_identical=True,scoped_failure_guards=True,normal_product_source_modified=False),ensure_ascii=False))
