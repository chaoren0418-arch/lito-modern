"""Audit completed native combat receipts; no network or gameplay mutation."""
import datetime
import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "驗證紀錄/current/native-impact"


def read(path):
    return json.loads((ROOT / path).read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def png_pin(path, expected):
    path = Path(path)
    path.resolve().relative_to(ROOT)
    with Image.open(path) as image:
        assert image.size == (800, 600)
        rgba = hashlib.sha256(image.convert("RGBA").tobytes()).hexdigest()
    assert rgba == expected, path
    return {"path": str(path), "sha256": digest(path), "rgba_sha256": rgba}


def main():
    source = read("驗證紀錄/current/native-combat/receipt.json")
    exe = ROOT / "LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe"
    assert source["exe_sha256"] == digest(exe)
    assert source["jar_sha256"] == read("驗證紀錄/current/reference-snapshot.json")["sha256"]
    cases, sounds = [], {}
    ranged = None
    for case in source["cases"]:
        rounds = case["client"]["combat_rounds"] if case["mode"] == "room-world-cycle" else [case["client"]["combat"]]
        for index, run in enumerate(rounds, 1):
            events, pngs, arrivals, hurt_frames, flight_frames = set(), [], [], [], []
            for frame in run["frames"]:
                if frame.get("png"):
                    pngs.append(dict(png_pin(frame["png"], frame["rgba_sha256"]), elapsed_ns=frame["elapsed_ns"]))
                for actor in frame["actors"]:
                    for event in actor.get("entered_events", []):
                        key = json.dumps(event, sort_keys=True)
                        assert key not in events, (case["mode"], index, event)
                        events.add(key)
                for audio in frame["audio"]:
                    assert audio["prepared"], audio
                    row = {"id": audio["event"]["sound"], "bytes": audio["bytes"], "sha256": audio["sha256"]}
                    assert sounds.setdefault(row["id"], row) == row
                if case["mode"] == "room-world-ranged":
                    callbacks = frame["projectiles"]["hurt_callbacks"]
                    for callback in callbacks:
                        assert callback["accepted"] and callback["target"] == 2
                        arrivals.append(dict(callback, elapsed_ns=frame["elapsed_ns"]))
                    target = next(a for a in frame["actors"] if a["id"] == 2)
                    if target["phase"] == "damage":
                        assert len(arrivals) == 1, "hurt before arrival"
                        hurt_frames.append(frame["elapsed_ns"])
                    if any(f["drawn"] for f in frame["projectiles"]["flights"]):
                        assert not arrivals, "ordinary arrow survives arrival"
                        flight_frames.append(frame["elapsed_ns"])
            row = {"mode": case["mode"], "round": index, "epoch": run["epoch"], "generation": run["result"]["world"]["generation"], "frames": len(run["frames"]), "pngs": pngs, "attacks": run["attacks_sent"], "frame_events": len(events)}
            cases.append(row)
            if case["mode"] == "room-world-ranged":
                assert len(arrivals) == 1 and hurt_frames and flight_frames
                assert case["server"]["remaining_arrows"] == 19
                assert run["frames"][-1]["projectiles"]["active"] == 0
                ranged = {"arrivals": arrivals, "hurt_frames_ns": hurt_frames, "flight_frames_ns": flight_frames, "arrows_before": 20, "arrows_after": 19, "source_banks": run["frames"][-1]["projectiles"]["sources"], "world_projectile": run["result"]["world"]["projectiles"][0]}
    assert len(cases) == 7 and ranged is not None
    vector = read("LiTo381-Modern/target/current/verification/native-arrow66.json")
    vector_pngs = [png_pin(f["png"], f["rgba_sha256"]) for f in vector["frames"] if f.get("png")]
    receipt = {"schema": 4, "utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "exe_sha256": source["exe_sha256"], "jar_sha256": source["jar_sha256"], "cases": cases, "frames": sum(c["frames"] for c in cases), "pngs": sum(len(c["pngs"]) for c in cases), "frame_events": sum(c["frame_events"] for c in cases), "sounds": sorted(sounds.values(), key=lambda s: s["id"]), "ordinary_arrow": ranged, "resource_vector_test": {"frames": len(vector["frames"]), "pngs": vector_pngs, "tcp": False}, "gui_tested": False, "audio_device_tested": False}
    OUT.mkdir(exist_ok=True)
    (OUT / "visual-receipt.json").write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({k: receipt[k] for k in ["frames", "pngs", "frame_events"]}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", default=str(OUT))
    OUT = Path(parser.parse_args().output).resolve()
    OUT.relative_to(ROOT)
    main()
