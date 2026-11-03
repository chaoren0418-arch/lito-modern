"""Read back retained real C02 sessions and match environment snapshots to wire."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, bisect, hashlib, json, struct, zipfile

ROOT=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--debug',type=Path,required=True);ap.add_argument('--release',type=Path,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-environment')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
scope=read(run/'independent-qualified.json');normal=read(run/'normal/qualified.json')
assert not (run/'formal-independent-qualified.json').exists()
for name,h in {**read(run/'source-inputs.json'),**read(run/'fixture-inputs.json')}.items():assert sha(ROOT/name)==h
for v in normal['profiles'].values():assert sha(Path(v['path']))==v['sha256']
jar=ROOT/'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar'
source=Path(r'D:/lineage開發/3-lito開發/lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar')
assert sha(source)==sha(jar)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
policy=read(ROOT/'驗證紀錄/current/native-reference-calendar/reference-policy.json')
transitions=policy['offset_transitions_raw_and_after_seconds'];starts=[t[0] for t in transitions]
def calendar(v):
    raw=v['raw'];i=bisect.bisect_right(starts,raw)-1;offset=transitions[i][1] if i>=0 else policy['initial_offset_seconds']
    dt=datetime.fromtimestamp(raw+offset,timezone.utc)
    assert [v[k] for k in ['offset_seconds','year','day_of_year','month','day','hour','minute','second']]==[
        offset,dt.year,dt.timetuple().tm_yday-1,dt.month,dt.day,dt.hour,dt.minute,dt.second]
profiles={};totals={'LOGIN':0,'GAME':0,'C20':0,'C1D':0,'S21':0,'S0C':0}
for profile,out in [('Debug',a.debug),('Release',a.release)]:
    out=out.resolve(strict=True);assert out.is_relative_to(run/'formal')
    q=read(out/'qualified.json');receipt=read(out/'receipt.json');ret=q['raw']
    assert q['profile']==profile and q['source_manifest_sha256']==scope['source_manifest_sha256']
    assert q['status']==receipt['status']=='actual_original_C02_GAME_environment_world_time_and_retirement_verified'
    assert q['actual_formal_sessions']==4 and q['all_owned_children_terminal']
    assert all(v['status']=='exited' and v['exit_code']==0 for v in q['owned_processes'].values())
    assert ret['readback_verified'] and sha(Path(ret['path']))==ret['sha256']
    with zipfile.ZipFile(ret['path']) as z:
        assert set(z.namelist())==set(ret['files'])
        for name,h in ret['files'].items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,name
    assert sha(out/'actual-client.exe')==q['actual_probe_sha256']
    b=(out/'actual-client.exe').read_bytes();at=struct.unpack_from('<I',b,60)[0]
    assert struct.unpack_from('<H',b,at+4)[0]==0x8664 and struct.unpack_from('<H',b,at+24)[0]==0x20b
    assert sha(out/'client.rs')==sha(ROOT/'維護工具/oracle/native_environment_probe.rs')
    for v in read(out/'linked-libraries.json').values():assert sha(Path(v['path']))==v['sha256']
    config={line.split('=',1)[0]:line.split('=',1)[1] for line in (out/'formal-server/server.properties').read_text('utf-8').splitlines() if '=' in line}
    assert config['database.mode']=='memory' and config['database.read.only']=='true' and config['state.persistence.enabled']=='false'
    assert config['lito.rental.town.directory.enabled']=='true' and config['lito.rental.town.map']=='500'
    server=read(out/'formal-server/result.json');client=read(out/'client/client.json')
    assert server['actual_client_sessions']==4 and server['LOGIN_sessions']==server['GAME_sessions']==2
    assert server['memberships_empty'] and not server['authority_mutations'] and not server['sql_access'] and not server['state_persistent']
    assert client['quiescent'] and not client['synthetic_packet_injection'] and not client['calendar_locally_advanced']
    assert len(client['rounds'])==2 and client['rounds'][1]['epoch']>client['rounds'][0]['epoch']
    groups=[]
    for p in (out/'formal-server/captures').rglob('frames.jsonl'):
        for line in p.read_text('utf-8-sig').splitlines():
            f=json.loads(line)
            if f['direction']=='server_to_client' and f.get('note')=='S_KEY':groups.append([])
            assert groups;groups[-1].append(f)
    assert len(groups)==4;counts={k:0 for k in totals};proof=[]
    for index,g in enumerate(groups):
        requests=[f for f in g if f['direction']=='client_to_server' and f['opcode'] in ['0x20','0x1D']]
        if index in [0,2]:counts['LOGIN']+=1;assert not requests;continue
        counts['GAME']+=1;v=client['rounds'][(index-1)//2];times=[]
        assert v['before_admission']['epoch']==v['epoch'] and v['before_admission']['calendar'] is None
        for field in ['after_admission','mapped']:calendar(v[field]['calendar']);assert v[field]['epoch']==v['epoch']
        for f in requests:
            op=int(f['opcode'],16);assert bytes.fromhex(f['plain'])==bytes([op]) and f['wire_length']==3
            counts['C20' if op==32 else 'C1D']+=1
        for f in g:
            if f['direction']!='server_to_client' or f['opcode'] not in ['0x0C','0x21']:continue
            b=bytes.fromhex(f['plain'])
            if f['opcode']=='0x0C':assert len(b)==37 and f['wire_length']==39;raw=int.from_bytes(b[25:29],'little');assert raw==0;counts['S0C']+=1
            else:
                assert len(b)==5 and f['wire_length']==7;raw=int.from_bytes(b[1:],'little');counts['S21']+=1
                ms=int(datetime.fromisoformat(f['time'].replace('Z','+00:00')).timestamp()*1000)
                expected=(ms-1735689600000)*6//1000;expected-=expected%3;assert raw%3==0 and abs(raw-expected)<=3
            times.append((f['opcode'],raw))
        assert times[0]==('0x21',v['after_admission']['calendar']['raw'])
        assert times[-1]==('0x0C',v['mapped']['calendar']['raw'])==('0x0C',0)
        assert v['mapped']['time_packets']==sum(op=='0x21' for op,_ in times)
        assert v['mapped']['status_time_packets']==sum(op=='0x0C' for op,_ in times)
        assert (v['mapped']['map'],v['mapped']['map_mask_argument'])==(500,15)
        assert v['after_cancel']['epoch'] is None and v['after_cancel']['map'] is None and v['after_cancel']['calendar'] is None
        proof.append({'epoch':v['epoch'],'last_time_authority':times[-1],'time_updates':len(times)})
    assert counts['LOGIN']==counts['GAME']==2 and counts['C20']>=2 and counts['S21']>=4 and counts['S0C']>=2
    for key in totals:totals[key]+=counts[key]
    profiles[profile]={'run':str(out),'qualification_sha256':sha(out/'qualified.json'),
        'raw_archive_sha256':ret['sha256'],'actual_probe_sha256':q['actual_probe_sha256'],'counts':counts,'rounds':proof}
result={'status':'independently_qualified_both_profiles_actual_C02_environment_scope',
    'source_files':978,'source_manifest_sha256':scope['source_manifest_sha256'],'normal_mains':scope['normal_mains'],
    'profiles':profiles,'counts':totals,'actual_formal_sessions':8,'all_raw_archives_readback_verified':True,
    'native_GUI_mains_executed_by_this_probe':False,'live_blind_weather_scenarios_verified':False,
    'same_transport_map_portal_live_scenarios_verified':False,'full_current_TCP_verified':False,
    'night_framebuffer_integrated':False,'whole_goal_complete':False,'qualifier_sha256':sha(Path(__file__))}
(run/'formal-independent-qualified.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:result[k] for k in ['status','counts','actual_formal_sessions']},ensure_ascii=False))
