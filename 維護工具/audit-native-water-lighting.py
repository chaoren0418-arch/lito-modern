"""Deferred private original water row/raster oracle; never runs during GUI."""
from pathlib import Path
from datetime import datetime,timezone
import csv,hashlib,json,os,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
STATIC=ROOT/'驗證紀錄/current/native-lighting-actor-motion-static'
CPP=ROOT/'維護工具/oracle/NativeWaterLightingOracle.cpp'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def digest(b):return hashlib.sha256(b).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def no_gui():
    g=read(APP/'GUI-current.json')
    assert g['status']=='qualified_current_lighting_TW14_GUI_cases' and not g['source_and_corresponding_tests_frozen'], 'Current GUI must actually finish first'
    command="$r='"+str(ROOT)+"';$p=@(Get-CimInstance Win32_Process|Where-Object {($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase) -and ($_.Name -like 'LiToModern*' -or $_.ExecutablePath -match '\\\\deps\\\\.*\\.exe$'))});if($p.Count){exit 1}"
    result=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    assert result.returncode==0,'Owned GUI/test/server load remains live'
no_gui()
normal_path=APP/'normal-current/qualified.json';normal=read(normal_path);pins=read(normal_path.parent/'source-inputs.json')
assert len(pins)==994 and normal['features']==[]
def stable():
    for name,h in pins.items():assert sha(ROOT/name)==h,name
    for p in normal['profiles'].values():assert sha(p['path'])==p['sha256']
