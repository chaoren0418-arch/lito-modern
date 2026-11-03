"""Prepare an immutable case contract. No runtime producer or test is invoked."""
import argparse,json
from native_E149_recovery_contract import SCENARIOS,make_plan,sha,read
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--profile',choices=['Debug','Release'],required=True)
 ap.add_argument('--scenario',choices=list(SCENARIOS),required=True);a=ap.parse_args();p=make_plan(a.profile,a.scenario)
 data=read(p)
 print(json.dumps(dict(plan=str(p),plan_sha256=sha(p),linked_image_qualified=data['linked_image_qualified'],
  runtime_producer_pending=data['runtime_producer_implementation_pending'],tests_deferred=data['test_execution_deferred_at_preparation'],
  tests_executed=0,database_started=False),ensure_ascii=False))
if __name__=='__main__':main()
