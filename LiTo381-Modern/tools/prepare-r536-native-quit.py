import hashlib
import json
import pathlib
import shutil

p = pathlib.Path(__file__).resolve().parent.parent
l = p / 'logs/continuation-20261003'
dest = p / 'target/r536-native-pet-race-quit-inputs'
def read(f): return json.loads(f.read_text(encoding='utf-8-sig'))
def pin(f):
    h = hashlib.sha256()
    with f.open('rb') as s:
        for b in iter(lambda: s.read(1024 * 1024), b''): h.update(b)
    return {'path': str(f), 'bytes': f.stat().st_size, 'sha256': h.hexdigest().upper()}
def check(d):
    a = pin(pathlib.Path(d['path']))
    assert (a['bytes'], a['sha256']) == (d['bytes'], d['sha256'].upper()), a
def save(f,v):
    with f.open('x', encoding='utf-8') as s:
        json.dump(v,s,ensure_ascii=False,indent=2); s.write('\n')

root = p.parent.parent / 'lito伺服器/Native-PetRace-D9-IOException退出完成候選交接.json'
check({'path':str(root),'bytes':4336,'sha256':'263ABBE9B775743D5D3B581FDA21CDE6E778A30A9EA1C3301552601C7313D4F0'})
v = read(root)
refs = [d for d in v.values() if isinstance(d,dict) and {'path','bytes','sha256'} <= d.keys()]
assert len(refs) == 11
for d in refs: check(d)
m = read(pathlib.Path(v['manifest']['path']))
owner = pathlib.Path(v['root']).resolve()
assert len(m['files']) == 670
for d in m['files']:
    assert pathlib.Path(d['path']).resolve().is_relative_to(owner)
    check(d)
dest.mkdir()
shutil.copyfile(root, l / 'r536-quit-finalization-server-root.json')
copied = []
for key in ['manifest','source_diff_from_64A40','version_binding','lifecycle_contract','verification','candidate_result','baseline_result']:
    f = pathlib.Path(v[key]['path']); to = dest / (key+f.suffix)
    shutil.copyfile(f,to); copied.append(pin(to))
result = read(pathlib.Path(v['candidate_result']['path']))
assert result['delivery_ACK'] is False and result['post_call_fixture_state_mutations'] == 0
cases=[]
for c in result['cases']:
    if c['scenario'].startswith('non_PetRace'): continue
    streams=[]
    for i,role in enumerate(['quitter','first-peer','healthy-peer']):
        logfiles=list((owner / 'candidate/run' / c['scenario'] / f'capture-{i}').glob('*/frames.jsonl'))
        assert len(logfiles)==1
        f=logfiles[0]
        records=[json.loads(line) for line in f.read_text(encoding='utf-8').splitlines() if line]
        complete=[r for r in c['all_transport'][i]['frames'] if r['complete_flush']]
        assert len(records)==len(complete)==c['all_transport'][i]['completed_frames']
        bodies=[]
        for r,tr in zip(records,complete):
            body=bytes.fromhex(r['plain'])
            assert r['direction']=='server_to_client'
            assert len(body)+2==r['wire_length']==tr['bytes_written']
            assert int(r['opcode'],16)==body[0]
            assert body[0] in [0xF0,0xCC,0x72,0x0F,0xD3]
            bodies.append({'ordinal':tr['ordinal'],'body_hex':body.hex().upper(),'server_complete_flush':True,
                           'client_delivery_proven':False,'source_note':r['note']})
        first=c['first_transport'][i]['completed_frames']
        to=dest / f'{c["scenario"]}-{role}-server-frames.jsonl'
        shutil.copyfile(f,to); copied.append(pin(to))
        streams.append({'role':role,'first_bodies':bodies[:first],'duplicate_bodies':bodies[first:],
                        'first_failed_attempts':[r['ordinal'] for r in c['first_transport'][i]['frames'] if not r['complete_flush']],
                        'server_poststate':c['first_poststate']['sessions'][i], 'source_log':pin(f), 'owned_log':pin(to)})
    cases.append({'scenario':c['scenario'],'streams':streams,'server_room_poststate':{
        k:c['first_poststate'][k] for k in ['battleStarted','resultPending','battleGeneration','room_member_count']}})
assert len(cases)==6
replay={'schema':'MODERN_R536_SERVER_GENERATED_QUIT_REPLAY_V1','candidate_source':v['candidate_source'],
        'candidate_result':v['candidate_result'],'client_delivery_ACK':False,
        'interpretation':'Bodies are actual successful application records paired with complete local flushes. Tests explicitly assume selected bodies arrive in this offline sequence; neither result metadata nor flush proves receipt.',
        'cases':cases,
        'missing':['Actual RESULT C_D0 confirmation method output: successful own S_D0 body plus following CC0 rows, including failed/empty list variants.',
                   'Wire generation/correlation token; no fields distinguish untagged late F0/CC from another intent.',
                   'Zero-row room-list completion marker and deadline; partial transport/client ACK and live receipt unavailable.']}
save(dest/'actual-application-replay.json',replay)
save(l/'r536-quit-finalization-adoption.json',{'state':'READ_ONLY_CANDIDATE_PINNED_WITH_ACTUAL_APPLICATION_REPLAY',
    'root':pin(root),'refs_checked':11,'manifest_files_checked':670,'copied':copied,
    'replay':pin(dest/'actual-application-replay.json'),'PET_cases':6,'streams':18,
    'old_64A40_5ABC_are_historical':True,'deployed_485F_adopted':False,'new_Java_calls':0,
    'SQL_TLS_provider_GUI':0,'client_received_assumption_only':True,'missing':replay['missing']})
print(json.dumps({'PET_cases':6,'streams':18,'actual_successful_bodies':sum(len(s['first_bodies'])+len(s['duplicate_bodies']) for c in cases for s in c['streams']),
                  'replay':pin(dest/'actual-application-replay.json')},ensure_ascii=False))
