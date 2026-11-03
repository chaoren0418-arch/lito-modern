"""Audit authoritative item sessions, original SPR pins and class0 sample timing."""
import datetime, hashlib, importlib.util, json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / '驗證紀錄/current/native-item-actions'

def read(p):
    return json.loads((ROOT / p).read_text(encoding='utf-8-sig'))

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest().upper()

def png_pin(p, expected, size=(800,600)):
    p = Path(p).resolve(); p.relative_to(ROOT)
    with Image.open(p) as im:
        assert im.size == size and im.mode == 'RGBA'
        assert hashlib.sha256(im.tobytes()).hexdigest().upper() == expected.upper()
    return {'path':p.relative_to(ROOT).as_posix(), 'sha256':sha(p)}

def audit_effects(rows, live=False):
    seen, done, pngs = {}, set(), []
    terminal = 0
    for f in rows:
        fx = f['item_effects'] if live else f
        assert fx['load_error'] is None and fx['omitted'] == 0 and not fx['entered_events']
        for e in fx['effects']:
            assert e['graphics'] in [189,194,197] and 'error' not in e
            elapsed = e['elapsed_ns'] * 24 // 1_000_000_000
            end = 0; source = 0; finished = True
            for index, ticks in zip([1,2,3,4,5,6,7,8,9,0],[1,1,1,2,2,2,2,2,2,3]):
                end += ticks
                if elapsed < end:
                    source, finished = index, False; break
            assert e['source_frame'] == source and e['finished'] == finished and e['drawn'] == (not finished)
            assert e['subframe'] == [5,0,1,2,3,4,3,4,3,4][source]
            fixed = {k:e[k] for k in ['graphics','position','target']}
            assert seen.setdefault(e['sequence'],fixed) == fixed
            if finished: done.add(e['sequence'])
            elif source == 0: terminal += 1
        if f.get('png'): pngs.append(png_pin(f['png'],f['rgba_sha256']))
    last = rows[-1]['item_effects'] if live else rows[-1]
    assert last['active'] == 0 and len(seen) == len(done) == last['started']
    return {'frames':len(rows),'pngs':pngs,'terminal_samples':terminal,'effects':list(seen.values()),'finished':len(done)}

def main():
    receipt=read('驗證紀錄/current/native-item-actions/receipt.json')
    exe=ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe'
    ref=read('驗證紀錄/current/reference-snapshot.json')
    assert receipt['exe_sha256'] == sha(exe) and receipt['jar_sha256'] == ref['sha256'] == sha(Path(ref['source']))
    assert len(receipt['cases']) == 5
    offline=read('LiTo381-Modern/target/current/verification/native-item-effects.json')
    effect_samples=audit_effects(offline)
    assert effect_samples['frames'] == 13 and len(effect_samples['pngs']) == 4 and effect_samples['terminal_samples'] == 9
    assert offline[9]['rgba_sha256'] == offline[11]['rgba_sha256'] != offline[12]['rgba_sha256']
    ground=read('LiTo381-Modern/target/current/verification/native-ground-items.json')
    ground['png_pin']=png_pin(ground['png'],ground['rgba_sha256'],(240,160))
    spec=importlib.util.spec_from_file_location('icon_reference',ROOT/'維護工具/verify-native-inventory-icons.py')
    archive=importlib.util.module_from_spec(spec);spec.loader.exec_module(archive)
    source_pins=[]
    for s in offline[0]['sources'] + ground['sources']:
        name=s.get('name',s.get('member'));raw=archive.member('Sprite',name)
        assert raw is not None and len(raw)==s['bytes'] and hashlib.sha256(raw).hexdigest().upper()==s['sha256'].upper()
        source_pins.append({'name':name,'bytes':len(raw),'sha256':s['sha256'].upper()})
    cases=[]
    for case in receipt['cases']:
        c,s=case['client'],case['server'];mode=case['mode']
        assert case['verified'] and c['inventory_shutdown_cleared'] and c['worker_quiescent']
        assert s['actual_client_sessions'] and not s['sql_access'] and not s['state_persistent']
        if mode.startswith('inventory'):
            assert len(c['item_actions'])==(2 if mode.endswith('refused') else 4)
            cases.append({'mode':mode,'actions':len(c['item_actions']),'frames':0,'pngs':[]});continue
        probe=c['combat'];w=probe['result']['world'];items=probe['item_actions']
        assert len(items)==2 and w['hp'][0]==s['final_player_hp']==50
        assert {str(i['id']):i for i in s['inventory']} == probe['result']['inventory']['items']
        if mode.endswith('heal'):
            visual=audit_effects(probe['frames'],True)
            assert items[0]['hp_before'][0]<50 and visual['finished']==2
            assert all(e['graphics']==189 for e in visual['effects'])
            for f in probe['frames']:
                if f['item_effects']['sources'] is not None:assert f['item_effects']['sources']==offline[0]['sources']
        else:
            pngs=[png_pin(f['png'],f['rgba_sha256']) for f in probe['frames'] if f.get('png')]
            visual={'frames':len(probe['frames']),'pngs':pngs}
            refused=mode.endswith('refused');drops=items[-1]['ground']
            assert len(drops)==(0 if refused else 2)
            if not refused:
                assert len({a['id'] for a in drops})==2
                for a in drops:
                    assert a['graphics']==773 and a['experience']==2
                    rows=[r for r in probe['frames'][-1]['actors'] if r['id']==a['id']]
                    assert len(rows)==1 and rows[0]['rendered'] and rows[0]['sprite_member']=='773-0.spr' and rows[0]['phase']=='ground_item'
        visual.update(mode=mode,actions=2);cases.append(visual)
    native=read('驗證紀錄/current/native-item-actions/native-use-dispatch.json')
    assert sha(Path(native['native_image']['path']))==native['native_image']['sha256']
    result={'schema':1,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exe_sha256':receipt['exe_sha256'],'jar_sha256':receipt['jar_sha256'],
        'cases':cases,'frames':sum(c['frames'] for c in cases),'pngs':sum(len(c['pngs']) for c in cases),
        'effect_samples':effect_samples,'ground_samples':ground,'sources':source_pins,
        'new_independent_pixel_comparisons':0,'source_bytes_independently_verified':True,'gui_tested':False,'audio_device_tested':False,'complete_item_system':False}
    (OUT/'item-visual-receipt.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:result[k] for k in ['frames','pngs','source_bytes_independently_verified']}))

if __name__=='__main__':main()
