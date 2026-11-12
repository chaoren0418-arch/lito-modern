"""Issue #308 (T036-3): join the closure tool's conservative class with the manual review and
write the final lists; optionally update the T036 ledger (維護工具/test-whitelist/full.tsv).

Read-only on src/ and tests/. Does not compile or run project code.

  python -I -B 維護工具/t036-helper-closure/issue308-finalize.py            # lists + ledger check (dry)
  python -I -B 維護工具/t036-helper-closure/issue308-finalize.py --ledger   # also rewrite full.tsv

Inputs : docs/reports/issue-308/closure-summary.tsv (from audit-test-helper-closure.py), scope.tsv
Outputs: docs/reports/issue-308/verdicts.tsv, approved-candidates.tsv, resource-root-readonly.tsv,
         still-excluded.tsv
"""
import collections, io, os, re, sys

OUT = "docs/reports/issue-308"
WL = "維護工具/test-whitelist"
ISSUE = "#308"

# Manual downgrades: the tool reports R (read-only resource root) only because it links every
# method of a touched type by name. Each entry was checked by reading the test body and callee.
PURE = "A"
MANUAL = {
    ("native_light_spell", "light_uses_self_selector_without_predicting_authority_and_scopes_UI_choice"):
        (PURE, "函式內 `use native_skill_selection::{Model,Target}`；工具把 `Model` 連到 native_world_scene::Model。"
               "本體只呼叫 native_skills::request、Model::sync／select／rows／choice、Target::capture（tests/native_light_spell.rs:15–30）"),
    ("native_skills", "invisibility_tick_envelope_guards_clock_and_menu_preserve_authority"):
        (PURE, "本體只有 self_tick_request／request／ClientTick::now、LoginController::default()、選單（tests/native_skills.rs:597–629）；"
               "沒有任何路徑參數。工具因觸及 LoginController 把它的其他方法連進來"),
}
for name in ("arrival_is_once_at_the_original_30ms_terminal_callback_even_when_frames_skip",
             "no_hurt_for_miss_zero_damage_death_removed_or_replaced_objects",
             "same_id_replacement_before_launch_cannot_bind_an_old_cue",
             "physical_class3_types_launch_once_and_release_hurt_only_at_arrival",
             "throwing_flights_retire_on_identity_change_and_do_not_hurt_misses_or_dead_targets"):
    MANUAL[("native_ranged", name)] = (
        PURE, "只呼叫 Projectiles::default() 與 update()。update（native_projectiles.rs:181–318）不呼叫 load；"
              "讀檔只在 prepare（:147）與 draw_to（:336）經 load（:168）。工具因觸及 Projectiles 型別把 load 連進來")
# Tool class U resolved by the two rules added to site-adjudication.json.
MANUAL[("native_ranged", "original_throwing_action63_launches_sting2989_through_world_model")] = (
    "V", "render_projectile（tests/native_ranged.rs:160–231）：Model::load／draw_at（唯讀資源根）＋寫 target/current/verification 固定檔名；與 #306 第 45 項同一 helper")
MANUAL[("native_ranged", "original_stone_and_energy_resources_reach_model_arrival_and_retirement")] = MANUAL[
    ("native_ranged", "original_throwing_action63_launches_sting2989_through_world_model")]

RENAMED = {("native_ranged", "malformed_ranged_packets_are_atomic_for_every_flight_type"):
           "malformed_ranged_packets_are_atomic_and_other_effect_branches_remain_unknown"}
TOOL2FINAL = {"P": "A", "R": "B", "G": "B"}
BASIS = {"A": "閉包內沒有檔案、網路、子程序、環境變數、視窗、音效裝置、執行緒原語",
         "B": "只多唯讀讀取寫死的 D:\\lineage開發\\0-客戶端，其餘在記憶體"}


def rows(path, enc="utf-8"):
    with io.open(path, encoding=enc) as fh:
        for line in fh:
            line = line.rstrip("\r\n")
            if line and not line.startswith("#"):
                yield line.split("\t")


