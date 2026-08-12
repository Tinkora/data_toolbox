import { expect, test } from '@playwright/test';

const runtimeFailures = new WeakMap();

test.beforeEach(async ({ page }) => {
  const failures = [];
  runtimeFailures.set(page, failures);

  page.on('console', (message) => {
    if (message.type() === 'error') failures.push(`console: ${message.text()}`);
  });
  page.on('pageerror', (error) => failures.push(`page: ${error.message}`));
  page.on('request', (request) => {
    const url = new URL(request.url());
    if (url.hostname !== '127.0.0.1') failures.push(`external request: ${url}`);
  });

  await page.goto('/web/');
  await expect(page.getByRole('status')).toHaveText('Ready');
});

test.afterEach(async ({ page }) => {
  expect(runtimeFailures.get(page)).toEqual([]);
});

test('initializes real WASM without horizontal page overflow', async ({ page }) => {
  const dimensions = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(dimensions.scrollWidth).toBeLessThanOrEqual(dimensions.clientWidth);
  await expect(page.getByRole('heading', { name: 'CSV Inspector' })).toBeVisible();
});

test('renders adversarial CSV as text', async ({ page }) => {
  await page.getByLabel('Delimiter').selectOption('comma');
  await page
    .getByLabel('CSV or TSV input')
    .fill('name\n<img src=x onerror=window.__executedMarkup=true>\n');
  await page.getByRole('button', { name: 'Inspect data' }).click();

  await expect(
    page.getByRole('cell', { name: '<img src=x onerror=window.__executedMarkup=true>' }),
  ).toBeVisible();
  expect(await page.evaluate(() => window.__executedMarkup)).toBeUndefined();
});

test('loads a local file and reports formula-like cells', async ({ page }) => {
  await page.getByLabel('Load file').setInputFiles({
    name: 'contacts.csv',
    mimeType: 'text/csv',
    buffer: Buffer.from('name,email\nAlice,=1+1\n'),
  });

  await expect(page.getByLabel('CSV or TSV input')).toHaveValue('name,email\nAlice,=1+1\n');
  await page.getByRole('button', { name: 'Inspect data' }).click();
  await expect(page.getByText('FORMULA_LIKE_CELL', { exact: true })).toBeVisible();
  await expect(page.getByText('1', { exact: true }).first()).toBeVisible();
});

test('rejects an oversized file before reading it', async ({ page }) => {
  await page.getByLabel('Load file').setInputFiles({
    name: 'too-large.csv',
    mimeType: 'text/csv',
    buffer: Buffer.alloc((10 * 1024 * 1024) + 1, 97),
  });

  await expect(page.getByRole('alert')).toContainText('INPUT_TOO_LARGE');
  await expect(page.getByLabel('CSV or TSV input')).toHaveValue('');
});

test('rejects non-UTF-8 files without replacement characters', async ({ page }) => {
  await page.getByLabel('Load file').setInputFiles({
    name: 'invalid.csv',
    mimeType: 'text/csv',
    buffer: Buffer.from([0x6e, 0x61, 0x6d, 0x65, 0x0a, 0xc3, 0x28]),
  });

  await expect(page.getByRole('alert')).toContainText('INVALID_UTF8');
  await expect(page.getByLabel('CSV or TSV input')).toHaveValue('');
});

test('shows malformed CSV as a stable error', async ({ page }) => {
  await page.getByLabel('Delimiter').selectOption('comma');
  await page.getByLabel('CSV or TSV input').fill('a,b\n\"broken\n');
  await page.getByRole('button', { name: 'Inspect data' }).click();

  await expect(page.getByRole('alert')).toContainText('INVALID_CSV');
});

test('exports preserved and spreadsheet-safe CSV explicitly', async ({ page }) => {
  await page.getByLabel('Delimiter').selectOption('comma');
  await page.getByLabel('CSV or TSV input').fill('value\n=1+1\n');
  await page.getByRole('button', { name: 'Inspect data' }).click();

  await page.getByRole('button', { name: 'Create export' }).click();
  await expect(page.getByLabel('Export output')).toHaveValue('value\n=1+1\n');

  await page.getByLabel('Formula policy').selectOption('escape_for_spreadsheet');
  await page.getByRole('button', { name: 'Create export' }).click();
  await expect(page.getByLabel('Export output')).toHaveValue("value\n'=1+1\n");
});

test('keeps JSON formula text preserved', async ({ page }) => {
  await page.getByLabel('Delimiter').selectOption('comma');
  await page.getByLabel('CSV or TSV input').fill('value\n=1+1\n');
  await page.getByRole('button', { name: 'Inspect data' }).click();
  await page.getByLabel('Formula policy').selectOption('escape_for_spreadsheet');
  await page.getByLabel('Format').selectOption('json');

  await expect(page.getByLabel('Formula policy')).toBeDisabled();
  await expect(page.getByLabel('Formula policy')).toHaveValue('preserve');
  await page.getByRole('button', { name: 'Create export' }).click();
  await expect(page.getByLabel('Export output')).toHaveValue('[{"value":"=1+1"}]');
});

test('exposes a visible keyboard focus indicator', async ({ page }) => {
  await page.keyboard.press('Tab');
  const focus = await page.evaluate(() => {
    const active = document.activeElement;
    const style = getComputedStyle(active);
    return {
      text: active.textContent?.trim(),
      outlineStyle: style.outlineStyle,
      outlineWidth: style.outlineWidth,
    };
  });

  expect(focus.text).toBe('Skip to inspector');
  expect(focus.outlineStyle).not.toBe('none');
  expect(Number.parseFloat(focus.outlineWidth)).toBeGreaterThanOrEqual(2);
});
