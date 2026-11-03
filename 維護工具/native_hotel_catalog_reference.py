"""Keep BBC fixture identity; prove route-only C02/75DA catalogue compatibility."""
from pathlib import Path
import hashlib,json,zipfile

ROOT=next(parent for parent in Path(__file__).resolve().parents
          if (parent/'AGENTS.md').is_file() and (parent/'LiTo381-Modern/Cargo.toml').is_file())
REFERENCE=ROOT/'驗證紀錄/current/native-hotel-current-reference/20261016-162329-513817'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def fixture_reference(path,actual_reference):
    fixture=read(path);original=fixture['jar_sha256'].upper();actual_reference=actual_reference.upper()
    if original==actual_reference:
        return dict(status='fixture_reference_identity_exact',fixture_sha256=sha(path),fixture_jar_sha256=original,actual_reference_sha256=actual_reference)
    assert original=='BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43'
    c02='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
    current_sha='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
    assert actual_reference in (c02,current_sha)
    bridge=read(REFERENCE/'geometry-compatibility.json');proof=read(REFERENCE/'independent-readback.json')
    assert proof['status']=='independently_readback_verified_route_only_reference_from_immutable_BBC_and_C02'
    assert sha(REFERENCE/'geometry-compatibility.json')==proof['raw_files']['geometry-compatibility.json']
    assert sha(REFERENCE/'raw.zip')==proof['raw_archive_sha256']
    assert bridge['old_sha256']==sha(Path(bridge['old_jar']))==original
    assert bridge['current_sha256']==sha(Path(bridge['current_jar']))==c02
    classes=bridge['byte_identical_classes'];assert len(classes)==14
    hotel=[r for r in classes if '/NativeLitoRentalHotelCatalog' in r['name']]
    assert {r['name'].split('/')[-1] for r in hotel}=={'NativeLitoRentalHotelCatalog.class','NativeLitoRentalHotelCatalog$Room.class','NativeLitoRentalHotelCatalog$Kind.class','NativeLitoRentalHotelCatalog$Holder.class'}
    with zipfile.ZipFile(bridge['old_jar']) as old,zipfile.ZipFile(bridge['current_jar']) as current:
        for row in classes:
            b=old.read(row['name']);assert b==current.read(row['name']) and hashlib.sha256(b).hexdigest().upper()==row['sha256']
    chain=None
    if actual_reference==current_sha:
        chain_path=ROOT/'驗證紀錄/current/native44FE-hotel/preflight/C02-to-75DA-route-only.json'
        chain=read(chain_path)
        assert chain['old_jar_sha256']==sha(Path(chain['old_jar']))==c02
        assert chain['actual_jar_sha256']==sha(Path(chain['actual_jar']))==current_sha
        assert chain['route_only_classes']==15 and chain['embedded_assets']==646
        with zipfile.ZipFile(chain['old_jar']) as before,zipfile.ZipFile(chain['actual_jar']) as actual:
            assets={n for n in before.namelist() if not n.endswith(('.class','/'))}
            assert assets=={n for n in actual.namelist() if not n.endswith(('.class','/'))}
            required={r['name'] for r in classes}|assets|{'local/lito381/server/NativeLitoRentalTownNpcCatalog$Npc.class'}
            assert set(chain['byte_identical_entries'])==required
            for n,h in chain['byte_identical_entries'].items():
                raw=before.read(n);assert raw==actual.read(n) and hashlib.sha256(raw).hexdigest().upper()==h
        chain=dict(path=str(chain_path),sha256=sha(chain_path),classes=15,embedded_assets=646,whole_protocol_compatibility_claimed=False)
    return dict(status=('fixture_keeps_BBC_identity_75DA_hotel_catalogues_byte_identical' if chain else 'fixture_keeps_BBC_identity_C02_hotel_catalogues_byte_identical'),fixture_sha256=sha(path),fixture_jar_sha256=original,
        actual_reference_sha256=actual_reference,reference_proof=str(REFERENCE/'independent-readback.json'),reference_proof_sha256=sha(REFERENCE/'independent-readback.json'),
        hotel_classes=hotel,all_compared_catalogue_classes=14,current_reference_chain=chain,old_results_relabelled=False,whole_protocol_compatibility_claimed=False)
