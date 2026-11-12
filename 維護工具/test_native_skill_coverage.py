"""Offline regression checks for the E149 request-coverage parser."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    'skill_catalog', ROOT / '維護工具/audit-native-E149-skill-catalog.py')
CATALOG = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CATALOG)


class SkillCoverageTests(unittest.TestCase):
    def parse(self, source):
        scratch = ROOT / 'scratch'
        scratch.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch) as temporary:
            path = Path(temporary) / 'native_skills.rs'
            path.write_text(source, encoding='utf-8')
            return CATALOG.source_ids(path)

    def source(self, helper='', arm='14 | 31 => {}'):
        return '''pub fn ground_request() {
            ensure!(matches!(skill, 58 | 63), "此地面技能尚未接入");
        }
        pub fn in_native_screen() {}
        ''' + helper + '''
        pub fn request() {
            match skill {
                1 | 4 => {}
                ''' + arm + '''
            }
        }
        pub fn reuse_millis() { match skill { 999 => 10 } }
        '''

    def test_legacy_explicit_arms_remain_recognized(self):
        result = self.parse(self.source())
        self.assertEqual(result['all_request_ids'], [1, 4, 14, 31, 58, 63])
        self.assertEqual(result['inventory_request_ids'], [])

    def test_compact_and_multiline_helpers_are_recognized(self):
        for helper in [
            'pub fn selector_only(skill:u16)->bool {matches!(skill,14|101|109)}',
            'pub fn selector_only( skill: u16 ) -> bool {\n matches!(skill,\n14 | 101 | 109)\n}',
        ]:
            source = self.source(helper + '\npub fn item_targeted(skill: u16) -> bool { matches!(skill, 107) }',
                                 'id if selector_only(id) => {}')
            result = self.parse(source)
            self.assertEqual(result['all_request_ids'], [1, 4, 14, 58, 63, 101, 107, 109])
            self.assertEqual(result['inventory_request_ids'], [107])

    def test_disconnected_or_unknown_helper_syntax_is_rejected(self):
        with self.assertRaises(RuntimeError):
            self.parse(self.source('pub fn selector_only(skill:u16)->bool {matches!(skill,101)}'))
        with self.assertRaises(RuntimeError):
            self.parse(self.source('pub fn selector_only(skill:u16)->bool { skill == 101 }',
                                   'id if selector_only(id) => {}'))

    def test_text_helper_is_recognized_only_when_wired(self):
        helper = ('pub fn text_skill(skill: u16) -> bool { matches!(skill, 113 | 116 | 118) }\n'
                  'pub fn text_request() {}')
        result = self.parse(self.source(helper, 'id if text_skill(id) => {}'))
        self.assertEqual(result['text_request_ids'], [113, 116, 118])
        self.assertEqual(result['all_request_ids'], [1, 4, 58, 63, 113, 116, 118])
        with self.assertRaises(RuntimeError):
            self.parse(self.source(helper))
        with self.assertRaises(RuntimeError):
            self.parse(self.source('pub fn text_skill(skill:u16)->bool { skill == 113 }',
                                   'id if text_skill(id) => {}'))

    def test_current_source_covers_second_batch(self):
        result = CATALOG.source_ids(ROOT / 'LiTo381-Modern/src/modern_lito/native_skills.rs')
        self.assertTrue(set(range(110, 119)) <= set(result['all_request_ids']))
        self.assertEqual(result['text_request_ids'], [113, 116, 118])
        self.assertTrue({110, 111, 112, 114, 115, 117}
                        <= set(result['actor_or_selector_request_ids']))

    def test_current_source_covers_third_batch(self):
        result = CATALOG.source_ids(ROOT / 'LiTo381-Modern/src/modern_lito/native_skills.rs')
        batch = {129, 130, 132, 133, 134, 137, 138}
        self.assertTrue(batch <= set(result['actor_or_selector_request_ids']))
        self.assertFalse(batch & set(result['text_request_ids']))
        self.assertFalse(batch & set(result['inventory_request_ids']))
        self.assertFalse(batch & set(result['ground_request_ids']))
        # 135／136 are outside this batch and must not be reported as wired.
        self.assertFalse({135, 136} & set(result['all_request_ids']))

    def test_current_source_covers_fourth_batch(self):
        result = CATALOG.source_ids(ROOT / 'LiTo381-Modern/src/modern_lito/native_skills.rs')
        self.assertTrue(set(range(145, 156)) <= set(result['actor_or_selector_request_ids']))
        self.assertFalse(set(range(145, 156)) & set(result['inventory_request_ids']))
        self.assertFalse(set(range(145, 156)) & set(result['text_request_ids']))

    def test_current_source_covers_legacy_and_first_batch(self):
        result = CATALOG.source_ids(ROOT / 'LiTo381-Modern/src/modern_lito/native_skills.rs')
        previous = {1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 13, 14, 15, 16, 17, 18, 19,
                    26, 28, 31, 32, 34, 35, 38, 42, 43, 49, 52, 54, 57, 58, 60,
                    63, 72, 77, 78, 131, 206}
        self.assertTrue((previous | set(range(101, 110))) <= set(result['all_request_ids']))
        self.assertIn(107, result['inventory_request_ids'])
        self.assertNotIn(107, result['actor_or_selector_request_ids'])
        self.assertTrue(set(result['all_request_ids']) <= set(range(1, 221)))


if __name__ == '__main__':
    unittest.main()
