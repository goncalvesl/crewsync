const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const fs = require('node:fs');
const { spawnSync } = require('node:child_process');
const assert = require('node:assert/strict');
const settings = JSON.parse(fs.readFileSync(process.env.CREWSYNC_ADMIN_FIXTURE, 'utf8'));
(async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.goto(settings.url);
    await page.getByLabel('Chave administrativa').fill(settings.key);
    await Promise.all([page.waitForURL(settings.url + '/'), page.getByRole('button', { name: 'Entrar no painel' }).click()]);
    await page.locator('#user-form').waitFor({timeout: 5000}).catch(async error => { console.error('Página após login:', await page.locator('body').innerText(), errors); throw error; });
    await page.locator('#user-form input[name=user]').fill('bruno');
    await page.locator('#user-form button').click();
    await page.locator('tr[data-user=bruno]').waitFor();
    await page.locator('#share-form input[name=share]').fill('Projetos');
    await page.locator('#share-form button').click();
    await page.locator('tr[data-share=Projetos]').waitFor();
    const permissions = page.locator('#permission-form');
    await permissions.locator('select[name=share]').selectOption('Projetos');
    await permissions.locator('select[name=user]').selectOption('bruno');
    await permissions.locator('select[name=access]').selectOption('write');
    await permissions.getByRole('button').click();
    assert.match(await page.locator('tr[data-share=Projetos]').innerText(), /bruno: leitura e escrita/);
    await page.locator('#invite-form select[name=user]').selectOption('bruno');
    await page.locator('#invite-form button').click();
    const invite = await page.locator('#invite-code').inputValue();
    assert.match(invite, /^[0-9a-f]{64}$/);
    const activate = spawnSync(settings.client, ['activate', settings.url, 'insecure-http', settings.session, 'Computador de Bruno'], { input: invite + '\n', encoding: 'utf8' });
    assert.equal(activate.status, 0, activate.stderr);
    await page.getByRole('link', { name: 'Voltar aos dispositivos' }).click();
    const device = page.locator('tr[data-device]').filter({ hasText: 'Computador de Bruno' });
    await device.waitFor();
    for (const width of [390, 768, 1440]) {
      await page.setViewportSize({ width, height: 1000 });
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, 'overflow ' + width);
      assert.equal(await page.locator('h1').count(), 1);
      await page.screenshot({ path: 'target/admin-' + width + '.png', fullPage: true });
    }
    await device.getByRole('button', { name: 'Revogar dispositivo' }).click();
    assert.match(await page.locator('tr[data-device]').filter({ hasText: 'Computador de Bruno' }).innerText(), /Revogado/);
    const blocked = spawnSync(settings.client, ['shares', settings.session], { encoding: 'utf8' });
    assert.notEqual(blocked.status, 0);
    await page.getByRole('button', { name: 'Sair', exact: true }).click();
    await page.getByRole('button', { name: 'Entrar no painel' }).waitFor();
    assert.deepEqual(errors, []);
    console.log('Painel: login, usuários, pastas, permissões, convite, dispositivo, revogação, logout e telas 390/768/1440 OK');
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
