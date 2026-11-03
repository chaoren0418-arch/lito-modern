from r491_common import *
import shutil
o=P/'target/r500-login-lifecycle-study/preparation-attempt1';o.mkdir(exist_ok=False);shutil.copyfile(P/'tools/prepare-r500-source.py',o/'prepare-r500-source.py');save(o/'failure.json',dict(step='source extraction expansion',error='KeyError nonexistent guessed sub_4C3DF0;partialfragments retained;correctactualnames fromASM beforefreeze',source=pin(o/'prepare-r500-source.py'),Native_execution=False))
