"""Remove a stale unresolved-defaults sentence; retain its original report."""
from pathlib import Path
from datetime import datetime,timezone
import json
from native_E149_recovery_contract import R,read,sha,encoded,require

def main():
 report=R.parent/'LiTo381-總開發報告.md';require(sha(report)=='B19E27646267104794536BB3A5383770A51A9F8BD3923906A6ED830B0E1A6855','report predecessor differs');old=report.read_bytes();s=old.decode('utf-8-sig');a='表初始化與方向6／shadow defaults仍須查，不預設Screen或其他blend。';b='本輪已補普通初始化defaults與既有Over提交橋（見上方）；任意後續globals與原生獨立像素仍未驗。';require(s.count(a)==1,'stale sentence differs');out=R/'驗證紀錄/current/master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);(out/'updater.py').write_bytes(Path(__file__).read_bytes());(out/'after.md').write_text(s.replace(a,b),encoding='utf-8',newline='\n');require(report.read_bytes()==old,'concurrent writer');report.write_bytes((out/'after.md').read_bytes());require(report.read_bytes()==(out/'after.md').read_bytes(),'report differs');w=read(R/'工作佇列.json');ref=w['current_E149_hide97_static_draw_bridge_development'];result=dict(before_sha256='B19E27646267104794536BB3A5383770A51A9F8BD3923906A6ED830B0E1A6855',after_sha256=sha(report),latest_evidence=ref['evidence'],latest_evidence_sha256=ref['evidence_sha256'],reconciled_stale_defaults_sentence=True,tests_executed=0,hide97_renderer_integrated=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(dict(master_readback=str(out/'readback.json'),**result),ensure_ascii=False))

if __name__=='__main__':main()
