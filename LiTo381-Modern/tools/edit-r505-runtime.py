from r491_common import *
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8')
s=s.replace('#[path="native_entry_navigation.rs"]pub mod entry_navigation;', '#[path="native_entry_navigation.rs"]pub mod entry_navigation;\n#[path="native_entry_transfer.rs"]pub mod entry_transfer;')
s=s.replace('Navigation(entry_navigation::Change) }','Navigation(entry_navigation::Change),Transfer(entry_transfer::Change) }')
s=s.replace('pub struct Runtime {entry_navigation:', 'pub struct Runtime {entry_transfer:entry_transfer::State,entry_navigation:')
s=s.replace('Ok(Self{entry_navigation:', 'Ok(Self{entry_transfer:entry_transfer::State::default(),entry_navigation:')
s=s.replace('self.entry_navigation.retire()?;', 'self.entry_transfer.retire()?;self.entry_navigation.retire()?;')
s=s.replace('let ingress=if matches!(bytes.first(),Some(200|201))', 'let ingress=if matches!(bytes.first(),Some(243|244)){self.consume_source_transfer_event(&packet).map(StreamIngress::Transfer)}else if matches!(bytes.first(),Some(200|201))')
needle='match ingress {Ok(StreamIngress::Actor'
s=s.replace(needle,'match ingress {Ok(StreamIngress::Transfer(change@entry_transfer::Change::RestartAfterUnqualifiedAddress{..}))=>{batch.events.push(StreamEvent{ingress:StreamIngress::Transfer(change),counter_step:Some(step)});batch.terminal=Some(source_stream::End{terminal:source_stream::Terminal::DispatchRefused,before});break;},Ok(StreamIngress::Actor')
assert s.count('entry_transfer.retire()?')==2
assert 'Some(243|244)' in s
p.write_text(s,encoding='utf-8')
