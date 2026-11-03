from r491_common import *
# Preserve the compiled report and original entry contract. Their inherited
# R517 prose still says CC insertion unavailable; correct only the final
# metadata, with no source/library/executable modification or rerun.
old=L/'r518-entry-source-contract.json';entry=read(old)
entry['inherited_compilation_contract']=pin(old)
entry['room_departure_candidate']='R517 D505 owner/CF behavior retained; R518 separately enables bounded CC0 catalog ingestion, no own exit qualification'
final=L/'r518-entry-source-contract-final.json';save(final,entry)
old_report=L/'r518-player-candidate.json';player=read(old_report);player['original_compilation_report']=pin(old_report);player['entry_source_contract']=pin(final)
player['metadata_correction']='Only inherited R517 prose corrected; actual compiled entry/source/library and 0 executions unchanged.'
save(L/'r518-player-candidate-final.json',player)
save(L/'r518-preparation-accounting.json',dict(network_attempts=1,failed_network_attempts=0,library_builds=1,consumer_builds=1,player_builds=1,product_changes_after_execution=0,metadata_correction=dict(original_contract=pin(old),final_contract=pin(final),original_report=pin(old_report),final_report=pin(L/'r518-player-candidate-final.json'),scope='Inherited R517 boundary prose only; original artifacts preserved, no executable/source/library change or rerun.'),routine_read_invocations='Initial cwd and stdin Python module path corrected before product/network work; no failed Java/SQL/runtime attempts.'))
print('Final entry metadata corrected; original compilation evidence preserved;0 code changes/reruns')
