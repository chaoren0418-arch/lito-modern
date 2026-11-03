"""Rebuild the route-only bridge using the immutable BBC snapshot; preserve historical evidence."""
from pathlib import Path
from datetime import datetime, timezone
import difflib
import hashlib
import json
import re
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / 'LiTo381-Modern/target/current/server-reference'
OLD = BASE / 'candidate-BBC59308.jar'
NEW = BASE / 'formal-server-C02AAAE8.jar'
OUT = ROOT / '驗證紀錄/current/native-hotel-current-reference' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
OLD_SHA = 'BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43'
NEW_SHA = 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'


def main():
    assert sha(OLD) == OLD_SHA and sha(NEW) == NEW_SHA
    OUT.mkdir(parents=True, exist_ok=False)
    families = ['ClassicWorldMapCatalog', 'NativeLitoTownPassabilityCatalog',
                'NativeLitoRentalHotelCatalog', 'NativeLitoRentalMerchantCatalog',
                'NativeLitoRentalTownNpcCatalog']
    classes = []
    assets = []
    with zipfile.ZipFile(OLD) as old, zipfile.ZipFile(NEW) as new:
        old_assets = {n for n in old.namelist() if not n.endswith(('.class', '/'))}
        new_assets = {n for n in new.namelist() if not n.endswith(('.class', '/'))}
        assert old_assets == new_assets
        for name in sorted(old_assets):
            raw = old.read(name)
            assert raw == new.read(name), f'embedded resource changed: {name}'
            assets.append(dict(name=name, sha256=hashlib.sha256(raw).hexdigest().upper()))
        for name in old.namelist():
            if not name.endswith('.class'):
                continue
            stem = name.rsplit('/', 1)[-1].split('$')[0].removesuffix('.class')
            if stem not in families or name.endswith('NativeLitoRentalTownNpcCatalog$Npc.class'):
                continue
            assert old.read(name) == new.read(name), f'geometry catalogue changed: {name}'
            classes.append(dict(name=name, sha256=hashlib.sha256(old.read(name)).hexdigest().upper()))
    assert all(any(x['name'].endswith('/' + f + '.class') for x in classes) for f in families)
    javap = r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe'
    texts = []
    for label, jar in [('old', OLD), ('current', NEW)]:
        result = subprocess.run([javap, '-J-Dfile.encoding=UTF-8', '-c', '-p', '-classpath', str(jar),
                                 'local.lito381.server.NativeLitoRentalTownNpcCatalog$Npc'], capture_output=True, check=True)
        text = result.stdout.decode('utf-8')
        (OUT / f'Npc-{label}.javap.txt').write_text(text, encoding='utf-8')
        text = re.sub(r'#\d+', '#CP', text)
        text = '\n'.join(re.sub(r'\s+//', ' //', line).strip() for line in text.splitlines())
        text, count = re.subn(r'public java\.lang\.String name\(\);\nCode:.*?(?=\npublic java\.lang\.String startHtml)', '', text, flags=re.S)
        assert count == 1
        texts.append(text)
    assert texts[0] == texts[1], '\n'.join(difflib.unified_diff(texts[0].splitlines(), texts[1].splitlines()))
    fixtures = {}
    for name in ['hotel-route.json', 'merchant-plan.json', 'citizen-routes.json']:
        path = ROOT / 'LiTo381-Modern/tests/fixtures/native-gui' / name
        data = json.loads(path.read_text(encoding='utf-8-sig'))
        assert data['jar_sha256'].upper() == OLD_SHA
        fixtures[path.relative_to(ROOT).as_posix()] = sha(path)
    proof = dict(status='verified_route_geometry_catalogues_and_Npc_except_name',
                 old_jar=str(OLD), old_sha256=OLD_SHA, current_jar=str(NEW), current_sha256=NEW_SHA,
                 byte_identical_classes=classes, normalized_Npc_without_name_identical=True,
                 byte_identical_nonclass_resources=assets,
                 excluded_name_change='Current name() calls ResourceDatabase.npcName(name,resourceIndex); labels must be checked against current runtime.',
                 fixtures=fixtures, old_results_relabelled=False, full_protocol_compatibility_claimed=False,
                 verifier_sha256=sha(Path(__file__)))
    (OUT / 'geometry-compatibility.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    reference = dict(source=r'D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar',
                     snapshot=str(NEW), sha256=NEW_SHA, source_access='read_only',
                     route_geometry_compatibility=str(OUT / 'geometry-compatibility.json'),
                     route_geometry_compatibility_sha256=sha(OUT / 'geometry-compatibility.json'))
    assert sha(Path(reference['source'])) == NEW_SHA
    (OUT / 'reference.json').write_text(json.dumps(reference, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': proof['status'], 'classes': len(classes), 'fixtures': len(fixtures), 'reference': str(OUT / 'reference.json')}, ensure_ascii=False))


if __name__ == '__main__':
    main()
