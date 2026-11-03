"""Append the completed bounded scope and retain preparation observer failure."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
run=Path(read(APP/'background-geometry-tests-current.json')['path']);q=read(run/'qualified.json')
assert q['source_files']==1012 and q['passed']==10 and q['actual_test_executables']==3
assert q['all_archives_readback_verified']
r=read(run/'receipt.json');assert r['owned_process'] is None and r['source_inputs_unchanged']
pids=[v['pid'] for v in r['process_history']]
command='$ids=@('+','.join(map(str,pids))+');$p=@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in $ids});if($p.Count){$p|Select-Object ProcessId,Name,CreationDate|ConvertTo-Json -Compress;exit 1};"[]"'
c=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert c.returncode==0,c.stdout
save(run/'terminal-process-observation.json',{'observed_utc':datetime.now(timezone.utc).isoformat(),
    'tool_session':96668,'observed_tool_terminal_exit_code':0,
    'tool_terminal_output_status':'Debug_background_geometry_pixel_and_floor_tests_passed',
    'cargo_and_test_PIDs':pids,'all_recorded_cargo_and_tests_gone':True,
    'CIM_query_stdout':c.stdout.decode('utf-8-sig').strip(),'qualification_sha256':sha(run/'qualified.json')})
old=read(APP/'background-pixel-tests-20261016-115714-009940/compiled-inputs.json')
new=read(run/'compiled-inputs.json')
changed=[n for n in old if old[n]!=new[n]];added=[n for n in new if n not in old]
assert changed==['LiTo381-Modern/src/modern_lito/mod.rs'] and len(added)==3
save(run/'source-delta.json',{'previous_compiled_inputs':1009,'current_compiled_inputs':1012,'changed':changed,'added':added})
failure=APP/'background-geometry-runner-preparation-failure'
save(failure/'review.json',{'status':'preparation_string_guard_failed_before_any_runner_was_written',
    'attempts':2,'cause':'Expected a blank line before def stable; original script has a single LF.',
    'original_preparer_sha256':sha(failure/'preparer.py'),'product_failure':False,
    'test_conditions_weakened':False,'corrected_preparer':str(ROOT/'維護工具/prepare-native-background-geometry-tests.py')})
lead=('2026-10-16來源1012：純Rust背景geometry已補齊，2688組原輸入與17472次有序pixel calls全部吻合；'
    '連同原背景135168 channel words／1584整frame與64地板案，Debug三實際EXE10過0敗0忽略，'
    '同96668實際exit0、source/test/raw封存讀回。保留signed負奇數與552溢出判斷／522條件裁切。'
    '正常場景、真IMG、完整overlay gate、modern viewport adapter與最新Release／正常主／GUI仍待；'
    '1005正常AFF5AF96/C7F36CC4及TW14保留變更前資格，完整G01–G12未完成。')
detail=('原source cell明示使用NativeDrawCell.half_x，不能將server whole-cell X直接代入；'
    '本API只規劃原744×552座標域，未接modern 800×600 camera。'
    '866189-byte純數值fixture F4B9F257由已核對原run-raw匯出，不含原指令。'
    '兩次runner文字準備守衛因誤猜空行失敗，皆發生在產生runner前，原preparer及原因保留；'
    '修正LF邊界後才建置，沒有削弱測試。')
boundary=('已更正原先before-all-bodies假設：原488C34第一輪按cell畫地板及同cell物件清單，'
    '488F49才在map303呼叫背景；物件plain RLE可走4BE000，不能用所有RGBA不透明像素代替gate。'
    '原map loader明示LiTo/LTMap的s32失敗再seg；目前原生303目錄不存在，BASE map/303雖有30份S32，'
    '尚未證明可fallback，未增加替代路徑。這是static來源觀察，非原分派實跑或正常303 GUI。'
    '重建版與登入器產物獨立，未替換或部署登入器JAR。')
link='驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/'+run.name+'/qualified.json'
for n in ['README.md','docs/ARCHITECTURE.zh-TW.md']:
    p=ROOT/n;s=p.read_text(encoding='utf-8-sig');assert lead not in s
    a,b=s.split('\n',1);p.write_text(a+'\n\n'+lead+'\n'+b,encoding='utf-8')
for n in ['docs/HISTORY.md','docs/NATIVE-FLOOR-GATE.zh-TW.md']:
    p=ROOT/n;s=p.read_text(encoding='utf-8-sig');assert lead not in s
    p.write_text(s+'\n\n## 2026-10-16 Rust背景geometry與場景來源邊界\n\n'+lead+'\n\n'+detail+'\n\n'+boundary+
        '\n\n[當版限定資格](../'+link+')；[原排序與來源觀察](../驗證紀錄/current/native-background-scene-boundary/20261016-122149-466090/observation.json)。\n',encoding='utf-8')
files=['terminal-process-observation.json','source-delta.json','qualified.json']
with zipfile.ZipFile(run/'completion-notes.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(run/n,n)
with zipfile.ZipFile(run/'completion-notes.zip') as z:
    for n in files:assert hashlib.sha256(z.read(n)).hexdigest().upper()==sha(run/n)
print(json.dumps({'status':'source1012_notes_and_completion_evidence_readback_verified','source_delta':{'changed':len(changed),'added':len(added)},'all_owned_recorded_processes_gone':True}))
