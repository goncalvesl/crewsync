# Site CrewSync

HTML/CSS estáticos; docs/MANUAL.md gera o manual usando scripts/build-site.py.
Sem conta, analytics, formulários, dependências web ou dados do serviço. Não há
hospedagem do servidor de arquivos no GitHub Pages.

```bash
python3 scripts/build-site.py
python3 tests/site/check.py target/site
python3 tests/site/test_build.py
```

A publicação usa .github/workflows/pages.yml. Links relativos funcionam sob
/crewsync/. Fonte de apresentação: index.html; layout: assets/style.css;
manual: manual-template.html. Instruções de navegador em docs/DISTRIBUTION.md.
