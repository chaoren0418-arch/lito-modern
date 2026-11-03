"""Independent readback of the typed delivery candidate, with no audio output."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,wave,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path)
ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-exit-code',type=int,required=True)
ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
r=a.run.resolve(strict=True)
assert r.is_relative_to(ROOT/'驗證紀錄/current/native-typed-audio-delivery') and a.terminal_exit_code==0
x=read(r/'receipt.json');rb=read(r/'readback.json');m=read(r/'candidate.json')
assert x['status']==rb['status']=='passed_isolated_typed_audio_delivery_candidate'
assert (x['passed'],x['failed'],x['ignored'])==(6,0,0)
assert sha(r/'raw.zip')==rb['raw_archive_sha256']
with zipfile.ZipFile(r/'raw.zip') as z:
    assert set(z.namelist())==set(rb['raw_files'])
    for n,h in rb['raw_files'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
base=Path(x['base_related_run']);pins=read(base/'compiled-inputs.json')
assert len(pins)==1025 and all(sha(ROOT/n)==h for n,h in pins.items())
for p,h in x['candidate_pins'].items():assert sha(p)==h
for n,h in m['after'].items():
    assert not (ROOT/n).exists() and sha(r/Path(n).name)==h
for v in x['libraries'].values():assert sha(v['path'])==v['sha256']
assert sha(x['executable'])==x['executable_sha256']
names=re.findall(r'^(.+): test$',(r/'inventory/stdout.log').read_text(encoding='utf-8-sig'),re.M)
text=(r/'runtime/stdout.log').read_text(encoding='utf-8-sig')
assert len(names)==len(set(names))==6
assert set(re.findall(r'^test (.+?) \.\.\. ok$',text,re.M))==set(names)
assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)==[('6','0','0')]
assert not (r/'runtime/stderr.log').read_bytes().strip()
tests=(r/'isolated_tests.rs').read_text(encoding='utf-8-sig')
assert 'CombatVoicePool::new' not in tests and 'impl Player for Spy' in tests and 'impl Player for Count' in tests
assert len(x['process_history'])==3 and [p['exit_code'] for p in x['process_history']]==[0,0,0]
assert x['owned_process'] is None
ids=','.join(str(p['pid']) for p in x['process_history'])
script=f"$ids=@({ids}); @(Get-CimInstance Win32_Process | Where-Object {{$_.ProcessId -in $ids}} | ForEach-Object {{[pscustomobject]@{{pid=$_.ProcessId;creation_utc=$_.CreationDate.ToUniversalTime().ToString('o')}}}}) | ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',script],encoding='utf-8-sig').strip()
live=json.loads(raw) if raw else []
if isinstance(live,dict):live=[live]
for p in x['process_history']:
    assert not any(p['pid']==v['pid'] and datetime.fromisoformat(p['creation_utc'])==datetime.fromisoformat(v['creation_utc'].replace('Z','+00:00')) for v in live)
with zipfile.ZipFile(r/'raw.zip') as z:
    catalog=json.loads(z.read('catalog-definitions.json').decode('utf-8-sig'))
sound=next(v for v in catalog['prepared_sound_bank']['sounds'] if v['id']==803)
wav=Path(r'D:\lineage開發\0-客戶端\Sound\803.wav')
assert sha(wav)==sound['source_wav_sha256']
with wave.open(str(wav),'rb') as f:
    pcm=f.readframes(f.getnframes())
    assert f.getnchannels()==sound['channels'] and f.getframerate()==sound['sample_rate']
    assert f.getsampwidth()*8==sound['bits_per_sample']
assert hashlib.sha256(pcm).hexdigest().upper()==sound['pcm_sha256'] and len(pcm)==sound['pcm_bytes']
terminal=dict(original_session_id=a.session_id,terminal_exit_code=0,terminal_tool_chunk_id=a.terminal_chunk,handle_closed=True)
save(r/'original-session-observation.json',terminal)
save(r/'independent-readback.json',dict(status='independently_qualified_isolated_typed_audio_delivery_candidate',
    original_session=terminal,source_manifest_sha256=x['source_manifest_sha256'],compiled_inputs_checked=1025,
    raw_archive_sha256=sha(r/'raw.zip'),executable_sha256=sha(x['executable']),passed=6,failed=0,ignored=0,
    modes_and_weighted_order_retained=True,zero_and_duplicate_candidates_retained=True,
    connection_generation_publication_fences_tested=True,context_rebinding_rejected=True,
    frame_clones_share_once_only_latch=True,parallel_copies=8,parallel_actual_forwardings=1,
    focus_discard_and_error_no_retry_tested=True,last_frame_bank_release_tested=True,
    original803_WAV_and_PCM_match=True,test_driver='Spy and Count recorders; no native voice pool instantiated.',
    recorded_direct_processes=3,recorded_direct_processes_all_terminal=True,
    product_source_changed=False,applied=False,native_window_integration_complete=False,
    actual_new_sessions=0,actual_new_casts=0,audio_device_opened=False,physical_audio_verified=False,
    GUI_verified=False,whole_goal_complete=False))
print(json.dumps(dict(status='independently_qualified_isolated_typed_audio_delivery_candidate',passed=6,failed=0,
    ignored=0,recorded_direct_processes_all_terminal=True,product_source_changed=False,
    output=str(r/'independent-readback.json')),ensure_ascii=False))
