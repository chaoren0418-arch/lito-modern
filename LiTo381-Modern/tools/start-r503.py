from r491_common import *
task='R503: source-closed character submit -> channel -> lobby; distinct channel Back -> character and lobby Back -> channel/reentry. Same R502 Runtime/EntryCommand/Window/bounded IO. Modern-only writes, no Java/server/provider/Original execution. Only new-path tests; no R502 historical PASS rerun. Overall16/40, GUI0.'
progress('R503_source_navigation_in_progress',task)
v=read(P.parent/'監製進度.json');v['r502']['new_player_flow_review']='FINITE_ACCEPTED_21JSON1157pins745unique';v['r503']={'status':'in_progress','current_task':task,'next':'Extract fixed ASM/PE 492FA0,46B1A0,495280,46F010,4A76D0,497020,4A7850,46BA10 and table630B20; derive C_CA/C_CB/C_0F and actual reply gates.','GUI':0,'complete_modes':16,'total_modes':40};save(P.parent/'監製進度.json',v)
save(L/'r503-continuation.json',v['r503'])
asm=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm')
lines=asm.read_text(encoding='utf-8-sig').splitlines()
out=L/'r503-source-fragments';out.mkdir(exist_ok=True)
for start,end in [(228912,229080),(230010,230210),(313613,313840),(317005,317180),(349993,350390),(552014,552075)]:
 (out/f'asm-{start}-{end}.txt').write_text('\n'.join(f'{i+1}: {lines[i]}' for i in range(start-1,end))+'\n',encoding='utf-8')
print(task)
