"""Independently qualify original interval captures and exact executable retention."""
import argparse
from datetime import datetime,timezone
import hashlib
import json
from pathlib import Path
import re
import zipfile

ROOT=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser();ap.add_argument('out',type=Path);a=ap.parse_args()
out=a.out.resolve(strict=True);assert out.is_relative_to(ROOT/'驗證紀錄/current/native-game-timers')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def stamp(s):return datetime.fromisoformat(s.replace('Z','+00:00')).timestamp()
q=read(out/'qualified.json');r=read(out/'receipt.json');assert q['status']==r['status']=='completed_original_intervals_formal4_sessions_and_backpressure_no_poll'
assert q['source_files']==976 and not q['source_must_remain_frozen'] and not q['intervals_modified']
assert q['optin_tests_passed']==2 and q['actual_formal_sessions']==4
assert all(c['exit_code']==0 for c in q['children'].values())
assert q['test_exe_sha256']==sha(out/'timer-tests.exe')
assert q['raw']['readback_verified'] and sha(out/'raw-evidence.zip')==q['raw']['archive_sha256']
with zipfile.ZipFile(out/'raw-evidence.zip') as z:
    assert set(z.namelist())==set(q['raw']['files'])
    for n,h in q['raw']['files'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
for name in ['formal-client','backpressure-client']:
    text=(out/(name+'.stdout')).read_text(encoding='utf-8-sig')
    assert re.search(r'test result: ok\. 1 passed; 0 failed; 0 ignored;',text)
classes=read(out/'helper-classes/observation.json')
assert len(classes['classes'])==2
for name,h in classes['classes'].items():assert sha(out/'helper-classes'/name)==h
sr=read(out/'formal-server/result.json');cr=read(out/'formal-client/client-result.json')
assert sr['actual_client_sessions']==4 and sr['LOGIN_sessions']==sr['GAME_sessions']==2
assert sr['memberships_empty'] and sr['authority_mutations']==0 and not sr['sql_access'] and not sr['state_persistent']
assert sr['jar_sha256'].upper()==q['reference_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert cr['quiescent'] and not cr['calendar_locally_predicted'] and len(cr['rounds'])==2
frames=[]
for p in (out/'formal-server/captures').rglob('frames.jsonl'):
    assert p.relative_to(out).as_posix() in q['raw']['files']
    frames.extend(json.loads(l) for l in p.read_text(encoding='utf-8-sig').splitlines())
frames.sort(key=lambda r:stamp(r['time']))
groups=[]
for frame in frames:
    if frame['direction']=='server_to_client' and frame['opcode']=='0x01':groups.append([])
    assert groups
    groups[-1].append(frame)
assert len(groups)==4,'Exactly LOGIN/GAME followed by LOGIN/GAME'
proof=[];time_rows=[]
for i,g in enumerate(groups):
    timer=[f for f in g if f['direction']=='client_to_server' and f['opcode'] in ['0x20','0x1D']]
    if i in [0,2]:
        assert not timer,'LOGIN received a GAME timer';continue
    round=(i-1)//2
    alive=[f for f in timer if f['opcode']=='0x20'];save=[f for f in timer if f['opcode']=='0x1D']
    assert len(alive)==(6 if round==0 else 2) and len(save)==(1 if round==0 else 0)
    assert all(bytes.fromhex(f['plain'])==bytes([int(f['opcode'],16)]) and f['wire_length']==3 for f in timer)
    selected=[f for f in g if f['direction']=='server_to_client' and f['opcode']=='0xC8' and bytes.fromhex(f['plain'])==bytes([0xc8,0])]
    assert len(selected)==1 and stamp(alive[0]['time'])>=stamp(selected[0]['time'])
    if round==0:assert stamp(selected[0]['time'])-stamp(g[0]['time'])>=61
    differences=[stamp(b['time'])-stamp(a['time']) for a,b in zip(alive,alive[1:])]
    assert all(59.9<=d<=62.0 for d in differences),differences
    if save:assert 299.9<=stamp(save[0]['time'])-stamp(alive[0]['time'])<=303
    replies=[f for f in g if f['direction']=='server_to_client' and f['opcode']=='0x21']
    assert len(replies)==len(alive)
    for request,response in zip(alive,replies):
        delay=stamp(response['time'])-stamp(request['time']);assert 0<=delay<2
        body=bytes.fromhex(response['plain']);assert len(body)==5 and response['wire_length']==7
        value=int.from_bytes(body[1:],'little');assert value%3==0
        elapsed_ms=int(stamp(response['time'])*1000)-1_735_689_600_000
        expected=(elapsed_ms*6//1000)//3*3
        assert abs(value-expected)<=3
        time_rows.append({'request_time':request['time'],'response_time':response['time'],'raw_game_seconds':value,'delay_seconds':delay})
    client_round=cr['rounds'][round];assert client_round['timers']['alive_sent']==len(alive) and client_round['timers']['save_sent']==len(save)
    proof.append({'round':round,'client_epoch':client_round['epoch'],'C_ALIVE':len(alive),'C_SAVE':len(save),'S_TIME':len(replies),'alive_intervals_seconds':differences,'save_delay_seconds':stamp(save[0]['time'])-stamp(alive[0]['time']) if save else None})
assert proof[1]['client_epoch']>proof[0]['client_epoch']
result={'status':'independently_qualified_original_GAME_timer_intervals_C02_clock_replies_and_backpressure_test',
    'profile':q['profile'],'source_files':976,'source_manifest_sha256':q['source_manifest_sha256'],'test_exe_sha256':q['test_exe_sha256'],
    'actual_formal_sessions':4,'LOGIN_sessions':2,'GAME_sessions':2,'C_ALIVE':8,'C_SAVE':1,'S_TIME':8,
    'rounds':proof,'reply_vectors':time_rows,'raw_archive_sha256':q['raw']['archive_sha256'],'archive_readback_verified':True,
    'original_interval_backpressure_no_UI_poll_test_passed':True,'local_game_calendar_advance_predicted':False,
    'original_client_calendar_UTC_or_local_resolved':False,'GUI_verified':False,'whole_goal_complete':False,'qualifier_sha256':sha(Path(__file__))}
with (out/'independent-qualified.json').open('x',encoding='utf-8') as f:json.dump(result,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps({k:v for k,v in result.items() if k not in ['rounds','reply_vectors']},ensure_ascii=False))
