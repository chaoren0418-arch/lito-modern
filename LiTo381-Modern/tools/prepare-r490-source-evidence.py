from pathlib import Path
import json,hashlib,re,datetime
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r490-state-flow-study';O.mkdir(exist_ok=False);A=O/'authority';A.mkdir()
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
prior=read(L/'r489-source-evidence.json');asm=Path(prior['original_disassembly']['path']);assert pin(asm)==prior['original_disassembly'];s=asm.read_text(encoding='utf-8');lines=s.splitlines();fragments=[]
for name in ['sub_4A8BA0','sub_4A9E50','sub_469CA0','sub_450230','sub_46B7D0','sub_474280','sub_4B4BB0','sub_4A9550','sub_4A9B90','sub_4B2970']:
 m=re.search('^'+name+r' proc near',s,re.M)
 if not m:continue
 end=re.search(r'^\w+ proc near',s[m.end():],re.M);p=A/(name+'-through-next-proc.asm');p.write_text(s[m.start():m.end()+end.start()],encoding='utf-8');fragments.append(pin(p))
for token in ['dword_64AF98','dword_64BBD4','dword_64A968']:
 i=next(i for i,l in enumerate(lines)if l.startswith(token+' '));p=A/(token+'-context.asm');p.write_text('\n'.join(lines[i:i+20])+'\n',encoding='utf-8');fragments.append(pin(p))
proc='';dispatch_writes=[]
for i,l in enumerate(lines):
 if re.match(r'^\w+ proc near',l):proc=l.split()[0]
 if l.startswith('mov     dword_64BBD4,'):dispatch_writes.append(dict(line=i+1,procedure=proc,text=l,context=lines[max(0,i-6):i+8]))
effects=[dict(order=1,site='4A8BDF',effect='450230 clears64A950 list ifnon-null'),dict(order=2,site='4A8BEE..4A8C2B',effect='4B4BB0 resizes64AB64..64AB68 vector usingbase asnewend;nullsub37 thenupdatesend;450230 clears64A94C'),dict(order=3,site='4A8C2B..4A8C49',effect='450230 clears64A968(maplist) then64A96C ifnon-null'),dict(order=4,site='4A8C49..4A8CEC',effect='4B4BB0 shrinks64AB54..64AB58 vector thenparserwrites fullknown fields includingidentifier andmodebyte'),dict(order=5,site='4A8CEC..4A8D30',effect='8extension bytes loop/nullsub36/selected-map byte64ACC9 transform'),dict(order=6,site='4A8D30..4A8D8F',effect='469CA0(2);optional64AFC7 callbackhooks then458830 init'),dict(order=7,site='4A8D8F..4A8FD9',effect='Additional encrypted/global/resource-dependent reporting branch;not represented byoffline Runtime')]
record=dict(state='R490_PERSISTENT_PHASE2_AND_NATIVE_MAP_LIST_CLEAR_ORDER_SOURCE',previous_R489_handoff=pin(L/'r489-native-state-snapshot-player-flow-handoff.json'),original_disassembly=prior['original_disassembly'],fragments=fragments,full206_effect_order=effects,map_alias='46B892 storesconstructedlist in64A968;474280 storeslist asmapcontrol+8;46B8C9 storescontrol64B094.206 clears64A968 through450230,whichdeletesitems thenzeroes+2Ccount/+28top;doesnotwrite+4Cselected-index.',initial_state='64AF98 dd0,64BBD4 dd0,64ABF4 dd0,word64ACCA dw0 frompinneddata. Default persistent subset startszero.',dispatch_named_writers=dispatch_writes,scope='Maplist alias/count/top clear andphase2 established. Otherlist/vector/init/effect/actor/nativecallback transitions listed,not guessed. No repeatedthumbgeometry search.',GUI=0,LOADED=False)
save(L/'r490-source-evidence.json',record)
pp=P.parent/'監製進度.json';v=read(pp);v['updated_at']=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat();v['phase']='R490_active_persistent_source_state_flow_and206_map_list_invalidation';v['current_task']='R490 active:persistent defaultBSS phase/dispatch state consuming source packets,206->phase2 andsource64A968map-list clear invalidatesselection/worker generation;213noimmediateclear. Separateexistinginspection APIs. Unknownphase6callbackarrival/dispatchproducers/actor sideeffects explicitlyunsupported. PreserveR489 andoldPASS/noGUI/noJava.';v['r489_T347_finite_acceptance']=dict(handoff=record['previous_R489_handoff'],accepted_scope='207newcases/9actualRuntime offline subset only;modec correction accepted',GUI=0,LOADED=False,complete_modes=16);save(pp,v);print('R490 source',len(fragments),'fragments;maplist alias clear/phase2 order proven;progress active')
