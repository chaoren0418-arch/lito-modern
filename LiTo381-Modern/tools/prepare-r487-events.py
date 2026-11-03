from pathlib import Path
import json
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r487-menu-source-study';cases=[]
def case(name,top,steps):
 events=[]
 for kind,t,hit,et,pressed,due,changed in steps:
  events.append(dict(kind=kind,time_ms=t,hit=hit,expected=dict(top=et,selected=None,pressed=pressed,repeat_due=due,changed=changed)))
 cases.append(dict(name=name,filter='DiagnosticAll',count=40,initial_top=top,initial_selected=None,events=events))
case('click_before_hold_delay',0,[('down',100,'Down',0,'Down',800,False),('tick',799,None,0,'Down',800,False),('release',799,'Down',1,None,None,True),('tick',1000,None,1,None,None,False)])
case('held_down_700_125_no_catchup_release_once',0,[('down',0,'Down',0,'Down',700,False),('tick',699,None,0,'Down',700,False),('tick',700,None,1,'Down',825,True),('tick',824,None,1,'Down',825,False),('tick',825,None,2,'Down',950,True),('tick',5000,None,3,'Down',5125,True),('release',5000,'Down',4,None,None,True),('tick',5125,None,4,None,None,False)])
case('up_leave_reenter_rearms700',3,[('down',100,'Up',3,'Up',800,False),('tick',800,None,2,'Up',925,True),('move',900,None,2,None,None,False),('tick',925,None,2,None,None,False),('move',1000,'Up',2,'Up',1700,False),('tick',1699,None,2,'Up',1700,False),('tick',1700,None,1,'Up',1825,True),('release',1701,'Up',0,None,None,True)])
case('same_arrow_move_keeps_existing_deadline',2,[('down',10,'Down',2,'Down',710,False),('move',500,'Down',2,'Down',710,False),('tick',710,None,3,'Down',835,True),('release',720,'Down',4,None,None,True)])
case('other_arrow_release_cancels',2,[('down',10,'Down',2,'Down',710,False),('move',20,'Up',2,None,None,False),('release',40,'Up',2,None,None,False),('tick',1000,None,2,None,None,False)])
case('focus_capture_loss_cancel_has_no_late_callback',2,[('down',10,'Down',2,'Down',710,False),('cancel',30,None,2,None,None,False),('tick',1000,None,2,None,None,False),('release',1000,'Down',2,None,None,False)])
case('hold_reaches_last_row_without_overrun',38,[('down',0,'Down',38,'Down',700,False),('tick',700,None,39,'Down',825,True),('tick',825,None,39,'Down',950,False),('release',900,'Down',39,None,None,False)])
case('hold_reaches_first_row_without_underflow',1,[('down',0,'Up',1,'Up',700,False),('tick',700,None,0,'Up',825,True),('tick',825,None,0,'Up',950,False),('release',900,'Up',0,None,None,False)])
case('existing_disabled_upper_boundary_policy',0,[('down',0,'Up',0,None,None,False),('tick',700,None,0,None,None,False),('release',700,'Up',0,None,None,False)])
case('existing_disabled_lower_boundary_policy',39,[('down',0,'Down',39,None,None,False),('tick',700,None,39,None,None,False),('release',700,'Down',39,None,None,False)])
case('thumb_still_display_only_does_not_arm',3,[('down',0,'Thumb',3,None,None,False),('move',100,'Thumb',3,None,None,False),('tick',700,None,3,None,None,False),('release',700,'Thumb',3,None,None,False)])
case('modern_u64_deadline_overflow_cancels_repeat',0,[('down',(1<<64)-700,'Down',0,'Down',None,False),('tick',(1<<64)-1,None,0,'Down',None,False),('release',(1<<64)-1,'Down',1,None,None,True)])
v=dict(state='R487_FIXED_SOURCE_DERIVED_ARROW_EVENT_EXPECTATIONS',authority='414BD0 arms700;41AA00 invokes once and schedules125 from current time;414C00 cancels timer then invokes once on valid release;477930/477990 ->474350/474360 +/-1 clamped to0..count-1.',cases=cases,events=sum(len(c['events'])for c in cases),independent_expectations='Explicit fixed table prepared separately from Rust implementation; native numeric scheduling arithmetic and callback chain checked against pinned complete ASM. No original BIN execution/emulator or wall-clock UI equivalence claim.',Modern_adaptations=['u64 monotonic clock and checked deadline overflow','Existing disabled boundary policy retained','Focus/capture-loss/Back/mode/close cancellation are Modern lifecycle safety policy','Existing thumb is display-only pending exact shared-scroll geometry lifecycle'],GUI=0,LOADED=False)
p=L/'r487-event-input.json';assert not p.exists();p.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8');print('R487 fixed events:',len(cases),'cases',v['events'],'events')
