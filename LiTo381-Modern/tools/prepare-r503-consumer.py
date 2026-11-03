from r491_common import *
old=(P/'tools/r502-player-events-consumer.rs').read_text(encoding='utf-8');helpers=old[:old.index('fn selected(')].replace('r502','r503').replace('R502','R503')
helpers=helpers.replace('        UiPage,','        UiPage, entry_navigation::{Scene,decode}, source_send::WriteOutcome,')
(P/'tools/r503-navigation-consumer.rs').write_text(helpers+(P/'tools/r503-navigation-consumer-body.rs').read_text(encoding='utf-8'),encoding='utf-8')
