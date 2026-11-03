"""Run a staged private x86 clock oracle only after the current GUI has ended."""
from pathlib import Path
from datetime import datetime,timezone
from collections import Counter,defaultdict
import csv,hashlib,json,os,struct,subprocess,sys,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'actual-tests-20261016-071612-845832/normal-qualified'
PROJECT=ROOT/'LiTo381-Modern'
OUT=Path(sys.argv[1]).resolve(strict=True);assert OUT.is_relative_to(ROOT/'驗證紀錄/current/native-water-clock-original')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
prepared=read(OUT/'prepared.json');assert prepared['status']=='prepared_private_clock_fragments_not_built_or_executed'
assert len(prepared['functions'])==8 and not (OUT/'receipt.json').exists()
for name,value in prepared['input_files'].items():assert sha(OUT/name)==value,name
pins=read(NORMAL/'source-inputs.json');assert len(pins)==999
def stable():
    for name,value in pins.items():assert sha(ROOT/name)==value,name
    for value in read(NORMAL/'qualified.json')['profiles'].values():assert sha(Path(value['path']))==value['sha256']
    assert sha(Path(prepared['original_PE']))==prepared['original_PE_sha256']
def no_gui():
    g=read(NORMAL/'GUI-current.json')
    assert g['status']=='qualified_current_lighting_TW14_GUI_cases' and not g['source_and_corresponding_tests_frozen']
    assert read(Path(g['cohort'])/'qualified.json')['scenarios']==14
    quoted=str(ROOT).replace("'","''")
    command="$r='"+quoted+"';$p=@(Get-CimInstance Win32_Process|Where-Object {($_.Name -in @('cargo.exe','rustc.exe')) -or ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase) -and ($_.Name -like 'LiToModern*' -or $_.ExecutablePath -match '\\\\deps\\\\.*\\.exe$'))});if($p.Count){exit 1}"
    result=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    assert result.returncode==0,'Owned GUI/Java/compiler/test processes still live'
no_gui();stable()
runtime=PROJECT/'target/current/native-water-clock-oracle';runtime.mkdir(exist_ok=True)
helper=runtime/'oracle.exe';retention=runtime/'last-retention.json'
if helper.exists():
    previous=read(retention);assert sha(helper)==previous['exe_sha256']
    archive=Path(previous['archive']);assert sha(archive)==previous['archive_sha256']
    with zipfile.ZipFile(archive) as z:assert digest(z.read('native-x86-oracle.exe'))==previous['exe_sha256']
command=read(ROOT/'驗證紀錄/current/native-lighting-source/build-command.json')['args']
command=[part.replace(str(ROOT/'維護工具/oracle/NativeLightingPrimitiveOracle.cpp'),str(OUT/'oracle.cpp')).replace('native-lighting-source-oracle','native-water-clock-oracle') for part in command]
assert str(OUT/'oracle.cpp') in command
save(OUT/'build-command.json',{'args':command,'cwd':str(runtime)})
(OUT/'auditor.py').write_bytes(Path(__file__).read_bytes())
(OUT/'normal-qualified.json').write_bytes((NORMAL/'qualified.json').read_bytes())
(OUT/'product-inputs.json').write_bytes((NORMAL/'source-inputs.json').read_bytes())
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
receipt={'status':'compiling_private_original_clock','started_utc':datetime.now(timezone.utc).isoformat(),
 'native_execution':False,'product_source_changes':0,'product_inputs':999,'game_sessions':0,'whole_goal_complete':False,'owned_process':None}
persist=lambda:save(OUT/'receipt.json',receipt)
def execute(args,name):
    with (OUT/name).open('wb') as stdout,(OUT/(name+'.stderr')).open('wb') as stderr:
        child=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        receipt['owned_process']={'pid':child.pid,'args':args};persist()
        code=child.wait();receipt['owned_process']=None;receipt[name+'_exit_code']=code;persist()
    assert code==0,(name,code)
