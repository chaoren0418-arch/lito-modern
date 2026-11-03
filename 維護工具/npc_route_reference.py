"""Validate an archived geometry-only bridge without relabeling its old fixture."""
from pathlib import Path
import hashlib
import json
import zipfile


def assert_route_reference(case, receipt, route, fixture_name, copied_file):
    if route['jar_sha256'].upper() == receipt['jar_sha256'].upper():
        return
    path = case / 'reference-evidence/geometry-compatibility.json'
    sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
    bridge = json.loads(path.read_text(encoding='utf-8-sig'))
    assert sha(path) == receipt['route_geometry_bridge']['sha256']
    assert bridge['old_sha256'] == route['jar_sha256'].upper()
    assert bridge['current_sha256'] == receipt['jar_sha256'].upper()
    assert bridge['fixtures'][fixture_name] == sha(case / copied_file)
    assert sha(Path(bridge['old_jar'])) == bridge['old_sha256']
    assert sha(Path(bridge['current_jar'])) == bridge['current_sha256']
    with zipfile.ZipFile(bridge['old_jar']) as old, zipfile.ZipFile(bridge['current_jar']) as current:
        for entry in bridge['byte_identical_classes']:
            data = old.read(entry['name'])
            assert data == current.read(entry['name'])
            assert hashlib.sha256(data).hexdigest().upper() == entry['sha256']
        assets = {n for n in old.namelist() if not n.endswith(('.class', '/'))}
        assert assets == {n for n in current.namelist() if not n.endswith(('.class', '/'))}
        assert all(old.read(n) == current.read(n) for n in assets)
    assert bridge['normalized_Npc_without_name_identical']
    assert receipt['route_geometry_bridge']['name_change_excluded']
