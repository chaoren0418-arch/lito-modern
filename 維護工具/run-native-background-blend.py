"""Execute isolated original tables/pixels with an explicit pinned host CRT."""
from pathlib import Path
from datetime import datetime,timezone
import array,csv,hashlib,json,os,struct,subprocess,sys,zipfile

ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'floor-tests-20261016-102632-227125/normal-qualified'
OUT=Path(sys.argv[1]).resolve(strict=True);assert OUT.is_relative_to(ROOT/'驗證紀錄/current/native-background-blend-original')
GB=0x630000;GE=0x832000
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
now=lambda:datetime.now(timezone.utc).isoformat()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
prep=read(OUT/'preparation-readback.json')
assert prep['status']=='prepared_complete_background_initializer_and_pixel_kernels_not_executed'
assert sha(OUT/'preparation-raw.zip')==prep['raw_archive_sha256']
with zipfile.ZipFile(OUT/'preparation-raw.zip') as z:
    assert set(z.namelist())==set(prep['files'])
    for n,h in prep['files'].items():assert digest(z.read(n))==sha(OUT/n)==h,n
pins=read(NORMAL/'source-inputs.json');nq=read(NORMAL/'qualified.json');assert len(pins)==1005
def stable():
    assert sha(NORMAL/'source-inputs.json')==prep['source_manifest_sha256']
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    for p,v in nq['profiles'].items():assert sha(v['path'])==v['sha256']==prep['normal_mains'][p]
    assert sha(prep['host_x86_CRT'])==prep['host_CRT_sha256']
def idle():
    q=read(NORMAL/'GUI-completion-observation.json');end=read(NORMAL/'GUI-process-completion-observation.json')
    assert q['status']=='independently_readback_verified_current1005_TW14_GUI_and_terminal_process'
    assert end['actual_terminal_tool_observed'] and end['terminal_exit_code']==0 and end['owned_workloads']==[]
    command="$r='"+str(ROOT).replace("'","''")+"';$p=@(Get-CimInstance Win32_Process|Where-Object {($_.Name -in @('cargo.exe','rustc.exe')) -or ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase) -and ($_.Name -match '^(LiToModern|lito_modern-|native_|qualified-client|actual-client|oracle)' -or ($_.ExecutablePath.Split([IO.Path]::DirectorySeparatorChar) -contains 'deps')))});if($p.Count){exit 1}"
    p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    assert p.returncode==0,'Owned GUI/server/compiler/test/helper remains live'
stable();idle();assert not (OUT/'run-receipt.json').exists()
(OUT/'runner.py').write_bytes(Path(__file__).read_bytes())
receipt={'status':'building_private_background_blend_oracle','started_utc':now(),'owned_process':None,
         'source_files':1005,'original_function_bytes':4246,'native_execution_started':False,
         'host_CRT_sha256':prep['host_CRT_sha256'],'historical_CRT_DLL_identity_verified':False,
         'game_sessions':0,'whole_goal_complete':False,'process_history':[]}
