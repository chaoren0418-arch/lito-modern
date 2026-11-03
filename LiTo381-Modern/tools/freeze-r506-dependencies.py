from r491_common import *
import shutil
root=P/'target/r506-dependency-snapshot';root.mkdir(exist_ok=False);external=P.parent.parent/'lito脫殼資源';pins={};changes=[]
for name in ['priority-api-resolution.json','modern-F4-contract.json','reconnect-host-audit.json','host-alias-lifecycle.json','api-vectors.json']:
 src=external/name;dst=root/name;before=pin(src);shutil.copyfile(src,dst);q=pin(dst);assert (q['bytes'],q['sha256'])==(before['bytes'],before['sha256']);assert pin(src)==before;pins[name]=q
 changes.append(dict(name=name,upstream_path=str(src),observed_sha256=before['sha256'],snapshot=q))
api=read(L/'r506-API-adoption.json');oldAPI={k:api[k]['sha256']for k in ['API_resolution','F4_contract','host_audit']}
for k,n in [('API_resolution','priority-api-resolution.json'),('F4_contract','modern-F4-contract.json'),('host_audit','reconnect-host-audit.json')]:api[k]=pins[n]
api['dependency_snapshot']=dict(reason='upstream F4/host contract files changed after initial read; exact adopted runtime branch/API facts remain independently fixedPE-proven; current document bytes frozen; original observations retained as metadata',original_observed_sha256=oldAPI,no_upstream_latest_runtime_execution_claim=True)
save(L/'r506-API-adoption.json',api)
host=read(L/'r506-host-alias-adoption.json');host['original_observed_sha256']=dict(host_contract=host['host_contract']['sha256'],conversion_vectors=host['conversion_vectors']['sha256']);host['host_contract']=pins['host-alias-lifecycle.json'];host['conversion_vectors']=pins['api-vectors.json'];host['dependency_current_snapshot_reason']='upstream document changed after six fixedPE writes were independently verified; no additional host authority adopted'
save(L/'r506-host-alias-adoption.json',host)
study=read(L/'r506-transfer-source-study.json');study['API_adoption']=pin(L/'r506-API-adoption.json');study['host_alias_adoption']=pin(L/'r506-host-alias-adoption.json');study['second_row_available_subset']='110!=FFFFFFFF and (114==0 or nonnegative signed110 < nonnegative signed114); same-address branch only until source startup-host authority';save(L/'r506-transfer-source-study.json',study)
save(L/'r506-dependency-snapshot.json',dict(files=changes,original_API_observations=oldAPI,original_host_observations=host['original_observed_sha256'],no_runtime_or_DLL_historical_authority_expansion=True,GUI=0))
print('Five upstream API/F4/host/vector documents frozen in Modern owned directory; changed inputs recorded explicitly')