persist()
try:
    execute(command,'build.log');stable();no_gui()
    binary=helper.read_bytes();pe=int.from_bytes(binary[60:64],'little')
    assert binary[:2]==b'MZ' and binary[pe+4:pe+6]==b'\x4c\x01' and binary[pe+24:pe+26]==b'\x0b\x01'
    receipt.update(status='executing_private_original_clock',native_execution=True);persist()
    execute([str(helper),str(OUT)],'oracle.json');native=read(OUT/'oracle.json')
    assert native=={'status':'original_water_clock_fragments_executed','phase_vectors':8656,'scroll_vectors':9668,
        'queue_scenarios':15,'native_allocation_growths_in_tie':2,'guard_failures':0,'game_sessions':0,'whole_client_executed':False}
    addresses={int(a,16):int(b,16) for a,b in (line.split() for line in (OUT/'addresses.tsv').read_text().splitlines())}
    for name,meta in prepared['functions'].items():
        original=(OUT/(name+'-original.bin')).read_bytes();relocated=(OUT/(name+'-relocated.bin')).read_bytes()
        assert digest(original)==meta['sha256'] and len(original)==len(relocated)==meta['bytes']
        allowed={i for r in meta['relocations'] for i in range(r['offset'],r['offset']+4)}
        assert all(a==b or i in allowed for i,(a,b) in enumerate(zip(original,relocated)))
        for r in meta['relocations']:
            expected=addresses[r['target']]
            if r['type']=='relative':expected=(expected-addresses[meta['start']]-r['next'])&0xffffffff
            assert struct.unpack_from('<I',relocated,r['offset'])[0]==expected
    phase_rows=list(csv.DictReader((OUT/'phase-vectors.csv').open(newline='',encoding='ascii')))
    expected_inputs=Counter((flag,c,p) for flag in [0,1] for c in range(100) for p in range(-20,22))
    expected_inputs.update((flag,99,21) for flag in range(256))
    actual_inputs=Counter()
    for row in phase_rows:
        flag,c,p,co,po,queued,deadline,reads=map(int,row.values());actual_inputs[(flag,c,p)]+=1
        assert (co,po,queued,deadline,reads)==((0 if c==99 else c+1,-20 if p>20 else p+1,1,12445,1) if flag else (0,0,0,0,0))
    assert actual_inputs==expected_inputs and len(phase_rows)==8656
    ds=[-640,-13,-6,-1,0,1,6,13,640];ys=[-420,-13,-6,-1,0,1,6,13,420]
    directions=[(4,2),(0,4),(-4,2),(-8,0),(-4,-2),(0,-4),(4,-2),(8,0)]
    expected_inputs=Counter((x,419,dx,0) for x in range(640) for dx in ds)
    expected_inputs.update((639,y,0,dy) for y in range(420) for dy in ys)
    expected_inputs.update((x,y,dx,dy) for x in [0,1,638,639] for y in [0,1,418,419] for dx,dy in directions)
    actual_inputs=Counter();scroll_rows=list(csv.DictReader((OUT/'scroll-vectors.csv').open(newline='',encoding='ascii')))
    for row in scroll_rows:
        x,y,dx,dy,ox,oy=map(int,row.values());actual_inputs[(x,y,dx,dy)]+=1
        assert (ox,oy)==((x+dx+640)%640,(y+dy+420)%420)
    assert actual_inputs==expected_inputs and len(scroll_rows)==9668
    queues=defaultdict(list)
    for row in csv.DictReader((OUT/'queue-vectors.csv').open(newline='',encoding='ascii')):
        name=row['scenario'];assert int(row['index'])==len(queues[name]);queues[name].append((int(row['deadline']),row['callback'],int(row['owner'])))
    kept=[(4000+i,'keep'+str(i),i+1) for i in range(9)];aux=(11000,'map_aux',0xffffffff)
    expected_queues={
      'tie':[(1025,'note1',104),(1050,'note1',101),(1050,'note2',102),(1050,'note1',103),(1075,'note2',105)],
      'cancel':[(1050,'note1',22),(1050,'note2',11)],
      'map_before':[(1050,'phase',0),(1055,'note1',88),(1090,'phase',0),(1095,'phase',42)]+kept,
      'map_filtered':kept+[aux],'map_aux_repeat':kept+[aux],
      'map_rescheduled':[(2050,'phase',0)]+kept+[aux],'map_tick':[(2150,'phase',0)]+kept+[aux],
      'disabled_cancel':[],'disabled_callback':[],
      'late':[(1107,'phase',0),(1114,'phase',0)],'late_not_due':[(1107,'phase',0),(1114,'phase',0)],
      'late_first':[(1114,'phase',0),(1207,'phase',0)],'late_again':[(2100,'phase',0),(2100,'phase',0)],
      'dispatch_equal':[],'u32_wrap':[(34,'note1',9),(0xfffffff0,'note2',10)]}
    states={}
    for row in csv.DictReader((OUT/'state-vectors.csv').open(newline='',encoding='ascii')):
        name=row.pop('scenario');assert name not in states;states[name]=tuple(map(int,row.values()))
    expected_states={
      'tie':(0,0,0,0,5,1000,5,0),'cancel':(0,0,0,0,2,1000,5,0),
      'map_before':(19,98,11,419,13,1000,13,0),'map_filtered':(19,98,11,419,10,1000,14,0),
      'map_aux_repeat':(19,98,11,419,10,1000,14,0),'map_rescheduled':(19,98,11,419,11,2000,15,0),
      'map_tick':(20,99,11,419,11,2050,17,1),
      'disabled_cancel':(7,99,0,0,0,1000,2,0),'disabled_callback':(0,0,0,0,0,1000,2,0),
      'late':(2,2,0,0,2,1021,5,1),'late_not_due':(2,2,0,0,2,1106,6,0),
      'late_first':(3,3,0,0,2,1107,8,1),'late_again':(5,5,0,0,2,2000,11,1),
      'dispatch_equal':(0,0,0,0,0,1050,4,1),'u32_wrap':(0,0,0,0,2,0xfffffff0,2,0)}
    assert states==expected_states,(states,expected_states)
    assert set(queues)<=set(expected_queues)
    for name,expected in expected_queues.items():assert queues[name]==expected,(name,queues[name],expected)
    stable();no_gui()
    receipt.update(status='original_clock_fragments_independently_verified',phase_vectors=8656,scroll_vectors=9668,
        queue_scenarios=15,original_functions=8,original_code_bytes=sum(m['bytes'] for m in prepared['functions'].values()),
        clock_source_stubbed=True,audio_dispatch_tail_stubbed=True,secondary_map_vector_supplied_empty=True,
        full_S28_maploader_or_client_executed=False,modern_clock_scene_binding_verified=False,owned_process=None)
    persist()
