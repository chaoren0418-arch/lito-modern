"""Independent formal/owned-client comparison for actual Quest Turn Undead."""
import json,sys,hashlib,re
from pathlib import Path
from PIL import Image
def load(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
p=Path(sys.argv[1]).resolve();r=load(p);assert r['action_profile']=='turn-undead-quest'
trace=[json.loads(v)['state'] for v in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
for role,id in [('a',1),('b',3)]:
 c=r['clients'][role];probe=c['turn_undead_probe'];w=probe['endpoint']['world'];assert c['worker_quiescent'] and c['shutdown']=='closed' and int(c['transport']['epoch'])==2
 assert c['start']['world']['status']['object_id']==w['status']['object_id']==id
 assert c['start']['world']['hp'][1]==32767 and c['start']['world']['map_id']==554
 assert any(s['object_id']==id and s['phase']=='BATTLE' and s['hp']==w['hp'][0] and s['mp']==w['mp'][0] and [s['x'],s['y']]==[w['actors'][str(id)]['x'],w['actors'][str(id)]['y']] for s in trace)
 for im in [c['start']['render'],c['render']]+probe['images']:
  with Image.open(im['png']) as pixels:assert hashlib.sha256(pixels.convert('RGBA').tobytes()).hexdigest().upper()==im['frame']['rgba_sha256'].upper()
assert 'both requested HP60000' in r['server']['initial_fixture']
a=r['clients']['a']['turn_undead_probe'];target=a['success_target'];assert target is not None
success=[v for v in a['actions'] if v['skill']==18 and v['success']];assert len(success)==1 and success[0]['target']==target
assert all(v['no_prediction'] for v in a['actions']);assert any(v['skill']==15 for v in a['actions'])
line=[v for v in (p.parent/'server.log').read_text(encoding='utf-8-sig').splitlines() if 'C_USE_SPELL Turn Undead caster=' in v and f' target={target} ' in v and ' success=true ' in v]
assert len(line)==1 and ' npc=45161 ' in line[0] and ' undead=1 ' in line[0] and ' isTU=true ' in line[0]
prob=int(re.search(r' probability=(\d+)',line[0])[1]);roll=int(re.search(r' roll=(\d+)',line[0])[1]);assert 1<=roll<=prob<=90
eligible=[n for row in trace for n in row['quest_monsters'] if n['id']==target and n['npc']==45161 and n['IsTU'] and n['undead']==1]
assert any(n['alive'] and n['hp']>0 and n['room_number']==4 for n in eligible)
corpses=[n for row in trace for n in row['quest_corpses'] if n['id']==target and n['npc']==45161 and n['IsTU'] and n['undead']==1 and n['room_number']==4]
assert any(not n['alive'] and n['hp']==0 for n in corpses), 'formal stored zero-HP corpse snapshot missing'
natural_failures=[v for v in a['actions'] if v['skill']==18 and v['failed'] and v['before']['target']['graphics']==145]
verified_natural_failures=[]
for failed in natural_failures:
    old=failed['before'];after=failed['after'];npc=after['target']
    assert failed['target']==old['target']['id']==npc['id'] and old['target']['incarnation']==npc['incarnation']
    assert not old['target']['dead'] and not npc['dead'] and old['target']['hp_ratio']==npc['hp_ratio']==100
    assert old['mp'][0]-after['mp'][0]==20 and after['casts']==old['casts']+1
    rows=[line for line in (p.parent/'server.log').read_text(encoding='utf-8-sig').splitlines() if 'C_USE_SPELL Turn Undead caster=' in line and f" target={failed['target']} " in line and ' success=false ' in line]
    row=next(line for line in rows if f" mp={after['mp'][0]}/" in line)
    fail_p=int(re.search(r' probability=(\d+)',row)[1]);fail_r=int(re.search(r' roll=(\d+)',row)[1]);assert 0<fail_p<=90 and fail_p<fail_r<=100 and ' isTU=true ' in row and ' npc=45161 ' in row
    assert any(s['object_id']==1 and s['mp']==after['mp'][0] and any(n['id']==failed['target'] and n['hp']==n['max_hp']==120 and n['alive'] for n in s['quest_monsters']) for s in trace)
    frames=[f for f in a['frames'] if f['mp'][0]==after['mp'][0]];assert frames
    assert all(not any(e['graphics']==754 for e in f['effect']['effects']) and not any(e['sound']==102 for e in f['effect']['entered_events']) for f in frames)
    assert r['clients']['a']['turn_undead_probe']['endpoint']['world']['last_skill_rejection'][1]==280
    verified_natural_failures.append({'target':failed['target'],'probability':fail_p,'roll':fail_r,'MP_cost':20,'NPC_HP_unchanged':120,'no754_or102_during_failed_cast':True,'message280':True})
frozen_probe=p.parent/'source-snapshot/src/modern_lito/native_turn_undead_probe.rs'
probe_pins=load(p.parent/'source-pins.json')
assert sha(frozen_probe)==probe_pins['src/modern_lito/native_turn_undead_probe.rs']
boss_requested='self.actions.iter().any(|a|a["skill"]==18&&a["before"]["target"]["graphics"]==240)' in frozen_probe.read_text(encoding='utf-8-sig')
boss_failures=[]
if boss_requested:
    boss_failures=[v for v in a['actions'] if v['skill']==18 and v['failed'] and v['before']['target']['graphics']==240]
    assert len(boss_failures)==1
    boss=boss_failures[0];assert boss['after']['target'] and not boss['after']['target']['dead'] and boss['after']['target']['hp_ratio']==boss['before']['target']['hp_ratio']==100
    assert boss['before']['mp'][0]-boss['after']['mp'][0]==20 and boss['after']['casts']==boss['before']['casts']+1
    boss_lines=[line for line in (p.parent/'server.log').read_text(encoding='utf-8-sig').splitlines() if 'C_USE_SPELL Turn Undead caster=' in line and f" target={boss['target']} " in line]
    assert len(boss_lines)==1 and ' npc=45601 ' in boss_lines[0] and ' isTU=false ' in boss_lines[0] and ' probability=0 ' in boss_lines[0] and ' success=false ' in boss_lines[0]
    assert any(n['id']==boss['target'] and n['npc']==45601 and n['hp']==n['max_hp']==7500 and not n['IsTU'] for row in trace for n in row['quest_monsters'])
else:
    assert not any(v["skill"]==18 and v["before"]["target"]["graphics"]==240 for v in a["actions"])
drawn=set();audio=0;done=False
for f in a['frames']:
 fx=f['effect']
 for v in fx['effects']:
  if v.get('graphics')!=754:continue
  assert v['target'][0]==target and v['sprite']=='754-0.spr'
  if v['drawn']:drawn.add(v['source_frame'])
  if v['finished']:assert v['source_frame']==0 and v['subframe']==20 and not v['drawn'];done=True
 for event in fx['entered_events']:
  if event['sound']==102:assert event['frame']==1;audio+=1
assert {2,20}<=drawn and done and audio==1
prepared=[cue for f in a['frames'] for cue in f['audio'] if cue['event']['sound']==102];assert len(prepared)==1 and prepared[0]['prepared']
original=load(p.parent/'source-snapshot/turn-undead-original-sources.json');assert (prepared[0]['bytes'],prepared[0]['sha256'].upper())==(original['sound']['bytes'],original['sound']['sha256'])
assert any(source['name']=='754-0.spr' and source['bytes']==original['sprite']['bytes'] and source['sha256'].upper()==original['sprite']['sha256'] for f in a['frames'] for source in (f['effect']['sources'] or []))
assert any(n['npc']==45601 for row in trace for n in row['quest_monsters']), 'natural summoning boss missing'
assert r['server']['authority_mutations']==0 and not r['server']['sql_access'] and r['server']['memberships_empty']
report={'status':'verified_real_quest_turn_undead_authority_and_original_effect','exe_sha256':r['exe_sha256'],'jar_sha256':r['jar_sha256'],'receipt_sha256':sha(p),'target':target,'npc':45161,'probability':prob,'roll':roll,'natural_boss_summon_and_quest_progression':True,'natural_probability_failures':verified_natural_failures,'ineligible_boss_rejected280_without_damage':bool(boss_failures),'boss_eligibility_requested_by_frozen_probe':boss_requested,'validator_sha256':sha(Path(__file__)),'initial_fixture':r['server']['initial_fixture'],'requested_initial_HP':60000,'observed_session_HP_cap':32767,'actor_HP_MP_matched_passive_formal_trace':True,'effect_first_last_positive_and_zero_terminal_verified':True,'effect_sound102_once_prepared':True,'gui_verified':False,'full_quest_presentation_verified':False,'goal_complete':False}
(p.parent/(sys.argv[2] if len(sys.argv)>2 else 'authority-comparison.json')).write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print('Verified actual Quest NPC Turn Undead:',target)
