#!/usr/bin/env python3
"""Gera manual estático sem dependências externas; links relativos ao projeto."""
import argparse
import html
import pathlib
import re
import shutil
import unicodedata

ROOT = pathlib.Path(__file__).resolve().parents[1]

def inline(text):
    text = html.escape(text)
    text = re.sub(r'`([^`]+)`', r'<code>\1</code>', text)
    text = re.sub(r'\[([^\]]+)\]\((https://[^)]+)\)', r'<a href="\2">\1</a>', text)
    return re.sub(r'\*\*([^*]+)\*\*', r'<strong>\1</strong>', text)

def render(source):
    output, toc, paragraph, code = [], [], [], None
    listing = False
    def flush():
        if paragraph:
            output.append('<p>' + inline(' '.join(paragraph)) + '</p>')
            paragraph.clear()
    for line in source.splitlines():
        if code is None and line.startswith('<!-- ') and line.endswith(' -->'):
            continue
        if line.startswith('```'):
            flush()
            if code is None:
                code = []
            else:
                output.append('<pre><code>' + html.escape('\n'.join(code)) + '</code></pre>')
                code = None
            continue
        if code is not None:
            code.append(line)
            continue
        if listing and not line.startswith('- '):
            output.append('</ul>')
            listing = False
        if line.startswith('# '):
            flush()
            output.append('<h1>' + inline(line[2:]) + '</h1>')
        elif line.startswith('## '):
            flush()
            label = line[3:]
            slug = re.sub(r'[^a-z0-9]+', '-', unicodedata.normalize('NFKD', label).encode('ascii', 'ignore').decode().lower()).strip('-')
            toc.append(f'<a href="#{slug}">{inline(label)}</a>')
            if len(toc) > 1:
                output.append('</section>')
            output.append(f'<section id="{slug}"><h2>{inline(label)}</h2>')
        elif line.startswith('### '):
            flush()
            output.append('<h3>' + inline(line[4:]) + '</h3>')
        elif line.startswith('- '):
            flush()
            if not listing:
                output.append('<ul>')
                listing = True
            output.append('<li>' + inline(line[2:]) + '</li>')
        elif not line:
            flush()
        else:
            paragraph.append(line)
    flush()
    if listing:
        output.append('</ul>')
    if toc:
        output.append('</section>')
    return '\n'.join(output), '\n'.join(toc)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=pathlib.Path, default=ROOT / 'target/site')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    for name in ['index.html', 'assets']:
        source, target = ROOT / 'site' / name, args.output / name
        if source.is_dir():
            shutil.copytree(source, target, dirs_exist_ok=True)
        else:
            shutil.copy2(source, target)
    content, toc = render((ROOT / 'docs/MANUAL.md').read_text())
    template = (ROOT / 'site/manual-template.html').read_text()
    (args.output / 'manual.html').write_text(template.replace('{{toc}}', toc).replace('{{content}}', content))
    (args.output / '.nojekyll').touch()
    print(args.output)

if __name__ == '__main__':
    main()
