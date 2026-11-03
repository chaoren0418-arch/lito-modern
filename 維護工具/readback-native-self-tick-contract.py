"""Independently read codec bytes as unsigned LE u32, with no gameplay claim."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-self-tick-contract')
receipt=read(run/'codec-receipt.json');ar=read(run/'codec-archive-readback.json')
assert receipt['status']=='executed_self_tick_codec_both_profiles_no_sessions'
assert len(receipt['calls'])==3 and all(v['exit_code']==0 for v in receipt['calls'])
assert receipt['actual_game_sessions']==receipt['actual_casts']==0 and not receipt['native_clock_source_verified']
assert sha(run/'codec-raw.zip')==ar['archive_sha256']
ticks=[0,2,0x12345678,0x7fffffff,0x80000000,0xffffffff]
with zipfile.ZipFile(run/'codec-raw.zip') as z:
    assert len(z.namelist())==len(ar['files']) and set(z.namelist())==set(ar['files'])
    for n,h in ar['files'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
        if not n.startswith('classes/'):assert sha(run/n)==h
    assert receipt['source_sha256']==ar['files']['NativeSelfTickContractOracle.java']
    for profile in ['lito-legacy','tw13081901-v1']:
        value=json.loads(z.read(profile+'.stdout.txt').decode('utf-8-sig'))
        assert value['profile']==profile and value['negative_rejections']==13
        assert value['actual_casts']==value['actual_game_sessions']==0 and not value['native_clock_source_verified']
        assert [row['tick'] for row in value['positive']]==ticks
        for row,tick in zip(value['positive'],ticks):
            raw=bytes.fromhex(row['hex'])
            assert len(raw)==7 and raw[:3]==bytes([0x14,7,3])
            assert int.from_bytes(raw[3:],'little',signed=False)==tick and row['skill_id']==60
jar=ROOT/'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar'
assert sha(jar)==receipt['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
q=dict(status='independently_read_back_C02_self_tick_unsigned_u32_codec_only',jar_sha256=receipt['jar_sha256'],
    profiles=['lito-legacy','tw13081901-v1'],positive_vectors=12,negative_rejections=26,skill_id=60,
    wire_prefix_hex='140703',wire_bytes=7,client_tick_encoding='unsigned_u32_little_endian',
    actual_game_sessions=0,actual_casts=0,dispatch_phase_or_reuse_or_MP_or_item_consume_verified=False,
    native_clock_source_verified=False,product_source_changed=False,GUI_verified=False,whole_goal_complete=False,
    raw_archive_sha256=ar['archive_sha256'],raw_files=ar['files'],observed_utc=datetime.now(timezone.utc).isoformat())
p=run/'codec-qualified.json';assert not p.exists()
p.write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'codec-independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:q[k] for k in ['status','positive_vectors','negative_rejections','native_clock_source_verified']},ensure_ascii=False))