def main():
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
    covered = {}
    for f, label in (("tests.tsv", "tests.tsv"), ("skills.tsv", "skills.tsv")):
        for c in rows(os.path.join(WL, f), "utf-8-sig"):
            covered.setdefault((c[0], c[1]), []).append(label)
    ledger_path = os.path.join(WL, "full.tsv")
    with io.open(ledger_path, encoding="utf-8", newline="") as fh:
        raw = fh.read()
    eol = "\r\n" if "\r\n" in raw else "\n"
    bom = raw.startswith("\ufeff")
    ledger = [l.split("\t") for l in raw.lstrip("\ufeff").split(eol) if l]
    by_key = {(c[0], c[1]): c for c in ledger}

    final = []
    for c in rows(os.path.join(OUT, "closure-summary.tsv")):
        target, name, ignored, tool = c[0], c[1], c[2], c[3]
        key = (target, name)
        if key in MANUAL:
            cls, basis = MANUAL[key]
        else:
            cls = TOOL2FINAL.get(tool)
            if cls is None:
                raise SystemExit("unresolved tool class %s for %s::%s" % (tool, target, name))
            basis = BASIS[cls]
        if key in RENAMED:
            origin = "54-renamed"
        elif key in by_key:
            origin = "54"
        else:
            origin = "new"
        final.append([target, name, ignored, origin, tool, cls, "+".join(covered.get(key, [])) or "-", c[5], c[8], basis])

    with io.open(os.path.join(OUT, "verdicts.tsv"), "w", encoding="utf-8", newline="\n") as fh:
        fh.write("# target\ttest\tignored\torigin(54|54-renamed|new)\ttool_class\tfinal_class(A 純記憶體|B 唯讀資源根|V 資源根＋固定檔名證據)"
                 "\talready_whitelisted\treached_items\tsource\tbasis\n")
        for r in final:
            fh.write("\t".join(r) + "\n")
    lists = {"approved-candidates.tsv": "A", "resource-root-readonly.tsv": "BV", "still-excluded.tsv": ""}
    for fname, classes in lists.items():
        with io.open(os.path.join(OUT, fname), "w", encoding="utf-8", newline="\n") as fh:
            for r in final:
                if r[5] in classes and r[6] == "-":
                    fh.write("\t".join([r[0], r[1], r[2], "-"]) + "\n")
    cnt = collections.Counter((r[3], r[5], "covered" if r[6] != "-" else "new-release") for r in final)
    for k in sorted(cnt):
        print(cnt[k], *k)
    print("total", len(final))

    # ---- ledger -----------------------------------------------------------
    changed = 0
    for r in final:
        target, name, origin, cls = r[0], r[1], r[3], r[5]
        if origin == "new":
            continue
        old = RENAMED.get((target, name), name)
        row = by_key[(target, old)]
        if row[4] == "covered":
            continue
        if row[4] != "excluded" or row[6] != ISSUE or not row[7].startswith("helper-pending"):
            if row[4] == "run" and row[6] == ISSUE:
                continue  # already applied
            raise SystemExit("unexpected ledger row: " + "\t".join(row))
        row[1] = name
        row[4], row[5], row[6], row[7] = "run", ("client" if cls in "BV" else "-"), ISSUE, "-"
        changed += 1
    left = [c for c in ledger if c[6] == ISSUE and c[4] == "excluded"]
    print("ledger rows released:", changed, "| still excluded under", ISSUE, ":", len(left))
    # Same checks as Read-FullManifest / Resolve-FullPlan in run-local-test-suite.ps1.
    seen = set()
    assert len(ledger) == 1055, len(ledger)
    for c in ledger:
        assert len(c) == 8 and re.match(r"^[A-Za-z0-9_]+$", c[0]) and re.match(r"^[A-Za-z0-9_:]+$", c[1]), c
        assert c[2] in ("yes", "no") and c[4] in ("run", "excluded", "covered") and c[5] in ("-", "client"), c
        assert re.match(r"^[#A-Za-z0-9_-]+$", c[6]), c
        if c[4] == "excluded":
            assert re.match(r"^[a-z0-9-]+: \S", c[7]), c
        else:
            assert c[7] == "-", c
        assert c[4] == "run" or c[5] == "-", c
        assert (c[0], c[1]) not in seen, c
        seen.add((c[0], c[1]))
        k = (c[0], c[1])
        if c[0] != "__lib" and not c[1].startswith("modern_lito::"):
            assert not (c[4] == "excluded" and k in covered), c
            if c[4] == "run":
                assert k not in {kk for kk, v in covered.items() if "tests.tsv" in v}, c
    print("ledger verdicts:", sorted(collections.Counter(c[4] for c in ledger).items()),
          "| run+client:", sum(1 for c in ledger if c[4] == "run" and c[5] == "client"))
    print("ledger exclusion codes:", sorted(collections.Counter(c[7].split(":")[0] for c in ledger if c[4] == "excluded").items()))
    if "--ledger" in sys.argv:
        with io.open(ledger_path, "w", encoding="utf-8", newline="") as fh:
            fh.write(("\ufeff" if bom else "") + eol.join("\t".join(c) for c in ledger) + eol)
        print("ledger written")


if __name__ == "__main__":
    main()
