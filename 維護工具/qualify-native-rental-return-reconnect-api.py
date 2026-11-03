"""Independently qualify public-API rejection and formal return, never GUI."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, re, subprocess, zipfile
R = Path(__file__).resolve().parents[1]
P = R / 'LiTo381-Modern'
V = R / '驗證紀錄/current'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb') as stream: return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path,value): Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def pe64(path):
    with Path(path).open('rb') as stream:
        assert stream.read(2)==b'MZ';stream.seek(60);offset=int.from_bytes(stream.read(4),'little');stream.seek(offset)
        header=stream.read(26);assert header[:6]==b'PE\0\0\x64\x86' and header[24:26]==b'\x0b\x02'
def inventory(body):
    assert body[0]==0x7c;at=2;items=[]
    def take(size):
        nonlocal at
        assert 0<=size<=len(body)-at
        value=body[at:at+size];at+=size;return value
    def number(size): return int.from_bytes(take(size),'little')
    for _ in range(body[1]):
        item=number(4);assert item>0
        use_type,charge,icon,blessing,count,identified=[number(size)for size in [1,1,2,1,4,1]]
        end=body.find(b'\0',at,at+256);assert end>=at
        name=take(end-at).decode('cp950');assert take(1)==b'\0'
        detail=list(take(number(1)));assert count>0
        items.append(dict(id=item,use_type=use_type,charge=charge,icon=icon,blessing=blessing,
            count=count,identified=identified,name=name,detail=detail))
    assert at==len(body);return items
parser=argparse.ArgumentParser();parser.add_argument('run',type=Path)
parser.add_argument('--session',type=int,required=True);parser.add_argument('--terminal-chunk',required=True)
parser.add_argument('--terminal-exit-code',type=int,required=True);a=parser.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-rental-return-reconnect-api')
assert a.session>0 and a.terminal_exit_code==0 and a.terminal_chunk not in ['','pending','unknown']
c=read(run/'cohort.json');source=c['source_manifest_sha256']
assert source=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert c['status']=='executed_both_current_rental_entry_guard_profiles_pending_independent_readback'
assert c['actual_sessions']==16 and c['owned_processes']==[]
assert len(c['process_history'])==7 and all(row['exit_code']==0 and not row.get('cleanup_stopped_original_owned_handle') for row in c['process_history'])
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==source
assert w['active_workload_sessions']in [{},{'rental_return_reconnect_api':a.session}]
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1049 and len(read(run/'source-inputs.json'))==1048
assert sha(run/'source-inputs.json')==source
for name,digest in inputs.items():assert sha(R/name)==digest,name
for name,digest in c['validation_pins'].items():assert sha(R/'維護工具'/name)==digest,name
assert sha(run/'runner.py')==c['validation_pins']['run-native-rental-return-reconnect-api.py']
for name in ['normal','focused']:
    assert sha(c[name+'_qualification'])==c[name+'_qualification_sha256']
    assert read(c[name+'_qualification'])['source_manifest_sha256']==source
assert read(c['normal_qualification'])['passed_per_profile']==19
assert read(c['focused_qualification'])['profiles']['Debug']['passed']==10
assert not c['full_library_in_this_scope']
assert sha(c['jar'])==c['jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(c['catalog'])==c['catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
assert sha(c['oracle'])==c['oracle_sha256'] and sha(run/'NativePeerHarness.java')==c['derived_oracle_sha256']
derivation=c['fixture_derivation'];assert derivation['counts_only']and not derivation['gameplay_clocks_changed']and not derivation['authority_or_constructor_changes']
original=Path(c['oracle']).read_text('utf-8-sig');assert original.count(derivation['before'])==1
assert (run/'NativePeerHarness.java').read_text('utf-8')==original.replace(derivation['before'],derivation['after'])
work=Path(c['runtime']).resolve(strict=True);assert work.is_relative_to(P/'target/current/native-rental-return-reconnect-api')
classes=work/'classes';assert c['class_files']
assert {path.relative_to(classes).as_posix()for path in classes.rglob('*.class')}=={Path(name).as_posix()for name in c['class_files']}
for name,digest in c['class_files'].items():assert sha(classes/name)==digest
utc=lambda text:datetime.fromisoformat(text.replace('Z','+00:00'))
compilers=[row for row in c['process_history']if row['label']in ['cargo','javac']]
servers=[row for row in c['process_history']if row['label']=='server']
assert len(compilers)==3 and len(servers)==2
assert max(utc(row['finished_utc'])for row in compilers)<min(utc(row['creation_utc'])for row in servers)
profiles={};raw={name:run/name for name in ['runner.py','cohort.json','source-inputs.json','compiled-inputs.json','NativePeerHarness.java']}
raw['qualifier.py']=Path(__file__)
for profile in ['Debug','Release']:
    info=c['profiles'][profile];d=run/profile;runtime=Path(info['runtime']).resolve(strict=True)
    assert runtime==work/profile and info['passed']==1 and info['failed']==info['ignored']==0 and info['actual_sessions']==8
    assert sha(info['product'])==sha(info['original_test_product'])==info['product_sha256'];pe64(info['product'])
    assert sha(d/'test-product.zip')==info['test_product_archive_sha256']
    with zipfile.ZipFile(d/'test-product.zip')as archive:
        assert archive.namelist()==['native_rental_entry_guard.exe']
        with archive.open('native_rental_entry_guard.exe')as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==info['product_sha256']
    job=next(row for row in c['process_history']if row['label']=='cargo'and row['output']==str(d))
    expected=['test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--test','native_rental_entry_guard']+(['--release']if profile=='Release'else[])
    assert job['command'][1:]==expected
    messages=[json.loads(line)for line in(d/'cargo.stdout').read_text('utf-8-sig').splitlines()if line.startswith('{')]
    artifacts=[row for row in messages if row.get('reason')=='compiler-artifact'and row.get('executable')and row['profile']['test']]
    assert len(artifacts)==1 and artifacts[0]['target']['name']=='native_rental_entry_guard'and artifacts[0]['features']==[]
    assert Path(artifacts[0]['executable']).resolve()==Path(info['original_test_product']).resolve()
    log=(d/'client.stdout').read_text('utf-8-sig')
    assert re.findall(r'^test (\S+) \.\.\. ok$',log,re.M)==['rental_entry_rejects_ordinary_channel_before_wire_and_remains_usable_after_return']
    assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)==[('1','0','0','0','0')]
    result=read(runtime/'result.json');assert sha(runtime/'result.json')==info['result_sha256']
    assert result['actual_client_sessions']==8 and result['game_sessions']==4 and result['memberships_empty']
    assert result['authority_mutations']==0 and not result['sql_access']and not result['state_persistent']
    assert result['classic_world_members_after_disconnect']==result['active_hotel_leases_after_disconnect']==0
    capture=list((runtime/'captures').glob('*/frames.jsonl'));assert len(capture)==1
    frames=[json.loads(line)for line in capture[0].read_text('utf-8-sig').splitlines()]
    assert sum(row['plain']=='MASKED_C_LOGIN'for row in frames)==8
    packets=[(row,bytes.fromhex(row['plain']))for row in frames if row['plain']!='MASKED_C_LOGIN']
    sent=[body for row,body in packets if row['direction']=='client_to_server']
    op=lambda code:[body for body in sent if body[0]==code]
    assert len(op(0xc8))==8 and all(body[1:].rstrip(b'\0').decode('cp950')=='測試乙'for body in op(0xc8))
    assert op(0xca)==[bytes([0xca,1]),bytes([0xca,1])], 'rejected guard leaked an ordinary channel request'
    assert op(0x10)==[bytes([0x10]),bytes([0x10])]
    assert len(op(0xc9))==4 and not op(0x0a)
    assert sum(row['direction']=='server_to_client'and body==bytes([0x65])for row,body in packets)==2
    assert sum(row['direction']=='server_to_client'and body==bytes([0x28,0xf4,1,0])for row,body in packets)==2
    versions=[(i,row)for i,row in enumerate(frames)if row['direction']=='client_to_server'and row['plain']!='MASKED_C_LOGIN'and bytes.fromhex(row['plain'])[0]==0x00]
    assert len(versions)==8
    restart_acks=[(i,row)for i,row in enumerate(frames)if row['direction']=='server_to_client'and row['plain']!='MASKED_C_LOGIN'and bytes.fromhex(row['plain'])==b'\x65'];assert len(restart_acks)==2
    returned_versions=[(i,row)for i,row in versions if i>max(i for i,_ in restart_acks)];assert len(returned_versions)==4
    fallback_seconds=(utc(returned_versions[0][1]['time'])-min(utc(row['time'])for _,row in restart_acks)).total_seconds()
    assert fallback_seconds>=8,'reconnect must not shorten the original restart deadline'
    wire_items=[item for row,body in packets if row['direction']=='server_to_client'and body[0]==0x7c for item in inventory(body)]
    assert wire_items
    trace=[json.loads(line)['state']for line in(runtime/'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
    roles={}
    for role,account,count in [('a','PeerFixture',45),('b','PeerFixture2',43)]:
        record=read(runtime/(role+'-rental-entry-guard.json'))
        assert record['role']==role and record['account']==account
        for key in ['rejected_before_entry','rejected_after_return']:
            rejected=record[key];snapshot=rejected['snapshot']
            assert '此入口提供租裝服務'in rejected['error'] and snapshot['phase']=='character_selected'
            assert snapshot['selected_source']=='測試乙'and snapshot['selected_character']==account+'(測試乙)'
            assert snapshot['services']['rooms'] and snapshot['inventory']['owner']['connection']==rejected['epoch']
        entered=record['entered'];returned=record['returned']
        assert entered['phase']=='battle_pending'and entered['world']['map_id']==500 and len(entered['inventory']['items'])==count
        assert entered['selected_character']==account+'(測試乙)'and entered['selected_source']=='測試乙'
        for key,value in entered['inventory']['items'].items():
            assert int(key)==value['id'] and value in wire_items, 'complete S7C item record mismatch'
        assert record['rejected_after_return']['epoch']==record['rejected_before_entry']['epoch']+2
        assert returned['inventory']['owner']is None and not returned['inventory']['items']
        assert returned['phase']=='characters_ready'and returned['selected_character']is None
        assert returned['rental_set']is None and returned['joined_room']is None and returned['world']['map_id']is None
        assert not returned['world']['actors']and returned['rental_restart_acks']==0 and returned['selected_source']=='測試乙'
        assert sum(body[1:].rstrip(b'\0').decode('cp950')==account+'(測試乙)'for body in op(0xc9))==2
        matching=[row for row in trace if row.get('test_account')==account and row.get('source')=='測試乙']
        assert matching, 'no passive account authority observations'
        assert len({row['session_identity']for row in matching})==2, 'fresh GAME session not observed'
        assert any(row['phase']=='BATTLE'and row.get('rental_town')for row in matching)
        assert any(row['phase']=='LOBBY'and not row.get('rental_town')for row in matching)
        roles[role]=dict(account=account,inventory_count=count,full_wire_item_fields_matched=True,
            rejected_calls=2,C10_intent_followed_by_real_reauthentication_directory_verified=True,
            passive_account_observations=len(matching),passive_rental_town_observed=any(row.get('rental_town')for row in matching))
    profiles[profile]=dict(passed=1,failed=0,actual_sessions=8,product_sha256=info['product_sha256'],roles=roles,
        public_source_C8=8,character_C9=4,only_legitimate_rental_CA01=2,return_C10=2,aggregate_first_reconnect_after_S65_seconds=fallback_seconds,GUI_verified=False)
    for path in d.rglob('*'):
        if path.is_file():raw[profile+'/'+path.relative_to(d).as_posix()]=path
    for path in runtime.rglob('*'):
        if path.is_file()and path.suffix.lower()!='.exe':raw[profile+'/runtime/'+path.relative_to(runtime).as_posix()]=path
identities=[c['runner'],*c['process_history']];filters=' OR '.join('ProcessId='+str(row['pid'])for row in identities)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
probe=subprocess.run(['pwsh','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert probe.returncode==0 and not probe.stderr
live=json.loads(probe.stdout.decode('utf-8-sig'))if probe.stdout.strip()else[];live=[live]if isinstance(live,dict)else live
assert not any(row['ProcessId']==identity['pid']and abs((utc(row['created'])-utc(identity['creation_utc'])).total_seconds())<.001 for row in live for identity in identities)
out=run/'qualified';out.mkdir();(out/'original-owned-CIM.json').write_bytes(probe.stdout);raw['original-owned-CIM.json']=out/'original-owned-CIM.json'
pins={name:sha(path)for name,path in raw.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as archive:
    for name,path in raw.items():archive.write(path,name)
with zipfile.ZipFile(out/'raw.zip')as archive:
    assert set(archive.namelist())==set(pins)and len(archive.namelist())==len(pins)
    for name,digest in pins.items():
        with archive.open(name)as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==digest
record=dict(status='independently_qualified_both_current_rental_return_reconnect_API_profiles_limited',run=str(run),
    source_manifest_sha256=source,cohort_sha256=sha(run/'cohort.json'),profiles=profiles,actual_sessions=16,
    original_session=a.session,terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=0,
    recorded_owned_identities=len(identities),recorded_owned_processes_terminal=True,
    raw_archive_sha256=sha(out/'raw.zip'),raw_files=pins,qualified=True,
    API_rejected_ordinary_channel_without_wire_verified=True,formal_rental_C10_attempt_and_fresh_authentication_return_verified=True,native_restart_quit_handshake_completion_claimed=False,full_library_qualified_by_this_scope=False,
    packet_counts_are_aggregate_across_two_concurrent_accounts=True,
    ordinary_channel_on_rental_host_supported=False,hotel_return_verified=False,
    GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'qualified.json',record);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(qualification=str(out/'qualified.json'),actual_sessions=16,GUI_verified=False),ensure_ascii=False))
