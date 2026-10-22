// PLAYWRIGHT_MODULE aponta para instalação de QA fora do repositório.
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const assert = require('node:assert/strict');
(async () => {
  const base = (process.env.SITE_URL || 'http://127.0.0.1:8765/crewsync/').replace(/\/?$/, '/');
  const deadline = Date.now() + 10000;
  while (true) {
    try { const response = await fetch(base); if (response.ok) break; } catch {}
    if (Date.now() > deadline) throw new Error('Servidor de preview indisponível');
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  const browser = await chromium.launch({ headless: true });
  try {
    for (const width of [390, 768, 1440]) {
      const page = await browser.newPage({ viewport: { width, height: 1000 } });
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      for (const name of ['', 'manual.html']) {
        const response = await page.goto(`${base}${name}`);
        assert.equal(response.status(), 200);
        assert.equal(await page.locator('h1').count(), 1);
        assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, `overflow: ${width}/${name}`);
        for (const link of await page.locator('a[href^="#"]').all()) {
          const href = await link.getAttribute('href');
          assert.equal(await page.locator(href).count(), 1);
        }
      }
      await page.goto(base);
      await page.getByRole('link', { name: 'Começar o piloto' }).click();
      assert.match(page.url(), /manual.html$/);
      assert.deepEqual(errors, []);
      await page.goto(base);
      await page.screenshot({ path: `target/site-${width}.png`, fullPage: true });
      await page.close();
    }
    console.log('Desktop/tablet/mobile: navegação, âncoras, overflow e console OK');
  } finally { await browser.close(); }
})();
