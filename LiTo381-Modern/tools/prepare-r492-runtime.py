from pathlib import Path
P=Path(__file__).resolve().parent.parent
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8')
for a,b in [('pub struct Runtime {source_flow:SourceFlow,','pub struct Runtime {source_scope:SourceScope,source_flow:SourceFlow,'),('Ok(Self{source_flow:SourceFlow::default(),','Ok(Self{source_scope:SourceScope::default(),source_flow:SourceFlow::default(),')]:
 assert s.count(a)==1;s=s.replace(a,b)
a='self.next_generation()?;self.invalidate();self.shutdown.store(true,Ordering::Release);';b='self.next_generation()?;self.invalidate();if self.source_scope.bound(){self.source_scope.retire();self.source_flow=SourceFlow::default();}self.shutdown.store(true,Ordering::Release);';assert s.count(a)==1;s=s.replace(a,b);p.write_text(s,encoding='utf-8');print('R492 candidate source-scope field/initialization/Close retirement applied;no normalwire bridge asserted')
