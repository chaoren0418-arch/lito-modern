from r491_common import *
p=P/'src/modern_lito/native_city_window.rs';s=p.read_text(encoding='utf-8')
s=s.replace('r.presentation_key().character_ui.active&&r.source_connection_epoch().is_some()&&r.presentation_key().character_ui.active&&','r.presentation_key().character_ui.active&&r.source_connection_epoch().is_some()&&')
s=s.replace('BACK=>true,_=>true','BACK=>!entry||s.character_back.borrow().is_some(),_=>true')
p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8')
a='  let source_send::Intent::SelectCharacter{slot}=intent else';assert a in s
s=s.replace(a,'  ensure!(!self.awaiting_source_roster,"new source roster required after Back");\n'+a)
p.write_text(s,encoding='utf-8')
print('R499 pending/stale Back and generic selected-send gates finalized')
