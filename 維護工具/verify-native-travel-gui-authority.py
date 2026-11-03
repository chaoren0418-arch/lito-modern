from pathlib import Path
import json,hashlib,sys
p=Path(sys.argv[1]).resolve()
read=lambda f:json.loads(f.read_text(encoding="utf-8-sig"))
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/"receipt.json");g=read(p/"travel-gui.json")
assert r["status"]=="native_GUI_travel_test_passed_independent_capture_verification_pending"
assert r["owned_processes_stopped"] and not r["player_tables_accessed"] and not r["source_project_modified"]
assert r["jar_sha256"]=="C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539"
assert r["game_harness"]["sessions"]==4 and r["game_harness"]["runtime_authority_mutations"]==0
assert g["status"]=="native_GUI_travel_flow_passed_independent_verification_pending"
assert g["private_desktop"] and g["desktop_name"].startswith("LitoTravelGui") and not g["physical_input_verified"]
for name,digest in r["source_pins"].items():assert sha(p/"source-snapshot"/name)==digest
for step in g["steps"]:
 if "png" in step:assert sha(Path(step["png"]))==step["png_sha256"]
steps=lambda name:[x for x in g["steps"] if x["role"]=="a" and x["step"]==name]
assert len(steps("armed"))==4
attempts=steps("target-pointer-attempt")
assert len(attempts)==2 and all(x["attempt"]==1 and x["single_click"] and x["paint_suppressed_ms"]==700 for x in attempts)
assert all(not x["cancel_visible"] and not x["return_visible"] for x in steps("inventory-wand"))
assert steps("esc-cancelled")[0]["request_count"]==0
focus=steps("focus-changed")[0]
assert focus["inventory_visible"] and focus["focus"]!=0 and focus["request_count"]==0
assert steps("focus-cancelled")[0]["request_count"]==0
assert steps("target-rejected")[0]["request_count"]==1 and "該座標已有單位或障礙物" in steps("target-rejected")[0]["status"]
assert steps("old-cursor-rejected")[0]["request_count"]==2
before,after,rejected,continued=g["before_a"],g["after_a"],g["rejected"],g["continued"]
assert before["phase"]==after["phase"]==rejected["phase"]==continued["phase"]=="BATTLE"
assert before["session_identity"]==after["session_identity"]==rejected["session_identity"]==continued["session_identity"]
assert [before["x"],before["y"]]==[rejected["x"],rejected["y"]]==[32729,32797]
assert [after["x"],after["y"]]==[32735,32799]
assert [continued["x"],continued["y"]]==[32735,32798]
assert g["before_b"]["session_identity"]==g["after_b"]["session_identity"]
assert [g["before_b"]["x"],g["before_b"]["y"]]==[g["after_b"]["x"],g["after_b"]["y"]]
item=next(i for i in before["inventory"] if i["catalog_id"]==42501)
for state in [rejected,after,continued]:assert next(i for i in state["inventory"] if i["id"]==item["id"])==item
assert item["quantity"]==1
cap=next((p/"travel-captures").glob("*/frames.jsonl"))
frames=[json.loads(x) for x in cap.read_text(encoding="utf-8-sig").splitlines()]
assert sum(x["plain"]=="MASKED_C_LOGIN" for x in frames)==4
packets=[(x,bytes.fromhex(x["plain"])) for x in frames if x["plain"]!="MASKED_C_LOGIN"]
uses=[(i,z) for i,(x,z) in enumerate(packets) if x["direction"]=="client_to_server" and z[0]==0x1c]
assert len(uses)==2
for (i,z),pos in zip(uses,[[32730,32797],[32735,32799]]):
 assert len(z)==13 and int.from_bytes(z[1:5],"little")==item["id"]
 assert [int.from_bytes(z[9:11],"little"),int.from_bytes(z[11:13],"little")]==pos
reject=[i for i,(x,z) in enumerate(packets) if x["direction"]=="server_to_client" and z[:2]==bytes([0x0f,9]) and z[2:-1].decode("cp950")=="該座標已有單位或障礙物，未執行傳送。"]
assert len(reject)==1 and uses[0][0]<reject[0]<uses[1][0]
portal=[(i,z) for i,(x,z) in enumerate(packets) if x["direction"]=="server_to_client" and z[0]==0x5a]
ack=[i for i,(x,z) in enumerate(packets) if x["direction"]=="client_to_server" and z==bytes([0x08])]
assert len(portal)==len(ack)==1 and uses[1][0]<portal[0][0]<ack[0]
z=portal[0][1];assert z==bytes([0x5a,0x34,2,0xdf,0x7f,0x1f,0x80])
fx=[z for x,z in packets if x["direction"]=="server_to_client" and z[0]==0x37 and int.from_bytes(z[5:7],"little")==169]
assert fx and all(int.from_bytes(z[1:5],"little")==before["object_id"] for z in fx)
moves=[(i,z) for i,(x,z) in enumerate(packets) if x["direction"]=="client_to_server" and z[0]==0x0a]
assert len(moves)==1 and moves[0][0]>ack[0] and moves[0][1]==bytes([0x0a,0xdf,0x7f,0x1f,0x80,0])
result={"status":"verified_native_GUI_SQL42501_Esc_focus_rejection_portal_and_walk",
 "exe_sha256":g["exe_sha256"],"reference_sha256":r["jar_sha256"],"formal_sessions":4,
 "target_requests":2,"esc_cancel_no_request":True,"actual_panel_focus_cancel_no_request":True,
 "visible_rejection":True,"single_visible_frame_clicks":2,"paint_suppressed_ms":700,"portal_acks":1,"effect169_packets":len(fx),"old_cursor_no_replay":True,
 "post_portal_walk_verified":True,"physical_input_verified":False,"owner_departure_effect_frame_verified":False,
 "GUI_runtime_verified":True,"runtime_authority_mutations":0,"player_tables_accessed":False,
 "verifier_sha256":sha(Path(__file__)),"file_hashes":{str(f.relative_to(p)):sha(f) for f in [p/"receipt.json",p/"travel-gui.json",p/"travel-server-trace.jsonl",cap]+list(p.glob("*-GUI-*.png"))}}
(p/"gui-authority-verifier.py").write_bytes(Path(__file__).read_bytes())
(p/"gui-authority-comparison.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
print(json.dumps({k:v for k,v in result.items() if k!="file_hashes"},ensure_ascii=False))
