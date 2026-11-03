"""Prepare a twelve-case reader accepting either qualified current-profile preflight."""
from pathlib import Path
import ast
T=Path(__file__).resolve().parent
s=(T/'qualify-native8CC5-audio-cohort.py').read_text('utf-8-sig').replace('native8CC5-audio-cohort','native33199-audio-cohort').replace('native8CC5-teleport-audio','native33199-teleport-audio')
needle="and key=='Release-refused'";assert s.count(needle)==1
s=s.replace(needle,"and key in ['Debug-refused','Release-refused']",1)
ast.parse(s);(T/'qualify-native33199-audio-cohort.py').write_text(s,encoding='utf-8',newline='\n')
print('Prepared only: exact-source twelve-case reader, 96 sessions, 24 retired cues, 44 actual C1F requests; parent terminal0 remains mandatory.')