persist=lambda:save(OUT/'run-receipt.json',receipt)
env=os.environ.copy();env.update(TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
def run(command,name):
    with (OUT/name).open('wb') as so,(OUT/(name+'.stderr')).open('wb') as se:
        child=subprocess.Popen(command,cwd=OUT,env=env,stdout=so,stderr=se,creationflags=subprocess.CREATE_NO_WINDOW)
        identity={'pid':child.pid,'arguments':command,'observed_started_utc':now()};receipt['owned_process']=identity;receipt['process_history'].append(identity);persist()
        try:code=child.wait(timeout=180)
        except subprocess.TimeoutExpired:
            child.kill();child.wait();receipt['owned_process']=None;receipt['owned_direct_handle_timeout']=identity;persist();raise
        identity.update(exit_code=code,finished_utc=now());receipt['owned_process']=None;receipt[name+'_exit_code']=code;persist()
    assert code==0,(name,code)
def screen(a,b):return a+b-(a*b+30)//31
def mul(a,b):return a*b//31
def channel_tables(mode):
    shifts=[0,6 if mode else 5,11 if mode else 10]
    addresses=[[0x81089c,0x81109c,0x81189c],[0x80d89c,0x80e09c,0x80e89c],
               [0x82109c,0x82609c,0x82b09c],[0x81209c,0x81709c,0x81c09c]]
    result=[]
    for kind,bases in enumerate(addresses):
        for base,shift in zip(bases,shifts):
            values=[]
            for strength in range(10) if kind>=2 else [32]:
                factor=15+strength*16//10
                for a in range(32):
                    for b in range(32):
                        v=screen(a,b) if kind%2==0 else mul(a,b)
                        if kind>=2:v=v*factor//31
                        values.append(v<<shift)
            result.append((base,values))
    return result
def base_rgb(x,y):return (x%32,y%32,(x+3*y)%32)
def overlay_rgb(x,y):return ((5*x+3*y)%32,(7*x+y)%32,(x+11*y)%32)
persist()
try:
    run(read(OUT/'build-command.json')['args'],'build.log');stable();idle()
    b=(OUT/'oracle.exe').read_bytes();e=int.from_bytes(b[60:64],'little')
    assert b[:2]==b'MZ' and b[e:e+4]==b'PE\0\0' and b[e+4:e+6]==b'\x4c\x01' and b[e+24:e+26]==b'\x0b\x01'
    receipt.update(status='executing_original_initializer_and_background_pixel_kernels',native_execution_started=True);persist()
    run([str(OUT/'oracle.exe'),str(OUT)],'native-summary.json')
    actual=read(OUT/'native-summary.json')
    assert actual['cases']==1584 and actual['host_ftol_calls']==2048 and actual['lock_calls']==1584 and actual['unlock_calls']==3168 and actual['guards_unchanged']
    loaded=Path((OUT/'host-CRT-final-path.txt').read_text(encoding='ascii').strip()).resolve(strict=True)
    assert loaded.samefile(prep['host_x86_CRT']) and sha(loaded)==prep['host_CRT_sha256']
    module=read(OUT/'host-CRT-module.json');assert module['machine']==0x14c and module['optional_header_magic']==0x10b and module['ftol_RVA']==0x9ac20 and module['captured_ftol_bytes']==48
    dll=loaded.read_bytes();eh=struct.unpack_from('<I',dll,60)[0];optional=eh+24
    for i in range(struct.unpack_from('<H',dll,eh+6)[0]):
        _,va,size,off=struct.unpack_from('<IIII',dll,optional+struct.unpack_from('<H',dll,eh+20)[0]+i*40+8)
        if va<=module['ftol_RVA'] and module['ftol_RVA']+48<=va+size:
            pinned_ftol=dll[off+module['ftol_RVA']-va:off+module['ftol_RVA']-va+48];break
    else:raise AssertionError('pinned conversion export section')
    assert (OUT/'host-ftol-loaded.bin').read_bytes()==pinned_ftol
    save(OUT/'host-CRT-comparison.json',{'physical_path':str(loaded),'file_sha256':sha(loaded),'module':module,'loaded_ftol48_sha256':digest(pinned_ftol),'loaded_export_matches_pinned_file':True,'historical_DLL_identity_verified':False})
    extraction=read(OUT/'extraction.json')
    with (OUT/'private-addresses.csv').open(encoding='ascii',newline='') as f:addresses={(r['function'],r['kind'],int(r['offset'])):int(r['address']) for r in csv.DictReader(f)}
    private_arena_bases=set();private_constant_bases=set()
    for name,v in extraction.items():
        original=(OUT/(name+'-original.bin')).read_bytes();relocated=(OUT/(name+'-relocated.bin')).read_bytes()
        assert digest(original)==v['sha256'] and len(original)==len(relocated)==v['bytes']
        allowed={i for kind,offset,key in v['relocations'] for i in range(offset,offset+4)}
        assert all(a==b or i in allowed for i,(a,b) in enumerate(zip(original,relocated)))
        for kind,offset,key in v['relocations']:
            value=struct.unpack_from('<I',relocated,offset)[0];target=addresses[(name,kind,offset)]
            if kind in ['D','I']:
                assert value==target
                (private_arena_bases if GB<=key<GE else private_constant_bases).add(target-(key-(GB if GB<=key<GE else 0x4fa328)))
            else:assert (addresses[(name,'B',0)]+offset+4+value)&0xffffffff==target
    assert len(private_arena_bases)==len(private_constant_bases)==1
    table_records=[];checked_words=0
    for mode in [0,1]:
        data=(OUT/f'initialized-mode{mode}.bin').read_bytes();assert len(data)==GE-GB
        for address,expected in channel_tables(mode):
            packed=struct.pack('<'+'H'*len(expected),*expected);native=data[address-GB:address-GB+len(packed)]
            assert native==packed,('channel table',mode,hex(address));checked_words+=len(expected)
            table_records.append({'format':mode,'address':address,'words':len(expected),'sha256':digest(native)})
    assert checked_words==135168
    save(OUT/'channel-table-comparison.json',{'checked_words':checked_words,'tables':table_records})
    with (OUT/'cases.csv').open(encoding='ascii',newline='') as f:cases=[{k:int(v) for k,v in row.items()} for row in csv.DictReader(f)]
    with (OUT/'results-native.csv').open(encoding='ascii',newline='') as f:results=[{k:int(v) for k,v in row.items()} for row in csv.DictReader(f)]
    assert len(cases)==len(results)==1584
    stream=(OUT/'pixels-native.bin').read_bytes();cursor=0;total=0;vectors=[]
    for c,result in zip(cases,results):
        case,count=struct.unpack_from('<II',stream,cursor);cursor+=8;assert case==c['case']==result['case']
        native=[]
        for i in range(count):index,value=struct.unpack_from('<IH',stream,cursor);cursor+=6;native.append((index,value))
        bottom=min(c['bottom'],c['top']+552-c['dest_y']);expected=[]
        oy=c['offset_y']+(16 if c['offset_x']//32%2 else 0)
        ox=32-c['offset_x']%32+c['left'];oy=32-oy%32+c['top']
        for y in range(bottom-c['top']):
            for x in range(c['right']-c['left']):
                dx=c['dest_x']+x;dy=c['dest_y']+y
                gate=0 if c['gate']==0 else 1 if c['gate']==1 else (7 if (dx+dy)%2 else 0)
                if gate:continue
                assert 0<=ox+x<64 and 0<=oy+y<64
                a=base_rgb(c['left']+x,c['top']+y);b=overlay_rgb(ox+x,oy+y)
                values=[(screen(u,v) if c['blend_mode']==0 else mul(u,v)) for u,v in zip(a,b)]
                if c['strength']!=32:values=[v*(15+c['strength']*16//10)//31 for v in values]
                color=(values[0]<<(11 if c['format'] else 10))|(values[1]<<(6 if c['format'] else 5))|values[2]
                if color!=0x1777:expected.append((dy*c['pitch']+dx,color))
        assert native==expected,('pixel case',case,native[:3],expected[:3])
        assert result=={'case':case,'bottom':bottom,'locks':1,'unlocks':2,'changes':len(expected)}
        frame=array.array('H',[0x1777])*(c['pitch']*552)
        for index,value in native:frame[index]=value
        assert sys.byteorder=='little';vectors.append({'case':case,'frame_sha256':digest(frame.tobytes()),'changed_words':count,'clipped_source_bottom':bottom});total+=count
    assert cursor==len(stream) and total==actual['changed_words']
    save(OUT/'full-frame-vectors.json',{'cases':1584,'vectors':vectors,'whole_goal_complete':False})
    stable();idle();receipt.update(status='original_background_tables_and_pixels_match_independent_scalar',checked_channel_words=135168,pixel_cases=1584,changed_words=total,source_inputs_unchanged=True,normal_mains_unchanged=True)
except BaseException as error:
    receipt.update(status='failed_preserved',error=repr(error));raise
finally:
    receipt['finished_utc']=now();persist()
    files={p.name:sha(p) for p in OUT.iterdir() if p.is_file() and p.name!='preparation-raw.zip'}
    with zipfile.ZipFile(OUT/'run-raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n in files:z.write(OUT/n,n)
    with zipfile.ZipFile(OUT/'run-raw.zip') as z:
        assert set(z.namelist())==set(files)
        for n,h in files.items():assert digest(z.read(n))==h,n
    save(OUT/'run-readback.json',{'status':receipt['status'],'files':files,'raw_archive_sha256':sha(OUT/'run-raw.zip'),'archive_readback_verified':True,'historical_CRT_DLL_identity_verified':False,'full_background_renderer_verified':False,'whole_goal_complete':False})
print(json.dumps({'path':str(OUT),'status':receipt['status'],'checked_channel_words':receipt.get('checked_channel_words'),'pixel_cases':receipt.get('pixel_cases')},ensure_ascii=False))
