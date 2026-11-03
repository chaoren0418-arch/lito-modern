"""Natural honor deaths, real death ACKs and timed server resurrection."""
import datetime,hashlib,json,sys
from pathlib import Path
def load(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
p=Path(sys.argv[1]);r=load(p);assert r['action_profile']=='honor-respawn'
trace=[json.loads(s) for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
images=[];checks=[]
for role,id in [('a',1),('b',3)]:
    c=r['clients'][role];probe=c['lifecycle_probe'];w=c['end']['world']
    assert c['worker_quiescent'] and c['shutdown']=='closed' and int(c['transport']['epoch'])==2
    assert c['end']['joined_room']['mode']==5 and w['generation']==c['start']['world']['generation']==1
    assert w['map_id']==510 and w['battle_result'] is None and not w['scores']
    assert len(probe['deaths'])==len(probe['revivals'])==2
    assert w['resurrection_packets']==2 and w['death_confirmations_sent']==(2 if role=='b' else 0)
    assert probe['attacks_sent']==w['own_attack_packets'] and (probe['attacks_sent']>0 if role=='a' else probe['attacks_sent']==0)
    assert probe['moves_sent']==w['predicted_moves'] and w['local_corrections']==0
    prior=c['start']['world']['actors']['3']['incarnation']
    for death,revival in zip(probe['deaths'],probe['revivals']):
        d=death['world'];v=revival['world'];a=d['actors']['3'];b=v['actors']['3']
        assert a['dead'] and not b['dead'] and prior<=a['incarnation']<b['incarnation']
        assert [a['x'],a['y']]==[b['x'],b['y']] and b['hp_ratio']>0
        prior=b['incarnation']
        for row,dead in [(death,True),(revival,False)]:
            ww=row['world'];pose=next(x for x in row['render']['frame']['actors'] if x['id']==3)
            assert pose['rendered'] and pose['phase']==('death' if dead else 'idle')
            if dead:assert pose['action_complete']
            if role=='b':assert ww['hp'][0]==(0 if dead else ww['hp'][1])
            own=ww['actors'][str(id)]
            assert any(t['state']['phase']=='BATTLE' and t['state']['object_id']==id and t['state']['source']=='測試乙'
                and [t['state'][k] for k in ['x','y','hp','mp']]==[own['x'],own['y'],ww['hp'][0],ww['mp'][0]] for t in trace)
        if role=='b':
            assert revival['retired_death_frame_rejected']
            if 'retired_held_input_rejected' in revival:assert revival['retired_held_input_rejected']
        checks.append(dict(role=role,old_incarnation=a['incarnation'],new_incarnation=b['incarnation'],position=[b['x'],b['y']]))
    for image in [c['start']['render'],c['render']]+[s['render'] for s in probe['deaths']+probe['revivals']]:
        f=Path(image['png']);assert f.is_file();images.append(dict(path=str(f),sha256=sha(f)))
    assert not w['actors']['3']['dead']
a=r['clients']['a']['end']['world'];b=r['clients']['b']['end']['world']
assert [a['actors']['3'][k] for k in ['x','y','heading','dead']]==[b['actors']['3'][k] for k in ['x','y','heading','dead']]
# Own S_PUT_OBJECT uses the no-ratio sentinel 255, with HP supplied by S_HIT_POINT.
# Remote S_HIT_RATIO is the observer's percentage; the two wire fields differ.
assert a['actors']['3']['hp_ratio']==100 and b['hp'][0]==b['hp'][1]
assert b['actors']['3']['hp_ratio']==r['clients']['b']['start']['world']['actors']['3']['hp_ratio']==255
assert r['clients']['b']['lifecycle_probe']['moves_sent']==1
input_probe=r['clients']['b']['lifecycle_probe']
held_input_verified=all(x.get('retired_held_input_rejected') is True for x in input_probe['revivals']) and input_probe.get('fresh_input_sent') is True
events=[];pins=[];own_ratio_wire=False
for f in Path(r['runtime']).glob('captures/*/frames.jsonl'):
    pins.append(dict(path=str(f),sha256=sha(f)))
    for s in f.read_text(encoding='utf-8-sig').splitlines():
        e=json.loads(s)
        if e['direction']=='server_to_client' and e['opcode']=='0x0B':
            body=bytes.fromhex(e['plain']);at=21
            for _ in range(2):at=body.index(0,at)+1
            at+=5 # status and emblem
            for _ in range(2):at=body.index(0,at)+1 # clan and master
            if int.from_bytes(body[5:9],'little')==3 and body[at+1]==255:own_ratio_wire=True
        if e['opcode'] not in ['0xE0','0x11','0x17','0x0A']:continue
        e['body']=bytes.fromhex(e['plain']);events.append(e)
acks=[e for e in events if e['direction']=='client_to_server' and e['body']==bytes([0xe0])]
revives=[e for e in events if e['direction']=='server_to_client' and e['body'][0]==0x11]
attacks=[e for e in events if e['direction']=='client_to_server' and e['body'][0]==0x17]
assert len(acks)==2 and len(revives)==4 and own_ratio_wire
assert len(attacks)==r['clients']['a']['lifecycle_probe']['attacks_sent']
for e in revives:
    body=e['body'];assert len(body)==12 and int.from_bytes(body[1:5],'little')==int.from_bytes(body[6:10],'little')==3 and body[5]==0
seconds=lambda e:datetime.datetime.fromisoformat(e['time'].replace('Z','+00:00')).timestamp()
acks.sort(key=seconds);revives.sort(key=seconds)
moves=[e for e in events if e['direction']=='client_to_server' and e['body'][0]==0x0a]
post_respawn_seconds=max(map(seconds,moves))-max(map(seconds,revives))
assert post_respawn_seconds>=9,post_respawn_seconds
delays=[]
for index,ack in enumerate(acks):
    delay=seconds(revives[index*2])-seconds(ack);assert 4.9<=delay<15,delay;delays.append(delay)
hp=[t['state']['hp'] for t in trace if t['state']['object_id']==3 and t['state']['phase']=='BATTLE']
transitions=[(x,y) for x,y in zip(hp,hp[1:]) if (x==0)!=(y==0)]
assert sum(x>0 and y==0 for x,y in transitions)==2 and sum(x==0 and y>0 for x,y in transitions)==2
report=dict(status='verified_real_peer_lifecycle_authority',profile='honor-respawn',jar_sha256=r['jar_sha256'],exe_sha256=r['exe_sha256'],
    receipt_sha256=sha(p),trace_sha256=sha(p.parent/'server-trace.jsonl'),capture_pins=pins,checks=checks,images=images,
    death_ack_to_resurrection_seconds=delays,post_respawn_movement_delay_seconds=post_respawn_seconds,natural_deaths=2,held_input_lifetime_verified=held_input_verified,gui_verified=False,all_modes_verified=False)
(p.parent/'authority-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified two natural honor deaths, timed respawns, retired input and resumed movement')
