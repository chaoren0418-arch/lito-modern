"""Independent two-client, authority, resource, cure and natural-expiry checks."""
from pathlib import Path
import hashlib, json, sys
from PIL import Image

def read(p): return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f: return hashlib.file_digest(f,'sha256').hexdigest().upper()

p=Path(sys.argv[1]).resolve();r=read(p);profile=r['action_profile'];full=profile=='poison-cure'
assert profile in ['poison-cure','poison-missing','poison-refused','cure-refused']
assert r['jar_sha256']=='9719F45DB324206617726CE06D1A9E2184E9A3B3AEFAB3D50F3EBBDFBF09CFAC'
assert sha(Path(r['runtime'])/'qualified-client.exe')==r['exe_sha256']
assert r['server']['actual_client_sessions']==4 and r['server']['memberships_empty']
assert r['server']['authority_mutations']==0 and not r['server']['sql_access'] and not r['server']['state_persistent']
trace=[json.loads(line) for line in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
states=[row['state'] for row in trace];reports={};pngs=[]
for role,id in [('a',1),('b',3)]:
    c=r['clients'][role];probe=c['poison_probe'];w=probe['endpoint']['world'];reports[role]=probe
    assert c['worker_quiescent'] and c['shutdown']=='closed' and int(c['transport']['epoch'])==2
    assert c['start']['world']['status']['object_id']==w['status']['object_id']==id
    assert c['end']['selected_source']=='測試乙' and w['own_attack_packets']==0
    assert probe['runtime_authority_mutations']==0
    assert 9 in c['start']['world']['learned_skills'] if role=='b' else 11 in c['start']['world']['learned_skills']
    if role=='b' and profile.startswith('poison-'): assert 11 in c['start']['world']['learned_skills']
    for snapshot in [cast['after']['world'] for cast in probe['casts']]+[w]:
        own=snapshot['actors'][str(id)]
        assert any(s['object_id']==id and s['phase']=='BATTLE' and s['source']=='測試乙'
                   and [s['hp'],s['mp'],s['x'],s['y']]==[snapshot['hp'][0],snapshot['mp'][0],own['x'],own['y']] for s in states)
    for image in probe['images']+[c['start']['render'],c['render']]:
        path=Path(image['png']);frame=image['frame']
        with Image.open(path) as pixels:
            assert pixels.size==(800,600)
            assert hashlib.sha256(pixels.convert('RGBA').tobytes()).hexdigest().upper()==frame['rgba_sha256'].upper()
        pngs.append({'path':str(path),'sha256':sha(path)})
    assert all(w['actors'][str(i)]['draw_state']['green'] is False for i in [1,3])

caster=reports['b'];target=reports['a'];assert not target['casts']
assert target['moves_sent']>0 and caster['moves_sent']==0
if not full:
    assert len(caster['casts'])==1 and caster['endpoint']['world']['own_cast_packets']==0
    expected={'Rejected':299} if profile=='poison-missing' else 'NoResponse'
    assert caster['endpoint']['skill_feedback']==expected
    cast=caster['casts'][0]
    assert cast['before']['world']['mp']==cast['after']['world']['mp'] and cast['before']['inventory']==cast['after']['inventory']
    assert all(not s['poison_active'] for s in states)
    assert all(not row['effect']['effects'] and not row['effect']['entered_events'] for probe in reports.values() for row in probe['frames'])
else:
    poison=[cast for cast in caster['casts'] if cast['skill']==11];cures=[cast for cast in caster['casts'] if cast['skill']==9]
    assert 2<=len(poison)<=8 and sum(cast['applied'] for cast in poison)==2 and len(cures)==1
    assert caster['attempts']==len(poison) and all(probe['episodes_observed']==2 for probe in reports.values())
    for cast in caster['casts']:
        assert cast['no_prediction'] and cast['duplicate_pending_rejected']
        before=cast['before'];after=cast['after'];skill=cast['skill']
        assert before['world']['mp'][0]-after['world']['mp'][0]==(10 if skill==11 else 15)
        assert after['world']['own_cast_packets']==before['world']['own_cast_packets']+1
        items=lambda s:sum(i['count'] for i in s['inventory']['items'].values())
        assert items(before)-items(after)==(1 if skill==11 else 0)
        if skill==11:
            assert after['skill_feedback']==('CastAction' if cast['applied'] else {'Rejected':280})
        else: assert not after['world']['actors']['1']['draw_state']['green']
    assert any(s['object_id']==3 and s['phase']=='BATTLE' and s['magic_gems']==8-len(poison) for s in states)
    for role,probe in reports.items():
        drawn={};events={};audio={}
        for row in probe['frames']:
            fx=row['effect'];assert fx['load_error'] is None and fx['omitted']==0
            for visual in fx['effects']:
                if visual['drawn']:
                    seq=visual['sequence'];gfx=visual['graphics'];assert gfx in [745,871] and visual['target'][0]==1
                    assert visual['sprite']==('192-0.spr' if gfx==871 else '745-0.spr')
                    drawn.setdefault((seq,gfx),set()).add(visual['subframe'])
            for event in fx['entered_events']:
                key=(event['sequence'],event['sound']);events[key]=events.get(key,0)+1
            for cue in row['audio']:
                if cue['event']['event'].startswith('EffectSound('):
                    assert cue['prepared'];sound=cue['event']['sound']
                    expected_sound={117:(82602,'0F5B6A26722A111F55D89DBDDB39D329B780AE340972B0F65AE2173EE2C8477F'),
                                    141:(77866,'48BE689ACC812210ABFD2D48786807F96A424D5EC6089318F394720FA7B6E803')}[sound]
                    assert (cue['bytes'],cue['sha256'].upper())==expected_sound
                    key=(cue['event']['sequence'],sound);audio[key]=audio.get(key,0)+1
        assert len(drawn)==3 and sorted(gfx for seq,gfx in drawn)==[745,745,871]
        assert all({0,7 if gfx==745 else 5}<=subframes for (seq,gfx),subframes in drawn.items())
        expected={(seq,117 if gfx==745 else 141):1 for seq,gfx in drawn}
        assert events==expected and audio==expected
        green_images=[image for image in probe['images'] if 'poison-green-' in image['png']]
        clear_images=[image for image in probe['images'] if 'poison-clear-' in image['png']]
        assert len(green_images)==len(clear_images)==2
        for image in green_images:
            actor=next(a for a in image['frame']['actors'] if a['id']==1)
            assert actor['rendered'] and actor['draw_state']['green'] and 'Green' in actor['draw_style']
        for image in clear_images:
            actor=next(a for a in image['frame']['actors'] if a['id']==1)
            assert actor['rendered'] and not actor['draw_state']['green'] and actor['draw_style']=='Normal'
    target_rows=[row for row in trace if row['state']['object_id']==1 and row['state']['phase']=='BATTLE']
    deadlines=sorted({row['state']['poison_expires_at_nanos'] for row in target_rows if row['state']['poison_active']})
    assert len(deadlines)==2
    durations=[]
    for deadline in deadlines:
        active=[row for row in target_rows if row['state']['poison_active'] and row['state']['poison_expires_at_nanos']==deadline]
        assert deadline-active[0]['state']['poison_next_tick_at_nanos']==27_000_000_000
        cleared=next(row for row in target_rows if row['ms']>active[-1]['ms'] and not row['state']['poison_active'])
        durations.append(cleared['ms']-active[0]['ms'])
    assert 2800<=durations[0]<10000 and 29800<=durations[1]<=30300,durations
    decreases=[previous['state']['hp']-current['state']['hp'] for previous,current in zip(target_rows,target_rows[1:]) if current['state']['hp']<previous['state']['hp']]
    assert len(decreases)>=10 and all(delta==5 for delta in decreases),decreases
    for episode in [1,2]:
        clean=read(Path(r['runtime'])/f'a-poison-clean-{episode}')
        assert not clean['world']['actors']['1']['draw_state']['green']
    config=(Path(r['runtime'])/'server.properties').read_text(encoding='utf-8')
    assert 'curse.poison.spell.fixture.enabled=true' in config
    assert 'curse.poison.fixture.die.roll=' not in config and 'curse.poison.fixture.percentile.roll=' not in config

report={'status':'verified_real_peer_poison_cure_scope','profile':profile,'receipt_sha256':sha(p),
        'exe_sha256':r['exe_sha256'],'jar_sha256':r['jar_sha256'],'actual_client_sessions':4,
        'pngs':pngs,'natural_poison_and_cure_verified':full,'initial_fixture':r['server']['initial_fixture'],
        'gui_verified':False,'physical_audio_device_verified':False,'full_game_verified':False}
if full: report.update(poison_attempts=len(poison),observed_poison_durations_ms=durations,observed_damage_ticks=len(decreases))
(p.parent/'authority-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified real-peer poison profile: '+profile)
