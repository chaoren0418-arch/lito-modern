"""Qualify owned native hotel widgets against independently observed formal states."""
import hashlib
import json
import sys
from pathlib import Path

p = Path(sys.argv[1]).resolve()
read = lambda name: json.loads((p / name).read_text(encoding="utf-8-sig"))
sha = lambda file: hashlib.sha256(file.read_bytes()).hexdigest().upper()
r = read("receipt.json")
assert r["status"] == "verified_native_hotel_widget_flow"
assert r["action_profile"] == "rental-hotel" and r["hotel_widget_flow_verified"]
assert r["private_desktop_world_focus_verified"] and not r["physical_input_verified"]
route = read("hotel-route.json")
assert sha(p / "hotel-route.json") == r["hotel_route_sha256"]
from npc_route_reference import assert_route_reference
assert_route_reference(p,r,route,'LiTo381-Modern/tests/fixtures/native-gui/hotel-route.json','hotel-route.json')

keys = [0x68, 0x69, 0x66, 0x63, 0x62, 0x61, 0x64, 0x67]
deltas = [(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)]
for step in route["steps"]:
    assert step["key"] == keys[step["heading"]]
    assert step["after"] == [v+d for v,d in zip(step["before"], deltas[step["heading"]])]
states = [v["state"] for line in (p / "server-trace.jsonl").read_text(encoding="utf-8-sig").splitlines() if "state" in (v := json.loads(line))]
checks = []
for role, object_id in [("a",1),("b",3)]:
    steps = [v for v in r["steps"] if v["role"] == role]
    images = {v["step"]:v for v in steps if "png" in v}
    moves = [v for v in steps if v["step"] == "native-key-message-movement"]
    assert len(moves) == len(route["steps"]) == 13
    owned = [v for v in states if v["object_id"] == object_id and v["phase"] == "BATTLE" and v["rental_town"]]
    cursor = 0
    for move, planned in zip(moves, route["steps"]):
        assert all(move[k] == planned[k] for k in ["before","after","key"])
        cursor = next(i for i in range(cursor,len(owned)) if [owned[i]["x"],owned[i]["y"]] == move["after"])
    baseline = owned[cursor]
    items = lambda v: {x["id"]:x for x in v["rental_inventory"]}
    assert baseline["rental_hotel"] is None and baseline["rental_gems"] == 20
    pending = next(i for i in range(cursor,len(owned)) if owned[i]["hotel_input_pending"])
    first = next(i for i in range(pending+1,len(owned)) if owned[i]["rental_hotel"])
    returned = next(i for i in range(first+1,len(owned)) if owned[i]["rental_hotel"] is None)
    second = next(i for i in range(returned+1,len(owned)) if owned[i]["rental_hotel"])
    arrival = next(i for i in range(second+1,len(owned)) if owned[i]["rental_map"] != 500)
    initial_lease, final_lease = owned[first]["rental_hotel"], owned[second]["rental_hotel"]
    assert initial_lease["keys"] == 1 and initial_lease["map"] == 501
    assert len(set(items(owned[first])) - set(items(baseline))) == 1
    assert items(owned[returned]) == items(baseline)
    assert final_lease["token"] != initial_lease["token"]
    entry = next(v for v in steps if v["step"] == "hotel-entry-authority")
    quantity, map_id = entry["quantity"], entry["map"]
    assert final_lease["keys"] == quantity and final_lease["map"] == map_id
    assert owned[arrival]["rental_map"] == map_id
    assert items(owned[arrival]) == items(owned[second])
    assert len(set(items(owned[second])) - set(items(baseline))) == quantity
    assert all(v["rental_gems"] == 20 and v["shop_points"] == 5000 for v in owned[cursor:arrival+1])
    assert "席巴:" in images["hotel-original"]["rental_body"]
    assert "幾個鑰匙" in images["hotel-key-count"]["rental_body"]
    for invalid in [0,9]:
        assert "請輸入 1～8 的數量" in images[f"hotel-invalid-{invalid}"]["status"]
    assert f"地圖 {map_id}" in images["hotel-entered"]["status"]
    checks.append({"role":role,"object_id":object_id,"steps":len(moves),"quantity":quantity,"map":map_id,"first_lease":initial_lease,"returned_inventory_restored":True,"entry_lease":final_lease,"formal_indices":[pending,first,returned,second,arrival]})
for step in r["steps"]:
    if "png" in step:
        file = Path(step["png"])
        assert file.parent == p and sha(file) == step["png_sha256"]
server = read("result.json")
assert server["actual_client_sessions"] == 4 and server["memberships_empty"]
assert server["authority_mutations"] == 0 and not server["sql_access"] and not server["state_persistent"]
assert server["active_hotel_leases_after_disconnect"] == 0
out = {"status":"verified_two_native_hotel_widget_flows","exe_sha256":r["exe_sha256"],"jar_sha256":r["jar_sha256"],"receipt_sha256":sha(p/"receipt.json"),"trace_sha256":sha(p/"server-trace.jsonl"),"checks":checks,"input_scope":"Owned unswitched private desktop; native control messages and posted keys with actual GUI thread focus; no foreground acquisition or authority mutation","physical_input_verified":False,"self_exit_verified":False,"natural_expiry_verified":False,"full_product_verified":False}
(p / "hotel-authority-comparison.json").write_text(json.dumps(out,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
print(json.dumps({"status":out["status"],"maps":[v["map"] for v in checks]}))
