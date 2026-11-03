"""Audit original location-effect timing separately from formal gameplay leases."""
import argparse,datetime,hashlib,json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parent.parent
parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,default=ROOT/'驗證紀錄/current/native-ground-skills')
OUT=parser.parse_args().output.resolve();assert OUT.is_relative_to(ROOT.resolve());OUT.mkdir(parents=True,exist_ok=True)
def read(p):return json.loads((ROOT/p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def sample(ns):
    tick=ns*24//1_000_000_000;end=0
    for source,duration in zip([1,2,3,4,5,6,7,8,9,10,0],[1,1,2,3,3,3,3,3,3,2,2]):
        end+=duration
        if tick<end:return source,False
    return 0,True
def audit(frames,count):
    events=set();audio=set();effects={};finished=set();sound_rows=[];pngs=[];terminal=0;actor_events=0
    for f in frames:
        fx=f['ground_effects'];assert fx['load_error'] is None and fx['omitted']==0 and f['skill_effects']['omitted']==0
        entries=[e for a in f['actors'] for e in a.get('entered_events',[])]+fx['entered_events']+f['skill_effects']['entered_events']+f['spell_projectiles']['entered_events']
        for e in entries:
            key=json.dumps(e,sort_keys=True);assert key not in events;events.add(key)
            if e['event']=='GroundSound(111)':assert e['actor']==0 and e['sound']==111 and e['frame']==1
            else:actor_events+=1
        for e in f['audio']:
            key=json.dumps(e['event'],sort_keys=True);assert e['prepared'] and key in events and key not in audio;audio.add(key)
            if e['event']['event']=='GroundSound(111)':sound_rows.append(e)
        for e in fx['effects']:
            assert 'error' not in e and e['graphics']==168 and e['sprite']=='168-0.spr'
            source,done=sample(e['elapsed_ns']);assert e['source_frame']==source and e['finished']==done and e['drawn']==(not done)
            assert e['subframe']==(10 if source==0 else source-1)
            key=e['sequence'];v={'start_ns':f['elapsed_ns']-e['elapsed_ns'],'position':e['position']}
            assert effects.setdefault(key,v)==v
            if e['drawn'] and source==0:terminal+=1
            if done:finished.add(key)
        if f.get('png'):
            p=ROOT/'LiTo381-Modern'/f['png']
            with Image.open(p) as im:
                assert im.size==(800,600) and im.mode=='RGBA'
                assert hashlib.sha256(im.tobytes()).hexdigest()==f['rgba_sha256']
            pngs.append({'path':p.relative_to(ROOT).as_posix(),'sha256':sha(p)})
    assert len(effects)==len(finished)==len(sound_rows)==count
    if count:
        assert sum(not x['same_id_suppressed'] for x in sound_rows)==1
        assert len({(x['bytes'],x['sha256']) for x in sound_rows})==1
        assert all(x['same_id_suppressed'] for x in sound_rows[1:])
    return {'frames':len(frames),'pngs':pngs,'effects':list(effects.values()),'effect_count':count,'frame_events':len(events),
        'terminal_samples':terminal,'sounds_prepared':len(sound_rows),'sounds_audible':sum(not x['same_id_suppressed'] for x in sound_rows),
        'actor_events':actor_events}
def main():
    receipt=read('驗證紀錄/current/native-combat/ground-skills-receipt.json')
    exe=ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe'
    ref=read('驗證紀錄/current/reference-snapshot.json')
    assert receipt['exe_sha256']==sha(exe) and receipt['jar_sha256']==ref['sha256']==sha(Path(ref['source']))
    cases=[]
    for c in receipt['cases']:
        probe=c['client']['combat'];w=probe['result']['world'];refused=c['mode'].endswith('refused')
        count=sum(e['graphics']==168 for e in w['effects']);assert count==0 if refused else 1<=count<=8
        row=audit(probe['frames'],count);row['mode']=c['mode'];row['mp']=w['mp'];row['bot_hp_ratio']=w['actors']['2']['hp_ratio']
        assert w['mp']==([2,2] if refused else [60,120]) and w['mp'][0]==c['server']['final_player_mp']
        assert w['hp']==[50,50] and w['own_attack_packets']==0 and w['own_cast_packets']==(0 if refused else 1)
        assert probe['result']['skill_feedback']==('NoResponse' if refused else 'CastAction')
        assert len(probe['skill_selection'])==1 and probe['skill_selection'][0]['skill']==58
        if not refused:
            first=probe['frames'][0]['world']['actors']['1'];choice=probe['skill_selection'][0]['ground']
            dx=(choice[0]>first['x'])-(choice[0]<first['x']);dy=(choice[1]>first['y'])-(choice[1]<first['y'])
            expected=[[first['x']+dx*n,first['y']+dy*n] for n in range(1,count+1)]
            assert [x['position'] for x in row['effects']]==expected
            damaged=[f['elapsed_ns'] for f in probe['frames'] if f['world']['actors']['2']['hp_ratio']<100]
            assert damaged and min(damaged)>=max(e['start_ns']+(26_000_000_000+23)//24 for e in row['effects'])
            row['first_damage_after_visual_end']=True
        cases.append(row)
    offline=audit(read('LiTo381-Modern/target/current/verification/native-wall.json'),8)
    assert offline['frames']==49 and len(offline['pngs'])==4 and offline['terminal_samples']>0
    oracle=read('LiTo381-Modern/tests/fixtures/native-ground-skills-current.json')
    assert oracle['jar_sha256'].upper()==ref['sha256'] and oracle['lease']=={'segments':8,'first_pulses':2,'late_pulses':2,'expires_ns':10_000_000_000}
    r={'schema':1,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exe_sha256':receipt['exe_sha256'],'jar_sha256':ref['sha256'],
        'cases':cases,'frames':sum(x['frames'] for x in cases),'pngs':sum(len(x['pngs']) for x in cases),'frame_events':sum(x['frame_events'] for x in cases),
        'offline_resource_samples':offline,'formal_lease_oracle':oracle['lease'],'gui_tested':False,'audio_device_tested':False,'complete_skill_system':False}
    (OUT/'ground-skill-visual-receipt.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:r[k] for k in ['frames','pngs','frame_events']}))
if __name__=='__main__':main()
