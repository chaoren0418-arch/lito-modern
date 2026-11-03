from pathlib import Path
import hashlib,json,sys
p=Path(sys.argv[1]).resolve()
read=lambda f:json.loads(f.read_text(encoding="utf-8-sig"))
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/"receipt.json")
assert r["status"]=="native_formal_travel_test_passed_independent_capture_verification_pending"
assert r["jar_sha256"]=="C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539"
assert not r["player_tables_accessed"] and not r["source_project_modified"] and r["owned_processes_stopped"]
assert r["game_harness"]["sessions"]==4 and r["game_harness"]["runtime_authority_mutations"]==0
retention=p/"test-artifact-retention.json"
exe=Path(read(retention)["retained_path"]) if retention.exists() else Path(r["game_test"]["exe"])
assert sha(exe)==r["game_test"]["sha256"]
for name,digest in r["source_pins"].items():assert sha(p/"source-snapshot"/name)==digest
cap=next((p/"travel-captures").glob("*/frames.jsonl"))
frames=[json.loads(x) for x in cap.read_text(encoding="utf-8-sig").splitlines()]
assert sum(x["plain"]=="MASKED_C_LOGIN" for x in frames)==4
packets=[(x,bytes.fromhex(x["plain"])) for x in frames if x["plain"]!="MASKED_C_LOGIN"]
a,b=read(p/"a-travel.json"),read(p/"b-travel.json")
own=a["before"]["world"]["status"]["object_id"]
item=a["item"]["id"]
uses=[(i,x,z) for i,(x,z) in enumerate(packets) if x["direction"]=="client_to_server" and z[0]==0x1c]
assert len(uses)==2
for (_,_,z),position in zip(uses,[a["blocked_position"],a["position"]]):
 assert len(z)==13 and int.from_bytes(z[1:5],"little")==item and z[5:9]==bytes(4)
 assert [int.from_bytes(z[9:11],"little"),int.from_bytes(z[11:13],"little")]==position
reject=[(i,z) for i,(x,z) in enumerate(packets) if x["direction"]=="server_to_client" and z[:2]==bytes([0x0f,9]) and z[2:-1].decode("cp950")=="該座標已有單位或障礙物，未執行傳送。"]
assert len(reject)==1 and uses[0][0]<reject[0][0]<uses[1][0]
portals=[(i,z) for i,(x,z) in enumerate(packets) if x["direction"]=="server_to_client" and z[0]==0x5a]
assert len(portals)==1
idx,z=portals[0]
assert len(z)==7 and idx>uses[1][0]
assert int.from_bytes(z[1:3],"little")==a["before"]["world"]["map_id"]
assert [int.from_bytes(z[3:5],"little"),int.from_bytes(z[5:7],"little")]==a["position"]
acks=[i for i,(x,z) in enumerate(packets) if x["direction"]=="client_to_server" and z==bytes([0x08])]
assert len(acks)==1 and acks[0]>idx
fx=[(i,z) for i,(x,z) in enumerate(packets) if x["direction"]=="server_to_client" and z[0]==0x37 and int.from_bytes(z[5:7],"little")==169]
assert fx and all(len(z)==7 and int.from_bytes(z[1:5],"little")==own for i,z in fx)
assert any(uses[1][0]<i<idx for i,z in fx)
for key in ["rejected","after"]:
 assert a[key]["inventory"]["owner"]==a["before"]["inventory"]["owner"]
 assert a[key]["inventory"]["items"][str(item)]==a["item"]
assert a["rejected"]["world"]["generation"]==a["before"]["world"]["generation"]
assert a["rejected"]["item_feedback"]=="TargetRejected"
assert a["after"]["world"]["generation"]>a["before"]["world"]["generation"]
assert a["after"]["world"]["completed_portals"]==a["before"]["world"]["completed_portals"]+1
assert a["old_frame_rejected"] and a["old_cursor_rejected"]
assert b["before"]["world"]["generation"]==b["after"]["world"]["generation"]
assert b["before"]["inventory"]["owner"]==b["after"]["inventory"]["owner"]
rows=[json.loads(x) for x in (p/"travel-server-trace.jsonl").read_text(encoding="utf-8-sig").splitlines()]
owner=[v for v in rows if v["test_account"]=="PeerFixture" and v["phase"]=="BATTLE"]
assert owner and any([v["x"],v["y"]]==a["position"] for v in owner)
assert all(any(i["id"]==item and i["catalog_id"]==42501 and i["quantity"]==1 for i in v["inventory"]) for v in owner)
if "continued" in a:
 continued=a["continued"]["world"];position=a["position"]
 delta=[(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)][a["walk_heading"]]
 destination=[position[0]+delta[0],position[1]+delta[1]]
 actor=continued["actors"][str(own)]
 assert [actor["x"],actor["y"]]==destination
 assert continued["generation"]==a["after"]["world"]["generation"]
 assert any(v["phase"]=="BATTLE" and [v["x"],v["y"]]==destination for v in owner)
 moves=[(i,z) for i,(x,z) in enumerate(packets) if x["direction"]=="client_to_server" and z[0]==0x0a]
 assert len(moves)==1 and moves[0][0]>acks[0]
 move=moves[0][1]
 assert len(move)==6 and [int.from_bytes(move[1:3],"little"),int.from_bytes(move[3:5],"little")]==position and move[5]==a["walk_heading"]
peer_fx=[v for v in b["observations"] if any(e["graphics"]==169 for e in v["effects"])]
result={"status":"verified_actual_SQL42501_rejection_and_same_map_portal_owner_retirement_peer_preserved",
 "verifier_sha256":sha(Path(__file__)),
 "profile":r["game_test"]["profile"],"test_exe_sha256":r["game_test"]["sha256"],"reference_sha256":r["jar_sha256"],
 "formal_sessions":4,"target_requests":2,"explicit_rejections":1,"portal_packets":1,"portal_acks":1,"effect169_packets":len(fx),
 "owner_generation":[a["before"]["world"]["generation"],a["after"]["world"]["generation"]],
 "empty_portal_snapshot_observed":any(v["portal"] is not None and v["actors"]==0 and not v["ready"] for v in a["transitions"]),
 "post_portal_authoritative_walk_verified":"continued" in a,
 "peer_effect169_observed":bool(peer_fx),"peer_frame_travel_effect":b["frame"].get("travel_effects"),
 "GUI_runtime_verified":False,"physical_input_verified":False,"owner_departure_effect_frame_verified":False,
 "TestMage_GM_item_possession_verified":False,"player_tables_accessed":False,
 "file_hashes":{str(f.relative_to(p)):sha(f) for f in [p/"a-travel.json",p/"b-travel.json",p/"travel-server-trace.jsonl",cap,p/"receipt.json",p/"game-test.log"]+list(p.glob("*.png"))}}
(p/"authority-verifier.py").write_bytes(Path(__file__).read_bytes())
(p/"authority-comparison.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
print(json.dumps({k:v for k,v in result.items() if k not in ["file_hashes","peer_frame_travel_effect"]},ensure_ascii=False))
