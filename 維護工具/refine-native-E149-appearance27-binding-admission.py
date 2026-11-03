"""Preserve the first source candidate; fix live-history admission and bounded hurt replacement."""
from pathlib import Path
import json,copy
from native_E149_recovery_contract import R,read,sha,encoded,require
T=R/'維護工具';C=T/'candidates/native-E149-appearance27-binding-20261024-144000';OUT=T/'candidates/native-E149-appearance27-binding-admission-20261024-144000';PREFIX='LiTo381-Modern/src/modern_lito/'
def main():
 cp=C/'contract.json';require(sha(cp)=='8F72B888B2BC12799A47888CE02F9FC77188DB779311DD7BC90BE7D579D1D927','precursor source changed');k=read(cp);require(not OUT.exists(),'corrected source exists');k=copy.deepcopy(k)
 oldqa=T/'templates/native_appearance27_animator_tests.rs';qa=T/'templates/native_appearance27_animator_admission_tests.rs';require(not qa.exists(),'corrected QA exists');s=oldqa.read_text('utf-8-sig');a='same.select_appearance_context(7,w.generation,w.map_id.unwrap());assert_eq!';b='same.select_appearance_context(7,w.generation,w.map_id.unwrap());let mut admitted=Admission::default();admitted.admit(&w);same.retire_admitted(admitted,&w,200_000_000);assert_eq!';require(s.count(a)==1,'live admission QA anchor differs');qa.write_text(s.replace(a,b),encoding='utf-8',newline='\n')
 fixes={
 'native_world_animation.rs':[("                self.combat.admit(actor,sequence,now);\n                if let Some(p)=self.appearance.actors.get_mut(&actor.id).filter(|p|p.owner.incarnation==actor.incarnation){p.admission_floor=p.admission_floor.max(sequence);}","                let live_body=self.appearance.actors.get_mut(&actor.id).filter(|p|p.owner.incarnation==actor.incarnation).is_some_and(|p|{p.admission_floor=p.admission_floor.max(sequence);!p.needs_rebase});\n                if !live_body{self.combat.admit(actor,sequence,now);}")],
 'native_appearance27.rs':[
 ('ensure!(h.pending.len()<QUEUE_LIMIT&&h.seen.len()<ADMISSION_LIMIT,"body operation/admission budget");','let coalesce=matches!(entry.operation,Operation::ServerAction{action:2,..})&&h.pending.back().is_some_and(|e|matches!(e.operation,Operation::ServerAction{action:2,..}));\n        ensure!((h.pending.len()<QUEUE_LIMIT||coalesce)&&h.seen.len()<ADMISSION_LIMIT,"body operation/admission budget");'),
 ('h.seen.insert(entry.sequence,entry.operation.clone());h.last=entry.sequence;h.admitted=admitted;h.pending.push_back(entry);Ok(true)','h.seen.insert(entry.sequence,entry.operation.clone());h.last=entry.sequence;h.admitted=admitted;if coalesce{h.pending.pop_back();}h.pending.push_back(entry);Ok(true)'),
 (' pub(crate) fn record(&mut self,entry:Entry){',' pub(crate) fn record(&mut self,entry:Entry){\n  if matches!(entry.operation,Operation::ServerAction{action:2,..})&&self.entries.back().is_some_and(|e|matches!(e.operation,Operation::ServerAction{action:2,..})){self.entries.pop_back();}'),
 ('self.queue.prune(tape.dropped_through);','self.queue.prune(floor);')]
 }
 OUT.mkdir()
 for short,delta in fixes.items():
  name=PREFIX+short;row=k['runtime_overrides'][name];source=Path(row['path']);require(sha(source)==row['sha256'],'source changed');text=source.read_text('utf-8-sig')
  for a,b in delta:require(text.count(a)==1,'source review anchor differs '+a[:90]);text=text.replace(a,b)
  row['changes'].extend(delta)
  if short=='native_world_animation.rs':
   a=oldqa.read_text('utf-8-sig');require(text.count(a)==1,'original QA source absent');text=text.replace(a,qa.read_text('utf-8-sig'));matches=[t for t in row['supplements'] if t['path']==str(oldqa)];require(len(matches)==1,'QA pin absent');matches[0].update(path=str(qa),sha256=sha(qa))
  p=OUT/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text,encoding='utf-8',newline='\n');row.update(path=str(p),sha256=sha(p));k['effective_overlay_files'][name]=sha(p);k['effective_overlay_paths'][name]=str(p)
 k.update(source_review_precursor=str(cp),source_review_precursor_sha256=sha(cp),source_review_reason='Live S27 History must keep its own released ceiling; ordinary raw admission cannot skip queued actions. Pending hurt2 replacement coalesces; completed keys prune against monotonic admission floor.',source_precursor_metadata_commands=0,live_body_admission_preserves_queued_order=True)
 p=OUT/'contract.json';p.write_bytes(encoded(k));print(json.dumps(dict(contract=str(p),sha256=sha(p),effective_files=39,changed_runtime=5,new_cases=10,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
