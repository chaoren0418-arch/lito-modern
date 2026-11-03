from pathlib import Path
import json
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';cases=[]
def full(identifier,mode,name=[],code=[],tag=[],extra=[]):
 return [206,0,*identifier.to_bytes(4,'little'),*name,0,0x91,*code,0,0x34,0x12,0xcd,0xab,mode,0xff,0xff,0xe1,0xd0,0xd3,0xd1,0xd2,*tag,0,0xd4,0xd5,0xd6,0xd7,len(extra),*extra]
def sub0(identifier,mode,name=[],tag=[]):return [213,0,*identifier.to_bytes(4,'little'),*name,0,mode,0x9c,0xff,0xc1,*tag,0,0xd0,0xd1,0xd2,0xd3]
def applied(previous,current,rebound,phase):return dict(kind='Applied',previous=previous,current=current,identifier_rebound=rebound,native_phase_after=phase)
def add(name,p,raw=0xab01,identifier=17,phase=6,dispatch=0,final_raw=None,final_id=None,result=None,decoded=None):
 expected=dict(raw_mode=raw if final_raw is None else final_raw,identifier=identifier if final_id is None else final_id,outcome=dict(kind='Rejected')if result is None else result)
 cases.append(dict(name=name,packet=p,initial=dict(raw_mode=raw,identifier=identifier),context=dict(phase=phase,dispatch_word=dispatch),expected=expected,decoded_expected=decoded))
def dec(kind,identifier,mode,strings,words,extra=None):return dict(kind=kind,identifier=identifier,mode_byte=mode,strings=strings,words=words,extension=extra)
f=full(0x12345678,3,[65],[0x81,0xff,0x40],[0x80,0xfe],[10,20,30]);mx=full(0x78563412,5,[0x81]*89,[0xff]*4,[0xfe]*27,list(range(8)));assert len(mx)==155
sm=sub0(0x78563412,4,[0x81]*89,[0xfe]*27);assert len(sm)==132
add('206 empty strings and unconditional identifier bind',full(0x12345678,1),raw=0,phase=5,final_raw=1,final_id=0x12345678,result=applied(0,1,True,2),decoded=dec('State206',0x12345678,1,[[],[],[]],[0x1234,0xabcd,0xffff],[]))
add('206 variable raw nonUTF8 strings mode is c',f,final_raw=0xab03,final_id=0x12345678,result=applied(0xab01,0xab03,True,2),decoded=dec('State206',0x12345678,3,[[65],[0x81,0xff,0x40],[0x80,0xfe]],[0x1234,0xabcd,0xffff],[10,20,30]))
add('206 exact max155 accepted strings89/4/27 extension8',mx,final_raw=0xab05,final_id=0x78563412,result=applied(0xab01,0xab05,True,2),decoded=dec('State206',0x78563412,5,[[0x81]*89,[0xff]*4,[0xfe]*27],[0x1234,0xabcd,0xffff],list(range(8))))
add('206 dispatch4 phase2 permits initialization',full(18,2),phase=2,dispatch=4,final_raw=0xab02,final_id=18,result=applied(0xab01,0xab02,True,2))
add('206 dispatch4 phase0 permits initialization',full(18,2),phase=0,dispatch=4,final_raw=0xab02,final_id=18,result=applied(0xab01,0xab02,True,2))
for phase in [3,6,0x7fffffff]:add('206 blocked dispatch4 phase'+str(phase),full(18,2),phase=phase,dispatch=4)
for phase in [0x80000000,0xffffffff]:add('206 signedJL permits negative raw phase'+str(phase),full(18,2),phase=phase,dispatch=4,final_raw=0xab02,final_id=18,result=applied(0xab01,0xab02,True,2))
add('213 subtype0 minimum16 matching identity',sub0(17,3),phase=2,final_raw=0xab03,result=applied(0xab01,0xab03,False,2),decoded=dec('State213',17,3,[[],[]],[0xff9c]))
add('213 subtype0 max132 phase6 binds new identity',sm,final_raw=0xab04,final_id=0x78563412,result=applied(0xab01,0xab04,True,6),decoded=dec('State213',0x78563412,4,[[0x81]*89,[0xfe]*27],[0xff9c]))
add('213 subtype0 non6 mismatch ignored',sub0(18,3),phase=2,result=dict(kind='IgnoredIdentifier'))
add('213 subtype0 dispatch4 still allowed',sub0(17,5),phase=3,dispatch=4,final_raw=0xab05,result=applied(0xab01,0xab05,False,3))
# Every proper prefix of the maximum fully populated known layout must refuse
# before changing mode or identifier, even with phase6/new identity.
for n in range(len(mx)):add('206 truncate max packet at '+str(n),mx[:n])
minimum=sub0(18,3)
for n in range(len(minimum)):add('213 truncate min packet at '+str(n),minimum[:n])
for name,p in [('206 name90',full(18,3,[65]*90)),('206 code5',full(18,3,[],[66]*5)),('206 tag28',full(18,3,[],[],[67]*28)),('213 name90',sub0(18,3,[65]*90)),('213 tag28',sub0(18,3,[],[67]*28))]:add('reject capacity overflow '+name,p)
for n in [9,127,128,255]:
 p=full(18,3);p[-1]=n;add('reject signed unsupported extension count '+str(n),p)
for n in [1,2,255]:
 for opcode in [206,213]:
  p=full(18,3)if opcode==206 else sub0(18,3);p[1]=n;add('reject subtype '+str(opcode)+'/'+str(n),p)
for opcode in [0,205,207,255]:p=full(18,3);p[0]=opcode;add('reject opcode '+str(opcode),p)
for opcode,p in [(206,full(18,3)),(213,sub0(18,3))]:add('reject trailing byte '+str(opcode),p+[1])
add('reject packet156 before parsing',mx+[0])
out=dict(state='R489_FIXED_SOURCE_LAYOUT_EXPECTATIONS_BEFORE_CONSUMER',cases=cases,case_count=len(cases),layout206='cdscshhchcccccsccccc',layout213='header cd +schcscccc',mode_width='Both snapshot mode fields c/low8,not full word;source correction toR488.',policy='Raw byte strings,NUL required within90/5/28 capacities;count0..8;complete packet consumed;atomic reject. Phase/dispatch context explicit. No production decoder called to generate expected values.',runtime_packets=dict(full_start=full(0x12345678,3,[65],[0x81,0xff],[0xfe],[10]),full_active=full(0x78563412,3,[66],[0x81],[0xff],[11,12]),subtype0_match=sub0(0x12345678,3,[0x80],[0xff]),subtype0_mismatch=sub0(18,2),malformed_late=mx[:-1],blocked_dispatch=full(18,2)),GUI=0,LOADED=False)
(L/'r489-event-input.json').write_text(json.dumps(out,indent=2),encoding='utf-8');print('R489 prepared',len(cases),'fixed new snapshot cases;max206155/max213132;raw strings/noUTF8 inference')
