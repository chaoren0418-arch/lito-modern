from r491_common import *
p=L/'r499-call-state-mapping.json';v=read(p)
for q in v['mapping']:
 if q['fragment']=='1/d ->4AE050 ->480560':
  assert 'PEinitial636584=3B6'in q['effect'];q['effect']=q['effect'].replace('PEinitial636584=3B6,5033C4=114F8','PEinitial636584=FFFFFFFF(-1),5033C4=114F8;58locale branchmayreplace636584')
v['initial_field_inspection']=pin(L/'r499-initial-field-inspection.json');v['metadata_erratum']='Firstmapping/verifier incorrectlycalled636584initial3B6. PinnedPEsectionmappingprovesFFFFFFFF(-1);3B6ispossible58localebrancheffect. Correctedmetadataonly;actualBackRuntime/consumerunchangedandnotrerun.';save(p,v)
print('R499 initialfieldmetadata corrected toactualPE FFFFFFFF;Runtime/consumer untouched')
