"""Independently qualify selected candidate bytes/frames/PCM, without gameplay."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(path):return json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def digest(value):return bytes(value).hex().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
parser=argparse.ArgumentParser();parser.add_argument('run',type=Path);parser.add_argument('--terminal-chunk',required=True);parser.add_argument('--terminal-exit-code',required=True,type=int);args=parser.parse_args()
run=args.run.resolve(strict=True);assert run.is_relative_to(V/'native-sunburst-assets')and args.terminal_exit_code==0 and args.terminal_chunk not in ['','pending','unknown']
r=read(run/'receipt.json');report=read(run/'report.json')
assert r['status']=='candidate_selected_sunburst_assets_loaded_only'and r['compiler']['exit_code']==r['asset_probe']['exit_code']==0
assert r['resources_loaded']and r['resources_metadata_unchanged']and not r['metadata_only']and r['linked_executable']
assert all(r[key]['status']=='compiling_default_targets'for key in ['broad_compilation_before','broad_after_link','broad_compilation_after'])
assert r['broad_compilation_before']['profile_run']==r['broad_after_link']['profile_run']==r['broad_compilation_after']['profile_run']
for name,digest_value in read(run/'compiled-inputs.json').items():assert sha(ROOT/name)==digest_value,name
for name,digest_value in r['candidate_inputs'].items():assert sha(run/name)==sha(ROOT/'維護工具/candidates/native-sunburst-assets'/name)==digest_value
for row in r['libraries'].values():assert sha(row['path'])==row['sha256']
nq=read(r['normal_qualification']);assert sha(r['normal_qualification'])==r['normal_qualification_sha256']and nq['source_manifest_sha256']==r['source_manifest_sha256']and nq['passed']==90
assert sha(nq['normal']['path'])==nq['normal']['sha256']==r['normal_product_sha256']
assert sha(run/'report.json')==r['asset_report_sha256']and sha(run/'candidate.exe')==r['product_sha256']
with(run/'candidate.exe').open('rb')as f:
    assert f.read(2)==b'MZ';f.seek(0x3c);offset=struct.unpack('<I',f.read(4))[0];f.seek(offset);assert f.read(4)==b'PE\0\0'and struct.unpack('<H',f.read(2))[0]==0x8664
rb=read(run/'readback.json');assert sha(run/'evidence.zip')==rb['archive_sha256']
with zipfile.ZipFile(run/'evidence.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==value for name,value in rb['files'].items())
assert read(run/'resource-metadata-before.json')==read(run/'resource-metadata-after.json')
for name,value in read(run/'resource-metadata-after.json').items():assert(Path(name).stat().st_size,Path(name).stat().st_mtime_ns)==(value['bytes'],value['mtime_ns'])
tw=V/'native-eruption-current-tw-binding/20261018-072559-347946/binding.json';t=read(tw)
assert digest(report['pack_sha256'])==t['pack']['sha256']and digest(report['text_sha256'])==t['decoded_sha256']and report['decoded_bytes']==t['decoded_bytes']==5974928
expected={row['graphics_id']:row for row in t['definitions']if row['graphics_id']in[1811]}
assert{row['id']for row in report['definitions']}==set(expected)
for row in report['definitions']:
    source=expected[row['id']];assert(row['offset'],row['bytes'],row['line'],digest(row['sha256']))==(source['byte_offset'],source['byte_length'],source['line'],source['sha256'])
inventory=V/'native-attack-five-resource-inspection/20261017-144953-005315/asset-inventory.json';iq=read(inventory.parent/'qualified/qualified.json');assert sha(inventory)==iq['raw_files'][inventory.name]
old={row['name']:row for row in read(inventory)['members']}
names={'1811-0.spr'}
assert{row['name']for row in report['sprites']}==names and report['sprite_members']==1
encoded=retained=flattened=frames=0
for row in report['sprites']:
    prior=old[row['name']];assert row['source_bytes']==prior['raw_bytes']and digest(row['source_sha256'])==prior['raw_sha256']
    assert len(row['frames'])==len(prior['frames'])==19
    encoded+=row['source_bytes'];pixels=0
    for actual,reference in zip(row['frames'],prior['frames']):
        assert actual['index']==reference['index']and actual['offset']==reference['offset']and actual['size']==reference['size']and actual['frame_type']==reference['frame_type']
        assert actual['RGBA_sha256']==reference['rgba_sha256']and actual['RGBA_bytes']==actual['size'][0]*actual['size'][1]*4>0
        frames+=1;pixels+=actual['RGBA_bytes']
    assert row['retained_RGBA_bytes']>=pixels
    flattened+=pixels;retained+=row['retained_RGBA_bytes']
assert frames==report['total_sprite_frames']==19 and encoded==report['encoded_sprite_bytes']and flattened<=retained==report['retained_sprite_RGBA_bytes']<=32*1024*1024
audio=V/'native-eruption-audio-namespace-binding/20261018-073033-438070/binding.json';a=read(audio);waves={row['id']:row for row in a['rows']if row['id']in[709]}
assert{row['id']for row in report['sounds']}==set(waves)
for sound in report['sounds']:
    reference=waves[sound['id']];assert digest(sound['WAV_sha256'])==reference['wav_sha256']and sound['WAV_bytes']==reference['wav_bytes']
    assert sound['PCM_sha256']==reference['PCM_sha256']and sound['PCM_bytes']==reference['PCM_bytes']
assert report['WAV_bytes']==sum(row['wav_bytes']for row in waves.values())and report['PCM_bytes']==sum(row['PCM_bytes']for row in waves.values())
assert report['new_sessions']==report['new_casts']==report['new_renders']==0 and not report['audio_delivery_executed']and not report['physical_audio_verified']and not report['normal_product_registered']and not report['renderer_qualified']
jobs=[r['runner'],r['compiler'],r['asset_probe']];ids=','.join(str(job['pid'])for job in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
output=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(output.decode('utf-8-sig'))if output.strip()else[];live=[live]if isinstance(live,dict)else live
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'CIM.json').write_bytes(output)
record=dict(status='independently_qualified_candidate_sunburst_selected_asset_bank_only',source_manifest_sha256=r['source_manifest_sha256'],run=str(run),original_exit_code=0,terminal_tool_chunk=args.terminal_chunk,original_tool_completed_without_session_id=True,product_sha256=r['product_sha256'],report_sha256=sha(run/'report.json'),evidence_zip_sha256=sha(run/'evidence.zip'),current_TW_typed_parser_executed=True,SPR_members=1,SPR_frames=19,encoded_sprite_bytes=encoded,flattened_frame_RGBA_bytes=flattened,retained_pixel_storage_including_compact_atlases=retained,compact_atlas_capacity_bytes=retained-flattened,retention_metric_source=dict(path=str(ROOT/'LiTo381-Modern/src/modern_lito/sprite.rs'),sha256=sha(ROOT/'LiTo381-Modern/src/modern_lito/sprite.rs'),method='DecodedSprite::retained_rgba_bytes counts flat frame RGBA plus each shared compact RGB555 atlas capacity once.'),WAV_ids=[709],WAV_bytes=report['WAV_bytes'],PCM_bytes=report['PCM_bytes'],retained_reference_inventory=dict(path=str(inventory),sha256=sha(inventory),scope='SPR byte/decoded-frame references only; no old gameplay promotion'),recorded_owned_identities=3,recorded_owned_all_terminal=True,broad_compilation_interval_confirmed=True,normal_product_registered=False,source_overrides_used=False,candidate_module_added=True,audio_delivery_executed=False,new_sessions=0,new_casts=0,new_renders=0,renderer_qualified=False,physical_audio_verified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'qualified.json',record);print(json.dumps(record,ensure_ascii=False));print('QUALIFICATION='+str(out/'qualified.json'))
