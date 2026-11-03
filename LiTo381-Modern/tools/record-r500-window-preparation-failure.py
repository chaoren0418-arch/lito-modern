from r491_common import *
import shutil
o=P/'target/r500-login-lifecycle-study/preparation-attempt2';o.mkdir(exist_ok=False);shutil.copyfile(P/'tools/edit-r500-window.py',o/'edit-r500-window.py');save(o/'failure.json',dict(step='Window source patch',error='globalmatchreplacement changedvisibleANDenabled in-memorytext,so subsequentenabledmatcher assertionfailed;Window file notwritten;RuntimeClose/menu/lifetimegates written beforefailure',source=pin(o/'edit-r500-window.py'),consumer_execution=False))
