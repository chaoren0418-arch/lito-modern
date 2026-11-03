from r491_common import *
s=(P/'tools/r519-self-consumer.rs').read_text('utf-8').split('include!("r519-self-case.rs");')[0].replace('R519','R520')
s=s.replace('java_room_self as self_flow,','java_room_self as self_flow,java_room_settings as settings,')
s=s.replace('r.handle_player_entry_command(Command::OpenLogin)?;','r.configure_java_room_settings_adapter(settings_adapter()?)?;r.handle_player_entry_command(Command::OpenLogin)?;')
s=s.replace('let mut bodies=bodies;if bodies.first().is_some_and(|b|b.is_empty()){g.expect(&[208])?;bodies.remove(0);}g.send_many(bodies)?;','g.send_many(bodies)?;')
(P/'tools/r520-settings-consumer.rs').write_text(s+'include!("r520-settings-case.rs");\n','utf-8')
print('R520 producer normal scaffold copied; old self case not executed, no C_D0 command hook')
