#!/usr/bin/env python3
"""Regressões do manual gerado: escape de HTML e exemplos sem interpolação."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('build_site', Path(__file__).parents[2] / 'scripts/build-site.py')
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)

class Manual(unittest.TestCase):
    def test_code_and_html_are_escaped_and_anchors_match_accents(self):
        content, toc = build.render('# Manual\n\n## Operação\n\n<script>bad</script>\n\n```bash\necho "$HOME" < file\n```\n')
        self.assertNotIn('<script>', content)
        self.assertIn('&lt;script&gt;', content)
        self.assertIn('id="operacao"', content)
        self.assertIn('href="#operacao"', toc)
        self.assertIn('echo &quot;$HOME&quot; &lt; file', content)

if __name__ == '__main__':
    unittest.main()
