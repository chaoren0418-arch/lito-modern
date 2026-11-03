"""Audit completed formal class4 casts, arrival callbacks and original PNGs."""
import argparse, datetime, hashlib, json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parent.parent
OUT=ROOT/'驗證紀錄/current/native-attack-skills'
parser=argparse.ArgumentParser()
parser.add_argument('--output',type=Path,default=OUT)
OUT=parser.parse_args().output.resolve()
assert OUT.is_relative_to(ROOT.resolve()), 'output must remain in reconstruction root'
OUT.mkdir(parents=True,exist_ok=True)
def read(p):return json.loads((ROOT/p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def audit(frames,casts):
    events=set();audio=set();pngs=[];arrivals=[];launches={};graphics=set();sounds={}
    for f in frames:
        fx=f['spell_projectiles'];assert fx['load_error'] is None and fx['omitted']==0
        entries=fx['entered_events']+[e for a in f['actors'] for e in a.get('entered_events',[])]+f['skill_effects']['entered_events']
        for e in entries:
            key=json.dumps(e,sort_keys=True);assert key not in events,e;events.add(key)
        for e in f['audio']:
            key=json.dumps(e['event'],sort_keys=True);assert e['prepared'] and key in events and key not in audio
            audio.add(key);v={'id':e['event']['sound'],'bytes':e['bytes'],'sha256':e['sha256']};assert sounds.setdefault(v['id'],v)==v
        for flight in fx['flights']:
            assert 'error' not in flight,flight
            key=(flight['attacker'],flight['sequence'])
            start=f['elapsed_ns']-flight['elapsed_ns'];to=flight['to']
            if key not in launches:
                assert any(e['actor']==key[0] and e['sequence']==key[1] and e['event']=='Hit' for e in entries),'not launched by native Hit'
                launches[key]={'start_ns':start,'arrival_ns':flight['arrival_ns'],'to':to,'projectile':flight['projectile']}
            assert launches[key]=={'start_ns':start,'arrival_ns':flight['arrival_ns'],'to':to,'projectile':flight['projectile']}
            if flight['drawn']:
                graphics.add(flight['graphics'])
                if flight['graphics']==167:assert flight['elapsed_ns']<flight['arrival_ns'] and flight['sprite'] in [f'167-{i}.spr' for i in range(4)]
                else:assert flight['graphics']==219 and flight['elapsed_ns']>=flight['arrival_ns'] and flight['position']==to
        for c in fx['hurt_callbacks']:
            assert c['accepted'] and c['target']==2
            assert c['projectile'] not in [a['projectile'] for a in arrivals]
            launch=next(v for v in launches.values() if v['projectile']==c['projectile'])
            assert f['elapsed_ns']>=launch['start_ns']+launch['arrival_ns']
            arrivals.append(dict(c,elapsed_ns=f['elapsed_ns']))
        if any(a['id']==2 and a['phase']=='damage' for a in f['actors']):assert arrivals,'hurt before arrival'
        if f.get('png'):
            p=Path(f['png']).resolve();p.relative_to(ROOT)
            with Image.open(p) as im:
                assert im.size==(800,600);rgba=hashlib.sha256(im.convert('RGBA').tobytes()).hexdigest()
            assert rgba==f['rgba_sha256'];pngs.append({'path':str(p),'sha256':sha(p),'rgba_sha256':rgba})
    assert len(launches)==casts and len(arrivals)==casts
    assert sum(json.loads(e).get('event')=='SpellSound(97)' for e in events)==casts
    assert frames[-1]['spell_projectiles']['active']==0
    assert not casts or graphics=={167,219}
    return {'frames':len(frames),'pngs':pngs,'frame_events':len(events),'sounds':list(sounds.values()),'arrivals':arrivals,'launches':list(launches.values()),'events_once':True}
def main():
    r=read('驗證紀錄/current/native-combat/attack-skills-receipt.json');pin=read('驗證紀錄/current/reference-snapshot.json')
    assert r['exe_sha256']==sha(ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe') and r['jar_sha256']==pin['sha256']
    assert len(r['cases'])==2;cases=[]
    for case in r['cases']:
        c=case['client']['combat'];w=c['result']['world'];refused=case['mode']=='room-world-bolt-refused';casts=0 if refused else 2
        assert w['own_cast_packets']==casts and w['own_attack_packets']==0
        assert w['mp']==([2,2] if refused else [4,10]) and w['mp'][0]==case['server']['final_player_mp']
        assert c['result']['skill_feedback']==({'Rejected':278} if refused else 'CastAction')
        row=audit(c['frames'],casts);row.update(mode=case['mode'],mp=w['mp'],route_steps=len(c['route']),casts=casts,sources=c['frames'][-1]['spell_projectiles']['sources']);cases.append(row)
    vector=audit(read('LiTo381-Modern/target/current/verification/native-bolt.json'),1)
    d={'schema':1,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exe_sha256':r['exe_sha256'],'jar_sha256':r['jar_sha256'],
       'cases':cases,'frames':sum(c['frames'] for c in cases),'pngs':sum(len(c['pngs']) for c in cases),'frame_events':sum(c['frame_events'] for c in cases),
       'offline_resource_samples':vector,'gui_tested':False,'audio_device_tested':False,'complete_skill_system':False}
    (OUT/'attack-skill-visual-receipt.json').write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:d[k] for k in ['frames','pngs','frame_events']}))
if __name__=='__main__':main()
