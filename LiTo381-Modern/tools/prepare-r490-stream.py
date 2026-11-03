from pathlib import Path
import json
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003'
def full(identifier,mode):return [206,0,*identifier.to_bytes(4,'little'),65,0,0,0,0,0,0,0,mode,0xff,0xff,0,0,0,0,0,0,0,0,0,0,0]
def sub0(identifier,mode):return [213,0,*identifier.to_bytes(4,'little'),66,0,mode,0xff,0xff,0,0,0,0,0,0]
def delta(identifier,mode):return [213,2,*identifier.to_bytes(4,'little'),mode]
assert len(full(17,3))==28 and len(sub0(17,3))==17
packets=dict(first206=full(17,3),delta4=delta(17,4),wrong_sub0=sub0(18,3),matching_sub0=sub0(17,3),reentry206=full(17,3),replacement206=full(23,3),stale_delta=delta(17,4),unsupported_phase209=[209,65,0,1],inspection206=full(44,5),malformed_replacement=full(23,4)[:-1],malformed_delta=delta(23,4)+[0])
out=dict(state='R490_FIXED_CONTINUOUS_PACKET_STREAM_NO_INPUT_PHASE_OR_DISPATCH',packets=packets,expected_source_transitions=[dict(packet='first206',phase_before=0,phase_after=2,identifier=17,mode=3,clear=True),dict(packet='delta4',phase_before=2,phase_after=2,identifier=17,mode=4,clear=False),dict(packet='matching_sub0',phase_before=2,phase_after=2,identifier=17,mode=3,clear=False),dict(packet='reentry206',phase_before=2,phase_after=2,identifier=17,mode=3,clear=True),dict(packet='replacement206',phase_before=2,phase_after=2,identifier=23,mode=3,clear=True)],input_context_fields=[],policy='Production Runtime owns persistent zero BSS context and206 phase2 transitions. No per-packet phase/dispatch supplied. Existing inspection API separation has one explicit inspection call;it cannot alter persistent source state.',GUI=0,LOADED=False)
(L/'r490-stream-input.json').write_text(json.dumps(out,indent=2),encoding='utf-8');print('R490 continuous input prepared;5expected applied transfers;phase/dispatch absent from packet inputs')
