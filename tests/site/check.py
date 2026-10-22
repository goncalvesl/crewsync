#!/usr/bin/env python3
"""Links/âncoras locais, metadados e navegação sob /crewsync/, sem rede."""
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlsplit, unquote
import sys

class Page(HTMLParser):
    def __init__(self, path):
        super().__init__()
        self.links, self.ids, self.lang, self.viewport, self.title = [], set(), False, False, False
        self.feed(path.read_text())
        assert self.lang and self.viewport and self.title, path
    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if 'id' in attrs:
            assert attrs['id'] not in self.ids, 'ID duplicado'
            self.ids.add(attrs['id'])
        if tag == 'html': self.lang = attrs.get('lang') == 'pt-BR'
        if tag == 'title': self.title = True
        if tag == 'meta' and attrs.get('name') == 'viewport': self.viewport = True
        if tag == 'img': assert attrs.get('alt') is not None
        if tag == 'a': self.links.append(attrs.get('href', ''))
        if tag in ['link', 'script', 'img']:
            self.links.append(attrs.get('href', attrs.get('src', '')))

root = Path(sys.argv[1]).resolve()
pages = {p: Page(p) for p in root.rglob('*.html')}
assert len(pages) >= 2
for path, page in pages.items():
    assert 'conteudo' in page.ids
    for href in page.links:
        link = urlsplit(href)
        if link.scheme:
            assert link.scheme == 'https', href
            continue
        assert href and not link.path.startswith('/'), f'Caminho quebra projeto /crewsync/: {href}'
        target = (path.parent / unquote(link.path)).resolve() if link.path else path
        if target.is_dir(): target /= 'index.html'
        assert target.is_relative_to(root) and target.exists(), (path, href)
        if link.fragment:
            assert target in pages and unquote(link.fragment) in pages[target].ids, (path, href)
print(f'{len(pages)} páginas: links, âncoras, idioma, viewport e caminhos relativos OK')
