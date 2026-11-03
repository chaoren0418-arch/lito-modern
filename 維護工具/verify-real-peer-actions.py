"""Verify actual two-peer actions against independent formal session observations."""
import hashlib
import json
import sys
from PIL import Image
from pathlib import Path

def load(p): return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f: return hashlib.file_digest(f,'sha256').hexdigest().upper()

p=Path(sys.argv[1]).resolve()
r=load(p); profile=r['action_profile']
trace=[json.loads(line)['state'] for line in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
active='b' if profile in ['heal','item-heal'] or profile.startswith('holy-weapon') or (profile.startswith('chill-touch') or profile.startswith('vampiric-touch')) else 'a'
checks=[]
roles=[('a',1),('b',3)]+([('c',6)] if profile=='lightning-multi' else [])
for role,id in roles:
    c=r['clients'][role]; probe=c['action_probe']; w=probe['endpoint']['world']
    assert c['worker_quiescent'] and c['shutdown']=='closed' and int(c['transport']['epoch'])==2
    assert c['action_profile']==profile and c['end']['selected_source']=='測試乙'
    assert c['start']['world']['status']['object_id']==w['status']['object_id']==id
    assert c['start']['world']['hp'][0]==c['start']['world']['hp'][1]
    assert all(str(i) in c['start']['world']['actors'] for i in ([id] if profile=='lightning-multi' else [1,3]))
    if profile=='lightning-multi':assert all(str(i) in w['actors'] for _,i in roles)
    expected=probe['attacks_sent'] if profile=='ranged' else (2 if profile in ['bolt','fire-arrow','wind-cutter','ice-arrow','item-heal','chill-touch','stalac','lightning','lightning-multi'] else 1)
    assert 2<=len(probe['actions'])<=12 if profile=='turn-undead-player' and role==active else len(probe['actions'])==(expected if role==active else 0)
    if profile=='ranged' and role==active: assert 1<=expected<=16
    snapshots=[a['after']['world'] for a in probe['actions']]+[w]
    for v in snapshots:
        own=v['actors'][str(id)]
        matches=[s for s in trace if s['object_id']==id and s['phase'] in ['BATTLE','RESULT']
            and s['hp']==v['hp'][0] and s['mp']==v['mp'][0]
            and [s['x'],s['y']]==[own['x'],own['y']] and s['source']=='測試乙']
        assert matches, (profile,role,'unmatched server state',v['hp'],v['mp'],own['x'],own['y'])
    assert probe['attacks_sent']==w['own_attack_packets'], 'each sent physical action must be acknowledged'
    other=r['clients']['b' if role=='a' else 'a']['action_probe']['endpoint']['world']
    own=w['actors'][str(id)]; remote=other['actors'][str(id)]
    assert [own['x'],own['y'],own['dead']]==[remote['x'],remote['y'],remote['dead']]
    images=[]
    for image in [c['start']['render'],c['render']]:
        image_path=Path(image['png']); assert image_path.is_file() and image['frame']['map_id']==564
        images.append({'path':str(image_path),'sha256':sha(image_path)})
    checks.append({'role':role,'hp':w['hp'],'mp':w['mp'],'attacks':probe['attacks_sent'],'actions':len(probe['actions']),'images':images})

actions=r['clients'][active]['action_probe']['actions']
for action in actions:
    before=action['before']['world']; after=action['after']['world']
    assert action['no_prediction'] and action['duplicate_pending_rejected']
    if profile.endswith('refused'):
        assert before['mp']==after['mp']==[2,2] and after['own_cast_packets']==0
    elif (profile.startswith('chill-touch') or profile.startswith('vampiric-touch')):
        assert before['mp'][0]-after['mp'][0]==(12 if profile.startswith('vampiric-touch') else 8)
        assert after['hp'][0]>before['hp'][0] if profile in ['chill-touch','vampiric-touch'] else after['hp'][0]==after['hp'][1]
    elif profile.startswith('holy-weapon'):
        assert before['mp'][0]-after['mp'][0]==10
        assert before['hp']==after['hp']
    elif profile=='turn-undead-player':
        assert before['mp'][0]-after['mp'][0]==20 and before['hp']==after['hp']
        assert after['actors']['3']['hp_ratio']==before['actors']['3']['hp_ratio']==100
        assert not after['actors']['3']['dead']
    elif profile=='heal':
        assert before['hp'][0]<after['hp'][0]<=after['hp'][1]
        assert before['mp'][0]-after['mp'][0]==4
    elif profile=='bolt': assert before['mp'][0]-after['mp'][0]==3
    elif profile=='lightning': assert before['mp'][0]-after['mp'][0]==20
    elif profile=='stalac': assert before['mp'][0]-after['mp'][0]==6
    elif profile=='fire-arrow': assert before['mp'][0]-after['mp'][0]==6
    elif profile in ['wind-cutter','ice-arrow']: assert before['mp'][0]-after['mp'][0]==4
    elif profile=='wall': assert before['mp'][0]-after['mp'][0]==60
    elif profile=='item-heal': assert before['hp'][0]<=after['hp'][0]
if profile in ['bolt','fire-arrow','wind-cutter','ice-arrow']:
    for visual_role in (['a','b'] if profile in ['wind-cutter','ice-arrow'] else [active]):
        frames=r['clients'][visual_role]['action_probe']['frames']
        expected_graphics={'bolt':{167,219},'fire-arrow':{1583,218},'wind-cutter':{1799,1800},'ice-arrow':{1797,1798}}[profile]
        drawn={}
        sounds={}
        secondary_hits={}
        for frame in frames:
            effect=frame['effect']
            assert effect['load_error'] is None and effect['omitted']==0
            for flight in effect['flights']:
                if flight['drawn']:drawn.setdefault(flight['sequence'],set()).add(flight['graphics'])
            if profile in ['wind-cutter','ice-arrow']:
                for cue in effect['non_audio_cues']:
                    assert cue['event']=='Hit' and cue['source_frame']==1 and cue['authority_callback'] is False
                    sequence=cue['sequence'];secondary_hits[sequence]=secondary_hits.get(sequence,0)+1
            for event in effect['entered_events']:
                key=(event['sequence'],event['sound']);sounds[key]=sounds.get(key,0)+1
        assert len(drawn)==2 and all(g==expected_graphics for g in drawn.values()), drawn
        if profile in ['wind-cutter','ice-arrow']:assert secondary_hits=={sequence:1 for sequence in drawn},secondary_hits
        wanted={'bolt':{97},'fire-arrow':{97,99},'wind-cutter':{719,711},'ice-arrow':{719,713}}[profile]
        assert sounds=={(sequence,sound):1 for sequence in drawn for sound in wanted}, sounds
if (profile.startswith('chill-touch') or profile.startswith('vampiric-touch')):
    caster=r['clients']['b']['action_probe']; victim=r['clients']['a']['action_probe']
    if profile.endswith('refused'):
        assert caster['endpoint']['skill_feedback']=={'Rejected':278}
        assert not caster['endpoint']['world']['projectiles']
    else:
        import re
        skill,gfx,sound=(28,236,103) if profile.startswith('vampiric-touch') else (10,252,110)
        lines=(p.parent/'server.log').read_text(encoding='utf-8-sig').splitlines()
        commits=[]
        for line in lines:
            if f'C_USE_SPELL attack skill={skill} ' not in line:continue
            hp=re.search(r' hp=(\d+)/(\d+)',line)
            commits.append({'damage':int(re.search(r' damage=(\d+)',line)[1]),'heal':int(re.search(r'life-drain-healing=(\d+)',line)[1]),
                            'before_hp':int(hp[1]),'max_hp':int(hp[2]),'mp':int(re.search(r' mp=(\d+)/',line)[1])})
        for role in ['a','b']:
            probe=r['clients'][role]['action_probe'];drawn={};audio={};hits={};hurt={}
            for frame in probe['frames']:
                fx=frame['effect'];assert fx['load_error'] is None and fx['omitted']==0
                if fx.get('sources'):
                    if any(source['name'].startswith(f'{gfx}-') for source in fx['sources']):assert {f'{gfx}-{h}.spr' for h in range(8)}<={source['name'] for source in fx['sources']}
                    assert all(source['name'].split('-')[0] in ['236','252','1801'] for source in fx['sources'])
                    assert all(len(source['sha256'])==64 and source['bytes']>0 for source in fx['sources'])
                for visual in fx['effects']:
                    assert 'error' not in visual
                    if visual['drawn']:
                        assert fx.get('sources') is not None and len(fx['sources']) in [8,9,16,17]
                        assert visual['graphics']==gfx and visual['sprite']==f"{gfx}-{visual['heading']}.spr"
                        drawn.setdefault(visual['sequence'],set()).add(visual['subframe'])
                for event in fx['entered_events']:
                    assert event['sound']==sound;key=(event['sequence'],event['sound']);audio[key]=audio.get(key,0)+1
                for hit in fx['non_audio_cues']:
                    assert hit['event']=='Hit' and hit['source_frame']==5 and not hit['authority_callback']
                    hits[hit['sequence']]=hits.get(hit['sequence'],0)+1
                for hit in fx['hurt_callbacks']:
                    assert hit['accepted'] and hit['target']==1;hurt[hit['sequence']]=hurt.get(hit['sequence'],0)+1
            expected=2 if profile=='chill-touch' else 1
            assert len(drawn)==expected and all({0,5}<=frames for frames in drawn.values()),drawn
            assert hits==hurt=={seq:1 for seq in drawn}
            assert audio=={(seq,sound):1 for seq in drawn}
            prepared=[cue for image in probe['images'] for cue in image['frame']['audio'] if cue['event']['sound']==sound]
            expected_wave={103:(32050,'6D5DE959A75E3153C5360668B4774BFC60AC2D092CB5470126557DB5F1268C98'),
                           110:(63108,'4988D5726CB361A715960C41444D9C45F685DEB8A719558B872D31322568BDFC')}[sound]
            assert len(prepared)==1 and prepared[0]['prepared']
            assert (prepared[0]['bytes'],prepared[0]['sha256'].upper())==expected_wave
            for image in probe['images']:
                with Image.open(image['png']) as pixels:assert hashlib.sha256(pixels.convert('RGBA').tobytes()).hexdigest().upper()==image['frame']['rgba_sha256'].upper()
        assert len(commits)==expected,commits
        for index,commit in enumerate(commits):
            assert commit['damage']>0 and commit['heal']==min(commit['damage'],commit['max_hp']-commit['before_hp'])
            after_hp=commit['before_hp']+commit['heal']
            assert any(row['object_id']==3 and row['phase']=='BATTLE' and row['hp']==after_hp and row['mp']==commit['mp'] for row in trace)
            assert any(row['hp'][0]==after_hp and row['mp'][0]==commit['mp'] for row in caster['observed_health'])
            before=caster['actions'][index]['before']['world'];after=caster['actions'][index]['after']['world']
            assert before['hp'][0]<=commit['before_hp']<=after['hp'][0]
            assert any(row['hp'][0]==commit['before_hp'] and row['mp'][0]==before['mp'][0] for row in caster['observed_health'])
        if profile in ['chill-touch','vampiric-touch']:assert all(commit['heal']>0 for commit in commits)
        else:assert commits[0]['heal']==0
if profile.startswith('lightning'):
    caster=r['clients']['a']['action_probe']
    if profile.endswith('refused'):
        assert caster['endpoint']['skill_feedback']=={'Rejected':278}
        assert not caster['endpoint']['world']['many_attacks']
    else:
        original=load(p.parent/'source-snapshot/lightning-original-sources.json')
        lines=(p.parent/'server.log').read_text(encoding='utf-8-sig').splitlines()
        commits=[line for line in lines if 'C_USE_SPELL line skill=17 ' in line]
        target_ids=[3,6] if profile=='lightning-multi' else [3]
        assert len(commits)==2 and all(' primary=3 ' in line and f' targets={len(target_ids)} ' in line for line in commits)
        for role,_ in roles:
            probe=r['clients'][role]['action_probe'];drawn={};audio={};hits={};hurt={}
            cues={v['action_sequence']:v for v in probe['endpoint']['world']['many_attacks']}
            assert len(cues)==2
            for frame in probe['frames']:
                fx=frame['effect'];assert fx['load_error'] is None and fx['omitted']==0
                for visual in fx['effects']:
                    assert 'error' not in visual and visual['graphics']==170
                    assert visual['sprite']==f"170-{visual['heading']}.spr"
                    cue=cues[visual['sequence']];assert cue['attack']['caster']==1 and cue['attack']['graphics']==170
                    assert visual['position']==cue['attack']['position'] and visual['heading']==cue['attack']['heading']
                    assert cue['attack']['targets']==[{'id':id,'hit':True} for id in target_ids]
                    assert {f'170-{h}.spr' for h in range(8)}=={source['name'] for source in fx['sources']}
                    for source in fx['sources']:
                        expected=next(v for v in original['sprites'] if v['member']==source['name'])
                        assert (source['bytes'],source['sha256'].upper())==(expected['bytes'],expected['sha256'])
                    if visual['drawn']:drawn.setdefault(visual['sequence'],set()).add(visual['source_frame'])
                for event in fx['entered_events']:
                    assert event['sound']==101 and event['frame']==1
                    key=(event['sequence'],101);audio[key]=audio.get(key,0)+1
                for hit in fx['non_audio_cues']:
                    assert hit['event']=='Hit' and hit['source_frame']==2 and not hit['authority_callback']
                    hits[hit['sequence']]=hits.get(hit['sequence'],0)+1
                for hit in fx['hurt_callbacks']:
                    assert hit['accepted'] and hit['target'] in target_ids;key=(hit['sequence'],hit['target']);hurt[key]=hurt.get(key,0)+1
            assert len(drawn)==2 and all({0,1}<=frames for frames in drawn.values()),drawn
            assert hits=={seq:1 for seq in drawn} and hurt=={(seq,id):1 for seq in drawn for id in target_ids},(hits,hurt)
            assert audio=={(seq,101):1 for seq in drawn},audio
            prepared=[cue for image in probe['images'] for cue in image['frame']['audio'] if cue['event']['sound']==101]
            assert len(prepared)==1 and prepared[0]['prepared']
            assert (prepared[0]['bytes'],prepared[0]['sha256'].upper())==(original['sound']['bytes'],original['sound']['sha256'])
            for image in probe['images']:
                with Image.open(image['png']) as pixels:assert hashlib.sha256(pixels.convert('RGBA').tobytes()).hexdigest().upper()==image['frame']['rgba_sha256'].upper()

if profile.startswith('stalac'):
    caster=r['clients']['a']['action_probe']
    if profile.endswith('refused'):
        assert caster['endpoint']['skill_feedback']=={'Rejected':278}
        assert not caster['endpoint']['world']['projectiles']
    else:
        original=load(p.parent/'source-snapshot/stalac-original-sources.json')
        import re
        lines=(p.parent/'server.log').read_text(encoding='utf-8-sig').splitlines()
        commits=[line for line in lines if 'C_USE_SPELL attack skill=16 ' in line]
        formula=[line for line in lines if 'ordinary magic damage skill=16 ' in line]
        assert len(commits)==len(formula)==2
        for line in commits:
            assert int(re.search(r' damage=(\d+)',line)[1])>0
            assert ' target=3 npc=0 ' in line and ' gfx=1801 ' in line and 'mp-cost=6' in line and 'life-drain-healing=0' in line
        assert all(' attr=2 ' in line for line in formula)
        for role in ['a','b']:
            probe=r['clients'][role]['action_probe'];drawn={};audio={};hits={};hurt={}
            for frame in probe['frames']:
                fx=frame['effect'];assert fx['load_error'] is None and fx['omitted']==0
                for visual in fx['effects']:
                    assert 'error' not in visual
                    assert visual['graphics']==1801 and visual['sprite']=='1801-0.spr'
                    body=next(source for source in fx['sources'] if source['name']=='1801-0.spr')
                    assert (body['bytes'],body['sha256'].upper())==(original['sprites'][0]['bytes'],original['sprites'][0]['sha256'])
                    assert len(visual['layers'])==2
                    for layer,name,mix,index in zip(visual['layers'],['1803-0.spr','1802-0.spr'],['Screen','Shadow'],[1,2]):
                        assert layer['sprite']==name and layer['mix']==mix and layer['subframe']==visual['subframe']
                        assert layer['drawn']==visual['drawn'] and layer['source']==original['sprites'][index]
                    if visual['drawn']:drawn.setdefault(visual['sequence'],set()).add(visual['subframe'])
                for event in fx['entered_events']:
                    assert event['sound']==712 and event['frame']==1
                    key=(event['sequence'],712);audio[key]=audio.get(key,0)+1
                for hit in fx['non_audio_cues']:
                    assert hit['event']=='Hit' and hit['source_frame']==4 and not hit['authority_callback']
                    hits[hit['sequence']]=hits.get(hit['sequence'],0)+1
                for hit in fx['hurt_callbacks']:
                    assert hit['accepted'] and hit['target']==3;hurt[hit['sequence']]=hurt.get(hit['sequence'],0)+1
            assert len(drawn)==2 and all({0,8}<=subs for subs in drawn.values()),drawn
            assert hits==hurt=={seq:1 for seq in drawn},(hits,hurt)
            assert audio=={(seq,712):1 for seq in drawn},audio
            prepared=[cue for image in probe['images'] for cue in image['frame']['audio'] if cue['event']['sound']==712]
            assert len(prepared)==1 and prepared[0]['prepared']
            assert (prepared[0]['bytes'],prepared[0]['sha256'].upper())==(original['sound']['bytes'],original['sound']['sha256'])
            for image in probe['images']:
                with Image.open(image['png']) as pixels:assert hashlib.sha256(pixels.convert('RGBA').tobytes()).hexdigest().upper()==image['frame']['rgba_sha256'].upper()

if profile.startswith('holy-weapon'):
    refused=profile.endswith('refused')
    caster=r['clients']['b']['action_probe']; recipient=r['clients']['a']['action_probe']
    message=recipient['endpoint']['world']['last_message']
    if refused:
        assert caster['endpoint']['skill_feedback']=='NoResponse'
        assert caster['endpoint']['world']['own_cast_packets']==0
        assert not caster['endpoint']['world']['effects'] and not recipient['endpoint']['world']['effects']
        assert all('skillId=0' in row['weapon_buff'] for row in trace)
    else:
        assert message[0]==(161 if profile=='holy-weapon' else 79), message
        for role in ['a','b']:
            probe=r['clients'][role]['action_probe']; drawn={};sounds={}
            for frame in probe['frames']:
                effect=frame['effect']
                assert effect['load_error'] is None and effect['omitted']==0
                for visual in effect['effects']:
                    if visual['drawn']:
                        assert visual['graphics']==2165 and visual['action']==0 and visual['target'][0]==1
                        drawn.setdefault(visual['sequence'],set()).add(visual['subframe'])
                for event in effect['entered_events']:
                    assert event['sound']==804 and event['action']==0 and event['frame']==1
                    key=(event['sequence'],event['sound']);sounds[key]=sounds.get(key,0)+1
            assert len(drawn)==1 and all(12 in frames and 0 in frames for frames in drawn.values()),drawn
            assert sounds=={(sequence,804):1 for sequence in drawn},sounds
            assert probe['endpoint']['world']['hp']==r['clients'][role]['start']['world']['hp']
            assert len(probe['images'])==2
            prepared=[audio for image in probe['images'] for audio in image['frame']['audio'] if audio['event']['sound']==804]
            assert len(prepared)==1 and prepared[0]['prepared'] is True
            assert prepared[0]['bytes']==335476 and prepared[0]['sha256'].upper()=='A1CCF06439DD3FDF721016B85BB08563A07F16E69CE9AD8D18C2AC07E4D36117'

            for image in probe['images']:
                with Image.open(image['png']) as pixels:
                    assert pixels.size==(800,600)
                    assert hashlib.sha256(pixels.convert('RGBA').tobytes()).hexdigest().upper()==image['frame']['rgba_sha256'].upper()
                assert any(visual['graphics']==2165 and visual['drawn'] for visual in image['frame']['skill_effects']['effects'])

        buff=[row['weapon_buff'] for row in trace if row['object_id']==1 and row['phase']=='BATTLE']
        if profile=='holy-weapon':
            assert any('skillId=8, meleeHit=1, meleeDamage=1, rangedHit=0, rangedDamage=0' in value for value in buff),buff
        else:
            assert all('skillId=0' in value for value in buff),buff
if profile=='turn-undead-player':
    import re
    commits=[line for line in (p.parent/'server.log').read_text(encoding='utf-8-sig').splitlines() if 'C_USE_SPELL Turn Undead caster=' in line]
    assert len(commits)==len(actions)
    outcomes=[]
    for action,line in zip(actions,commits):
        assert ' target=3 ' in line and ' npc=0 ' in line
        probability=int(re.search(r' probability=(\d+)',line)[1]);roll=int(re.search(r' roll=(\d+)',line)[1]);success=' success=true ' in line
        assert 0<=probability<=90 and 1<=roll<=100 and success==(roll<=probability)
        assert action['turn_undead_success']==success
        assert action['after']['skill_feedback']==('CastAction' if success else {'Rejected':280})
        assert len(action['after']['world']['effects'])==len(action['before']['world']['effects'])+int(success)
        outcomes.append(success)
    assert any(outcomes) and not all(outcomes)
    original=load(p.parent/'source-snapshot/turn-undead-original-sources.json')
    for role,id in roles:
        probe=r['clients'][role]['action_probe'];w=probe['endpoint']['world'];assert w['hp']==r['clients'][role]['start']['world']['hp'] and not w['actors'][str(id)]['dead']
        cues={v['sequence']:v for v in w['effects']};assert len(cues)==sum(outcomes) and all(v['graphics']==754 and v['target'][0]==3 for v in cues.values())
        drawn={};finished=set();sounds={};prepared=[]
        for f in probe['frames']:
            fx=f['effect'];assert fx['load_error'] is None and fx['omitted']==0
            for v in fx['effects']:
                assert v['graphics']==754 and v['target'][0]==3
                if v['drawn']:drawn.setdefault(v['sequence'],set()).add(v['source_frame'])
                if v['finished']:assert v['source_frame']==0 and v['subframe']==20 and not v['drawn'];finished.add(v['sequence'])
            for e in fx['entered_events']:
                if e['sound']==102:assert e['frame']==1;sounds[e['sequence']]=sounds.get(e['sequence'],0)+1
            prepared.extend(c for c in f['audio'] if c['event']['sound']==102)
        assert set(drawn)==set(cues)==finished and all({2,20}<=v for v in drawn.values())
        assert sounds=={seq:1 for seq in cues}
        assert len(prepared)==sum(outcomes) and all(c['prepared'] and c['bytes']==original['sound']['bytes'] and c['sha256'].upper()==original['sound']['sha256'] for c in prepared)
        assert any(source['name']=='754-0.spr' and source['sha256'].upper()==original['sprite']['sha256'] for f in probe['frames'] for source in (f['effect']['sources'] or []))
if profile=='turn-undead-refused':
    caster=r['clients']['a']['action_probe'];assert caster['endpoint']['skill_feedback']=='NoResponse'
    for action in caster['actions']:
        before=action['before']['world'];after=action['after']['world']
        assert 18 in before['learned_skills']
        assert before['hp']==after['hp'] and before['mp']==after['mp']==[2,2]
        assert before['actors']['1']['action_sequence']==after['actors']['1']['action_sequence']
        assert after['last_skill_rejection']==before['last_skill_rejection']
    for role,_ in roles:
        probe=r['clients'][role]['action_probe'];w=probe['endpoint']['world']
        assert not w['effects'] and w['own_cast_packets']==0 and not w['actors'][str(w['status']['object_id'])]['dead']
        assert all(f['effect']['started']==0 and f['effect']['active']==0 and not f['effect']['effects'] and not f['effect']['entered_events'] for f in probe['frames'])
    lines=(p.parent/'server.log').read_text(encoding='utf-8-sig').splitlines()
    assert not any('C_USE_SPELL Turn Undead caster=' in line for line in lines), 'MP refusal committed TU'
if profile=='ranged': assert r['server']['remaining_arrows']==20-r['clients']['a']['action_probe']['attacks_sent']
if profile in ['heal','item-heal','chill-touch','vampiric-touch']:
    assert r['clients']['a']['action_probe']['attacks_sent']>0 and actions[0]['before']['world']['hp'][0]<100
elif profile!='turn-undead-player' and not profile.endswith('refused') and not profile.startswith('holy-weapon') and not (profile.startswith('chill-touch') or profile.startswith('vampiric-touch')):
    b=r['clients']['b']['action_probe']
    assert any(s['hp'][0]<100 for s in b['observed_health'])
    assert any(s['object_id']==3 and s['hp']<100 and s['phase']=='BATTLE' for s in trace)
if profile=='lightning-multi':
    assert any(row['object_id']==6 and row['phase']=='BATTLE' and row['hp']<r['clients']['c']['start']['world']['hp'][1] for row in trace)
    assert r['clients']['c']['action_probe']['moves_sent']>0
report={'status':'verified_real_peer_action_authority','action_profile':profile,'jar_sha256':r['jar_sha256'],
    'exe_sha256':r['exe_sha256'],'receipt_sha256':sha(p),'checks':checks,'initial_fixture':r['server']['initial_fixture'],
    'runtime_authority_mutations':r['server']['authority_mutations'],'gui_verified':False,'full_gameplay_verified':False,
    'visual_roles_verified':['a','b','c'] if profile=='lightning-multi' else ['a','b'] if profile in ['wind-cutter','ice-arrow','holy-weapon','holy-weapon-missing','chill-touch','chill-touch-full','vampiric-touch','vampiric-touch-full','stalac','lightning'] else ([active] if profile in ['bolt','fire-arrow'] else []),
    'wind_secondary_hit_presentation_only_verified':profile=='wind-cutter',
    'ice_secondary_hit_presentation_only_verified':profile=='ice-arrow'}
if (profile.startswith('chill-touch') or profile.startswith('vampiric-touch')) and not profile.endswith('refused'): report['life_drain_commits']=commits
(p.parent/'authority-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified real-peer action:',profile)