except BaseException as error:
    receipt.update(status='failed_preserved',error=repr(error),owned_process=None);persist();raise
finally:
    files={p.name:p for p in OUT.iterdir() if p.is_file() and p.name not in ['raw.zip','archive-readback.json']}
    if helper.exists() and receipt.get('build.log_exit_code')==0:files['native-x86-oracle.exe']=helper
    with zipfile.ZipFile(OUT/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for name,p in files.items():z.write(p,name)
    with zipfile.ZipFile(OUT/'raw.zip') as z:
        assert set(z.namelist())==set(files)
        for name,p in files.items():assert digest(z.read(name))==sha(p),name
    save(OUT/'archive-readback.json',{'status':'raw_archive_readback_verified','receipt_status':receipt['status'],'archive_sha256':sha(OUT/'raw.zip'),'files':{n:sha(p) for n,p in files.items()}})
    if 'native-x86-oracle.exe' in files:save(retention,{'exe_sha256':sha(helper),'archive':str(OUT/'raw.zip'),'archive_sha256':sha(OUT/'raw.zip')})
assert receipt['status']=='original_clock_fragments_independently_verified'
save(OUT/'qualified.json',{'status':receipt['status'],'phase_vectors':8656,'scroll_vectors':9668,'queue_scenarios':15,
     'original_functions':8,'archive_sha256':sha(OUT/'raw.zip'),'full_client_or_maploader_verified':False,
     'modern_binding_verified':False,'game_sessions':0,'whole_goal_complete':False})
save(APP/'water-clock-original-current.json',{'output':str(OUT),'qualification':str(OUT/'qualified.json'),'whole_goal_complete':False})
print(json.dumps({'output':str(OUT),'status':receipt['status'],'phase_vectors':8656,'scroll_vectors':9668,'queue_scenarios':15}))