stable()
pe=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe')
assert sha(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
asset=Path(r'D:\lineage開發\0-客戶端\shade.dat');assert sha(asset)=='CB4F1ACA4E5272C68830B106DB1508292807233C199CA062AB53E3D2E709B52D'
fragments=read(STATIC/'water-original-fragments/fragments.json')['functions']
static=read(STATIC/'water-original-static-tables.json');assert static['bytes']==1472
base_channels=ROOT/'驗證紀錄/current/native-lighting-original-tables/channel-tables.bin'
original_channels=base_channels.read_bytes();assert len(original_channels)==12288
channel_qualification=read(base_channels.parent/'qualified.json')
assert channel_qualification['channel_outputs']==6144 and channel_qualification['archive_readback_verified']
for mode in range(2):
    for ch,shift in enumerate([0,6 if mode else 5,11 if mode else 10]):
        values=struct.unpack_from('<1024H',original_channels,(mode*3+ch)*2048)
        assert values==tuple((c*l//31)<<shift for c in range(32) for l in range(32))
out=ROOT/'驗證紀錄/current/native-water-lighting'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True,exist_ok=False)
runtime=PROJECT/'target/current/native-water-lighting-oracle';runtime.mkdir(exist_ok=True)
for n,v in fragments.items():
    p=STATIC/'water-original-fragments'/(n+'-original.bin');assert sha(p)==v['sha256'];(out/p.name).write_bytes(p.read_bytes())
tables=STATIC/'water-original-static-tables.bin';assert sha(tables)==static['sha256'];(out/tables.name).write_bytes(tables.read_bytes())
for name,p in [('oracle.cpp',CPP),('auditor.py',Path(__file__)),('normal-qualified.json',normal_path),('normal-source-inputs.json',normal_path.parent/'source-inputs.json'),('prior-original-channel-qualification.json',base_channels.parent/'qualified.json')]:
    (out/name).write_bytes(p.read_bytes())
save(out/'original-fragment-metadata.json',fragments)
command=read(ROOT/'驗證紀錄/current/native-lighting-source/build-command.json')['args']
command=[a.replace('NativeLightingPrimitiveOracle.cpp',CPP.name).replace('native-lighting-source-oracle',runtime.name) for a in command]
save(out/'build-command.json',{'args':command,'cwd':str(runtime)})
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
receipt={'status':'initializing_original_water_oracle','source_inputs':994,'product_source_changes':0,'native_execution':False,'game_sessions':0,'whole_goal_complete':False}
def persist():save(out/'receipt.json',receipt)
def execute(command,name):
    with (out/name).open('wb') as stdout,(out/(name+'.stderr')).open('wb') as stderr:
        child=subprocess.Popen(command,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        receipt['owned_process']={'pid':child.pid,'arguments':command};persist()
        code=child.wait();receipt['owned_process']=None;receipt[name+'_exit_code']=code;persist()
    assert code==0,(name,code)
persist()
try:
    execute(command,'build.log');no_gui();stable()
    helper=runtime/'oracle.exe';data=helper.read_bytes();at=int.from_bytes(data[60:64],'little');assert data[:2]==b'MZ' and data[at+4:at+6]==b'\x4c\x01'
    receipt.update(status='executing_private_original_water_fragments',native_execution=True);persist()
    execute([str(helper),str(out),str(asset)],'oracle.json')
    native=read(out/'oracle.json');assert native['row_calls']==640 and native['native_raster_calls']==56
    assert native['viewport_pixels_checked']==56*640*408 and native['guard_pixels_checked']==56*(800*414-640*408)
    assert native['retained_vectors']==24192 and not native['full_original_initializer_executed']
    for name,v in fragments.items():
        original=(out/(name+'-original.bin')).read_bytes();relocated=(out/(name+'-relocated.bin')).read_bytes()
        allowed={i for x in v['relocations'] for i in range(x['offset'],x['offset']+4)}
        assert len(original)==len(relocated) and all(a==b or i in allowed for i,(a,b) in enumerate(zip(original,relocated)))
        for address in {x['original_address'] for x in v['relocations']}:
            assert len({struct.unpack_from('<I',relocated,x['offset'])[0] for x in v['relocations'] if x['original_address']==address})==1
    raw=asset.read_bytes().replace(b'\r\n',b'\n');assert len(raw)==268800 and 26 not in raw
    shade=bytes(min(v,9) for v in raw);assert (out/'shade-private-CRT-clamped.bin').read_bytes()==shade
    source=tables.read_bytes();inputs=[struct.unpack_from('<32H',source,i*64) for i in range(3)]
    responses=[struct.unpack_from('<320H',source,192+i*640) for i in range(2)]
    native_tables=(out/'water-native-channel-tables.bin').read_bytes();assert len(native_tables)==122880
    expected=bytearray()
    for mode in range(2):
        for ch,shift in enumerate([0,6 if mode else 5,11 if mode else 10]):
            response=responses[int(ch==1)]
            for s in range(10):
                for color in range(32):
                    for level in range(32):expected.extend(struct.pack('<H',(response[s*32+inputs[ch][color]]*level//31)<<shift))
    assert native_tables==expected
    rows=list(csv.DictReader((out/'water-vectors.csv').read_text().splitlines()));assert len(rows)==24192
    for row in rows:
        mode,phase,sx,sy,x,y,raw,color,s,level,actual=[int(row[k]) for k in ['mode','phase','water_x','water_y','x','y','raw','color','shade','mapped','actual']]
        wx=(1280-sx-abs(phase)+2*(x//2))%640;wy0=(816-sy)%408;wy=wy0 if y==0 else (wy0+y-1)%408+1
        assert s==shade[(wx//2)*420+wy] and level==((raw//8)*13+7)%32
        values=[color&31,(color>>(6 if mode else 5))&31,(color>>(11 if mode else 10))&31]
        result=0
        for ch,c in enumerate(values):result|=struct.unpack_from('<H',native_tables,((mode*3+ch)*10240+s*1024+c*32+level)*2)[0]
        assert result==actual,row
    stable();no_gui();assert sha(asset)=='CB4F1ACA4E5272C68830B106DB1508292807233C199CA062AB53E3D2E709B52D'
    receipt.update(status='original_water_fragments_and_private_CRT_resource_read_verified',**{k:v for k,v in native.items() if k!='status'},
        Python_channel_entries_rechecked=61440,Python_retained_vectors_rechecked=24192,original_initializer_executed=False,
        source_normal_tables_supplied_from_prior_qualified_floor_formula=True,full_world_framebuffer_verified=False,
        modern_phase_camera_binding_verified=False,original_shade_loader_function_executed=False,owned_process=None)
    persist()
    files={p.name:p for p in out.iterdir() if p.is_file()};files['native-x86-oracle.exe']=helper
    archive=out/'water-original-evidence.zip'
    with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    file_pins={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(archive) as z:
        assert set(z.namelist())==set(file_pins)
        for n,h in file_pins.items():assert digest(z.read(n))==h
    save(out/'qualified.json',{**receipt,'file_pins':file_pins,'archive_sha256':sha(archive),'archive_readback_verified':True})
    print(json.dumps({'output':str(out),'status':receipt['status'],'native_raster_calls':56,'retained_vectors':24192}))
except BaseException as error:
    receipt.update(status='failed_preserved',error=repr(error));persist();raise
