"""Qualify independent decoder samples, never infer a native render bank or cast."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-attack-five-resource-inspection')
r=read(run/'report.json');assert r['status']=='executed_readonly_independent_CSharp_544_blank_samples'
assert r['reference_DLL_sha256']=='8F31AEA698513D8606432784DB6047EEDBC9431355A1E0797DC4C4006D03DE6F'
assert not r['independent_archive_reader_claimed'] and r['new_sessions']==r['new_casts']==0 and not r['product_sources_modified']
assert sha(r['inventory'])==r['inventory_sha256']
inventory={m['name']:m for m in read(r['inventory'])['members']}
for p,h in r['reference_libraries'].items():assert sha(p)==h
ep=Path(r['raw_export_receipt']);e=read(ep);assert sha(ep)==r['raw_export_receipt_sha256']
assert e['status']=='exported_three_exact_pinned_raw_samples_no_decoder' and not e['sprite_decode_executed']
assert e['owned_process'] is None and len(e['process_history'])==2 and all(p['exit_code']==0 for p in e['process_history'])
assert e['new_sessions']==e['new_casts']==0 and e['inventory_sha256']==r['inventory_sha256']
assert sha(Path(e['process_history'][1]['command'][0]))==e['product_sha256']
pins=read(ROOT/'驗證紀錄/current/native-default111/20261017-135059-659904/compiled-inputs.json') if (ROOT/'驗證紀錄/current/native-default111/20261017-135059-659904/compiled-inputs.json').exists() else read(ROOT/'驗證紀錄/current/native-selector-integration/validation-20261017-123504-346151/compiled-inputs.json')
assert len(pins)==1029 and all(sha(ROOT/n)==h for n,h in pins.items())
expected={'1815-0.spr':list(range(11,21)),'1816-0.spr':list(range(11,21)),'1818-0.spr':list(range(5))+list(range(14,21))}
assert {m['name'] for m in r['members']}==set(expected)=={m['name'] for m in e['members']}
files={'report.json':run/'report.json','inspector.ps1':run/'inspector.ps1','export/receipt.json':ep,'inventory-reference.json':Path(r['inventory'])}
assert sha(run/'inspector.ps1')==sha(ROOT/'維護工具/inspect-native-attack-five-csharp-blanks.ps1')
members={};nulls=0;nonempty=0
for m in r['members']:
    name=m['name'];old=inventory[name];exported=next(x for x in e['members'] if x['name']==name)
    assert m['raw_bytes']==old['raw_bytes']==exported['raw_bytes']
    assert m['raw_sha256']==old['raw_sha256']==exported['raw_sha256']==sha(m['raw_sample'])
    assert Path(m['raw_sample']).resolve()==Path(exported['path']).resolve()
    assert len(m['frames'])==old['declared_headers']['frame_count']==21
    assert m['reviewed_544_null_slots']==expected[name]
    for index,f in enumerate(m['frames']):
        h=old['declared_headers']['frames'][index];assert f['source_header']==h and f['index']==index
        left,top,right,bottom=h['bounds']
        if h['references']:
            raw=Path(m['raw_sample']).read_bytes();positions=[]
            for ref_index in range(h['references']):
                aa,bb,depth,block_id=struct.unpack_from('<bbBH',raw,h['header_offset']+14+5*ref_index)
                half=aa//2;positions.append((24*(bb+aa-half),12*(bb-half)))
            left=min(x for x,y in positions);top=min(y for x,y in positions)
            right=max(x+23 for x,y in positions);bottom=max(y+23 for x,y in positions)
        assert (f['XOffset'],f['YOffset'],f['Width'],f['Height'])==(left,top,right-left+1,bottom-top+1)
        assert (f['unknown1'],f['unknown2'])==tuple(h['unknown'])
        if index in expected[name]:
            assert h['bounds']==[544,384,-480,-384] and h['references']==0 and h['unknown']==[65535,0]
            assert f['pixels_null'] and f['pixel_bytes']==0 and f['pixel_sha256'] is None;nulls+=1
        else:
            assert not f['pixels_null'] and f['pixel_bytes']==f['Width']*f['Height']*4
            assert len(f['pixel_sha256'])==64;nonempty+=1
    members[name]=dict(raw_sha256=m['raw_sha256'],raw_bytes=m['raw_bytes'],frames=21,reviewed_null_slots=expected[name])
    files['samples/'+name]=Path(m['raw_sample'])
assert nulls==32 and nonempty==31 and sum(m['raw_bytes'] for m in members.values())==20879
for n in ['probe.rs','runner.py','rustc.stdout.txt','rustc.stderr.txt','export.stdout.txt','export.stderr.txt']:files['export/'+n]=ep.parent/n
failures=[]
for name,chunk in [('csharp544-20261017-162124-436','79ae9c'),('csharp544-20261017-162210-460','96e6d0')]:
    p=run.parent/name;failure=read(p/('observer-failure-'+chunk+'.json'))
    assert failure['observed_terminal_exit_code']==1 and failure['terminal_tool_chunk_id']==chunk
    assert sha(p/'report.json')==failure['original_report_sha256'];failures.append(str(p))
    for n in ['report.json','inspector.ps1','observer-failure-'+chunk+'.json']:files['retained-failures/'+name+'/'+n]=p/n
failure=run/'qualifier-failure-06a3f1';f=read(failure/'failure.json')
assert f['observed_terminal_exit_code']==1 and f['terminal_tool_chunk_id']=='06a3f1'
assert sha(failure/'qualifier.py')==f['original_qualifier_sha256'] and sha(run/'report.json')==f['report_sha256']
for n in ['failure.json','qualifier.py']:files['retained-failures/qualifier-06a3f1/'+n]=failure/n
failures.append(str(failure))
identities=e['process_history'];ids=','.join(str(p['pid']) for p in identities)
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for p in live:
    for old in identities:
        if p['ProcessId']==old['pid']:assert abs((datetime.fromisoformat(p['created'].replace('Z','+00:00'))-datetime.fromisoformat(old['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'CIM.json').write_bytes(raw);files['CIM.json']=out/'CIM.json';files['qualifier.py']=Path(__file__)
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(hashes)
    for n,h in hashes.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
q=dict(status='independently_qualified_CSharp_exact_544_no_draw_samples',source_manifest_sha256=e['source_manifest_sha256'],members=members,
    exact_marker=[544,384,-480,-384,65535,0],zero_block_references_required=True,reviewed_null_slots=32,nonempty_reference_frames=31,
    all_63_frame_offsets_and_bounds_independently_rederived=True,nonempty_bounds_from_24x24_block_union=True,
    raw_bytes=20879,reference_DLL_sha256=r['reference_DLL_sha256'],export_terminal_tool_chunk='771a8c',CSharp_terminal_tool_chunk='60263c',
    recorded_export_child_identities=2,recorded_export_children_all_terminal=True,retained_observer_failures=failures,
    native_decoder_changed=False,native_pixel_equivalence_verified=False,native_render_bank_verified=False,new_sessions=0,new_casts=0,
    GUI_verified=False,whole_goal_complete=False,raw_files=hashes,raw_archive_sha256=sha(out/'raw.zip'))
save(out/'qualified.json',q)
print(json.dumps(dict(qualification=str(out/'qualified.json'),reviewed_null_slots=32,nonempty_reference_frames=31,native_decoder_changed=False),ensure_ascii=False))
