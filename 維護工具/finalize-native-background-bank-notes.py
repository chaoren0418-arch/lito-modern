"""Record both terminal profiles and original dispatch evidence without claiming a scene."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
work=read(APP/'background-bank-product-current.json');assert not work['source_frozen_for_tests']
assert work['status']=='source1015_Debug_and_Release_each12_0_0_real_IMG_bank_and_core_verified_scene_pending'
profiles=work['profiles'];assert set(profiles)=={'Debug','Release'}
qfiles={};pids=[]
for profile,v in profiles.items():
    qpath=Path(v['qualification']);q=read(qpath);qfiles[profile+'-qualified.json']=qpath
    assert q['source_files']==1015 and q['passed']==12 and q['failed']==q['ignored']==0 and q['actual_test_executables']==4
    r=read(Path(v['run'])/'receipt.json');assert r['owned_process'] is None and r['compile_exit_code']==0
    pids.extend(x['pid'] for x in r['process_history'])
    assert q['all_archives_readback_verified']
for label,path in [('original-real',Path(work['native_real_IMG_original_qualification'])),('original-dispatch',Path(work['original_dispatch_qualification']))]:
    q=read(path);assert q['all_archives_readback_verified'];qfiles[label+'-qualified.json']=path
    r=read(path.parent/'run-receipt.json');assert r['owned_process'] is None
    pids.extend(x['pid'] for x in r['process_history'])
pins=read(Path(profiles['Debug']['run'])/'compiled-inputs.json')
assert pins==read(Path(profiles['Release']['run'])/'compiled-inputs.json') and len(pins)==1015
for n,h in pins.items():assert sha(ROOT/n)==h,n
previous=read(APP/'background-geometry-tests-20261016-122035-301179/compiled-inputs.json')
changed=[n for n in previous if previous[n]!=pins[n]];added=[n for n in pins if n not in previous]
assert set(changed)=={'LiTo381-Modern/src/modern_lito/mod.rs','LiTo381-Modern/src/modern_lito/native_background_pixels.rs'}
assert set(added)=={'LiTo381-Modern/src/modern_lito/native_background.rs','LiTo381-Modern/tests/native_background.rs','LiTo381-Modern/tests/fixtures/native-background-real-original.json'}
cmd='$ids=@('+','.join(map(str,pids))+');$p=@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in $ids});if($p.Count){$p|Select-Object ProcessId,Name,CreationDate|ConvertTo-Json -Compress;exit 1};"[]"'
c=subprocess.run(['powershell.exe','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert c.returncode==0,c.stdout
out=APP/('background-bank-completion-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
proof={'status':'source1015_both_profiles_bank_core_and_original_dispatch_completion_readback_verified',
    'observed_utc':datetime.now(timezone.utc).isoformat(),'path':str(out),'source_files':1015,
    'source_manifest_sha256':profiles['Debug']['source_manifest_sha256'],
    'actual_test_executables_per_profile':4,'passed_per_profile':12,'failed_per_profile':0,'ignored_per_profile':0,
    'terminal_tool_sessions':{'Debug':{'session':43971,'exit_code':0},'Release':{'session':85319,'exit_code':0}},
    'recorded_process_PIDs':pids,'all_recorded_owned_processes_gone':True,'CIM_query_stdout':c.stdout.decode('utf-8-sig').strip(),
    'original_real_IMG_frames':24,'original_real_IMG_pixel_calls':132,'original_dispatch_cases':263423,
    'modified_compiled_inputs':changed,'added_compiled_inputs':added,'background_world_adapter_integrated':False,
    'current_normal_mains_qualified':False,'current_normal_GUI_verified':False,'whole_goal_complete':False}
save(out/'receipt.json',proof)
lead=('2026-10-16來源1015：真IMG背景bank已加入純Rust，LTsurf優先且索引缺項才讀BASE Sprite，保留原RGB555／key字值並先驗整批矩形。'
    'Debug/Release各四實際x64測試EXE12過0敗0忽略，同43971/85319實際exit0、1015同來源887A9D54與source/test/raw封存讀回。'
    '原init/geometry/RGB555共4439bytes配五真IMG已隔離24完整平面/132calls核對，Rust bank對map303十二整frame完全吻合。'
    '完整scene gate、camera/viewport、clock、正常新主/GUI仍待；1005正常AFF5AF96/C7F36CC4與TW14保留原版資格，G01–G12未完成。')
detail=('389byte原第一輪物件selector另已實跑263423案：262144全16bit旗標組（兩map、兩完整option組）＋1279針對header、shade、單項option與uint32 counter案。'
    '九條route及其實際引數逐項核對；普通4BE000僅map303傳gate低byte1，非此map為0。roof shade即使31仍走另一分支，不可憑不透明度推定gate。'
    '這是記錄callee的分派驗證，沒有執行完整物件像素或證明整個scene gate。只有私有尾端附加1byte ret，原389bytes除14允許重定位不改；所有定義的guard及輸入不變。')
clock=('背景回呼先增相位再排64ms；static已找到直接開始、取消，以及取消後立即重啟路徑，所有具名直接phase寫入只在該callback。'
    '這尚未排除間接寫入，未執行生命週期，不能據此把phase在每次GAME歸零或做elapsed補步。'
    '原生LiTo/LTmap/303仍缺目錄，未把BASE map/303加入fallback；真IMG路由不代表地圖路由。')
for n in ['README.md','docs/ARCHITECTURE.zh-TW.md']:
    f=ROOT/n;s=f.read_text(encoding='utf-8-sig');assert lead not in s
    title,body=s.split('\n',1);f.write_text(title+'\n\n'+lead+'\n'+body,encoding='utf-8')
prefix='驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/'
for n in ['docs/HISTORY.md','docs/NATIVE-FLOOR-GATE.zh-TW.md']:
    f=ROOT/n;s=f.read_text(encoding='utf-8-sig');assert lead not in s
    f.write_text(s+'\n\n## 2026-10-16 真IMG背景bank與物件分派\n\n'+lead+'\n\n'+detail+'\n\n'+clock+
        '\n\n[Debug限定資格](../'+prefix+Path(profiles['Debug']['run']).name+'/qualified.json)；[Release限定資格](../'+prefix+Path(profiles['Release']['run']).name+'/qualified.json)。'+
        '\n\n[原真IMG完整背景平面](../驗證紀錄/current/native-background-real-original/20261016-123005-086741/qualified.json)；[原物件分派](../驗證紀錄/current/native-background-dispatch-original/20261016-124150-153377/qualified.json)。\n',encoding='utf-8')
for label,p in qfiles.items():(out/label).write_bytes(p.read_bytes())
(out/'finalizer.py').write_bytes(Path(__file__).read_bytes())
files={f.name:sha(f) for f in out.iterdir() if f.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(out/n,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    for n,h in files.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
save(APP/'background-bank-completion-current.json',{**proof,'files':files,'raw_archive_sha256':sha(out/'raw.zip'),'archive_readback_verified':True})
print(json.dumps({'status':proof['status'],'path':str(out),'all_recorded_owned_processes_gone':True},ensure_ascii=False))
