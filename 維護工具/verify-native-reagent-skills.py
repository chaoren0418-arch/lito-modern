"""Audit bounded modern Life Stream presentation and formal reagent authority."""
import datetime, hashlib, json, struct, wave
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-reagent-skills'
SOURCE=Path(r'D:\lineage開發\0-客戶端')
def read(p):return json.loads((ROOT/p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def member(name):
 b=(SOURCE/'Sprite.idx').read_bytes();assert len(b)==4+struct.unpack_from('<I',b)[0]*28
 rows=[]
 for at in range(4,len(b),28):
  if b[at+4:at+24].split(b'\0',1)[0].lower()==name.encode('ascii'):
   rows.append((struct.unpack_from('<I',b,at)[0],struct.unpack_from('<I',b,at+24)[0]))
 assert len(rows)==1
 offset,size=rows[0];assert 0<size<=2*1024*1024
 with (SOURCE/'Sprite.pak').open('rb') as f:f.seek(offset);raw=f.read(size)
 assert len(raw)==size
 return {'name':name,'bytes':size,'sha256':hashlib.sha256(raw).hexdigest().upper()}
def audit(rows,live=False):
 seen={};finished=set();events=set();pngs=[];terminal=0
 for frame in rows:
  fx=frame['reagent_effects'] if live else frame
  assert fx['presentation']=='modern_explicit_action3_once' and fx['load_error'] is None and fx['omitted']==0
  for e in fx['effects']:
   tick=e['elapsed_ns']*24//1_000_000_000;source=tick//4+1 if tick<72 else 0
   assert e['graphics']==2231 and e['action']==3 and e['sprite']=='2231-0.spr' and e['target'] is None
   assert e['source_frame']==source and e['subframe']==(source-1 if source else 18)
   assert e['finished']==(tick>=76) and e['drawn']==(tick<76)
   assert seen.setdefault(e['sequence'],e['position'])==e['position']
   if e['finished']:finished.add(e['sequence'])
   if source==0 and e['drawn']:terminal+=1
  for e in fx['entered_events']:
   assert e['action']==3 and e['frame']==1 and e['sound']==841 and e['event']=='EffectSound(841)'
   assert e['sequence'] not in events;events.add(e['sequence'])
  if frame.get('png'):
   p=Path(frame['png']).resolve();p.relative_to(ROOT)
   with Image.open(p) as im:
    assert im.mode=='RGBA' and im.size==(800,600)
    assert hashlib.sha256(im.tobytes()).hexdigest().upper()==frame['rgba_sha256'].upper()
   pngs.append({'path':p.relative_to(ROOT).as_posix(),'sha256':sha(p)})
 last=rows[-1]['reagent_effects'] if live else rows[-1]
 assert last['active']==0 and last['started']==len(seen)==len(finished)==len(events)
 return {'frames':len(rows),'pngs':pngs,'finished':len(finished),'terminal_samples':terminal,'sounds_once':len(events)}
def main():
 receipt=read('驗證紀錄/current/native-reagent-skills/receipt.json');ref=read('驗證紀錄/current/reference-snapshot.json')
 assert receipt['exe_sha256']==sha(ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe')
 assert receipt['jar_sha256']==ref['sha256']==sha(Path(ref['source']))==sha(Path(ref['snapshot']))
 offline=read('LiTo381-Modern/target/current/verification/native-reagent-effects.json');sample=audit(offline)
 assert sample['frames']==21 and len(sample['pngs'])==4 and sample['finished']==1 and sample['terminal_samples']==1
 sprite=member('2231-0.spr')
 assert [{**s,'sha256':s['sha256'].upper()} for s in offline[0]['sources']]==[sprite]
 wav=SOURCE/'Sound/841.wav';sound={'id':841,'bytes':wav.stat().st_size,'sha256':sha(wav)};assert sound['bytes']<=1<<20
 with wave.open(str(wav),'rb') as w:sound.update(channels=w.getnchannels(),sample_rate=w.getframerate(),frames=w.getnframes(),sample_width=w.getsampwidth())
 offline_audio=read('LiTo381-Modern/target/current/verification/native-reagent-audio.json');assert len(offline_audio)==1
 assert offline_audio[0]['bytes']==sound['bytes'] and offline_audio[0]['sha256'].upper()==sound['sha256']
 cases=[]
 for case in receipt['cases']:
  c,s=case['client'],case['server'];probe=c['combat'];success=case['mode']=='room-world-stream';accepted=2 if success else 0
  assert case['verified'] and c['worker_quiescent'] and c['inventory_shutdown_cleared'] and s['actual_client_sessions'] and not s['sql_access'] and not s['state_persistent']
  assert {str(i['id']):i for i in s['inventory']}==probe['result']['inventory']['items']
  result=audit(probe['frames'],True);assert result['finished']==accepted
  sounds=[a for f in probe['frames'] for a in f['audio'] if a['event']['sound']==841];assert len(sounds)==accepted
  for a in sounds:assert a['prepared'] and not a.get('same_id_suppressed',False) and a['bytes']==sound['bytes'] and a['sha256'].upper()==sound['sha256']
  for f in probe['frames']:
   sources=f['reagent_effects']['sources']
   if sources:assert [{**s,'sha256':s['sha256'].upper()} for s in sources]==[sprite]
  result.update(mode=case['mode'],attempts=len(probe['reagent_actions']),accepted=accepted);cases.append(result)
 assert len(cases)==3
 native=read('驗證紀錄/current/native-reagent-skills/frame-selection-raw.json');assert sha(Path(native['native_image']['path']))==native['native_image']['sha256']
 result={'schema':1,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exe_sha256':receipt['exe_sha256'],'jar_sha256':receipt['jar_sha256'],'cases':cases,'frames':sum(c['frames'] for c in cases),'pngs':sum(len(c['pngs']) for c in cases),'offline_samples':sample,'sprite':sprite,'sound':sound,'source_bytes_independently_verified':True,'new_independent_pixel_comparisons':0,'presentation':'modern_explicit_action3_once','legacy_action0_equivalent':False,'server_duration_is_animation_duration':False,'gui_tested':False,'audio_device_tested':False,'goal_complete':False}
 (OUT/'reagent-visual-receipt.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 print(json.dumps({k:result[k] for k in ['frames','pngs','source_bytes_independently_verified']}))
if __name__=='__main__':main()
