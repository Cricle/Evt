import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import { createHmac } from 'node:crypto';

const SEED_ADMIN = {
  username: 'evt_playwright_admin',
  password: 'Passw0rd_123',
};

let seedAdminToken: string | null = null;

function base64UrlJson(value: unknown) {
  return Buffer.from(JSON.stringify(value)).toString('base64url');
}

function issueJwtToken(payload: Record<string, unknown>) {
  const header = base64UrlJson({ alg: 'HS256', typ: 'JWT' });
  const body = base64UrlJson(payload);
  const data = `${header}.${body}`;
  const secret = 'replace-with-a-long-random-secret';
  const signature = createHmac('sha256', secret).update(data).digest('base64url');
  return `${data}.${signature}`;
}

function installErrorCollector(page: Page) {
  const runtimeErrors: string[] = [];

  page.on('pageerror', (error) => {
    runtimeErrors.push(`pageerror: ${error.message}`);
  });

  page.on('console', (message) => {
    if (message.type() === 'error') {
      runtimeErrors.push(`console: ${message.text()}`);
    }
  });

  return runtimeErrors;
}

async function expectNoRuntimeErrors(errors: string[], label: string) {
  expect(errors, label).toEqual([]);
}

async function primeClientState(page: Page, token?: string | null) {
  await page.addInitScript(
    ([localeKey, localeValue, tokenKey, tokenValue]) => {
      window.localStorage.setItem(localeKey, localeValue);
      if (tokenValue) {
        window.localStorage.setItem(tokenKey, tokenValue);
      } else {
        window.localStorage.removeItem(tokenKey);
      }
    },
    ['EVT_LOCALE', 'zh-CN', 'EVT_TOKEN', token ?? null],
  );
}

async function ensureSeedAdmin(request: APIRequestContext) {
  if (seedAdminToken) {
    return { ...SEED_ADMIN, token: seedAdminToken };
  }

  let login = await request.post('/v1/auth/login', {
    data: SEED_ADMIN,
  });

  if (!login.ok()) {
    const register = await request.post('/v1/auth/register', {
      data: SEED_ADMIN,
    });
    expect(register.ok()).toBeTruthy();

    login = await request.post('/v1/auth/login', {
      data: SEED_ADMIN,
    });
  }

  expect(login.ok()).toBeTruthy();
  const loginBody = await login.json();
  seedAdminToken = loginBody.data.token as string;

  return { ...SEED_ADMIN, token: seedAdminToken };
}

async function resolveAdminSession(request: APIRequestContext) {
  const devAdminToken = issueJwtToken({
    sub: '1',
    uid: 1,
    username: 'ggg',
    iss: 'evt',
    exp: Math.floor(Date.now() / 1000) + 86_400,
  });

  for (const token of [devAdminToken, seedAdminToken]) {
    if (!token) continue;
    const currentUser = await request.get('/v1/users/me', {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    if (!currentUser.ok()) continue;
    const body = await currentUser.json();
    if (body?.data?.is_admin) {
      return {
        username: body.data.username as string,
        token,
      };
    }
  }

  const seeded = await ensureSeedAdmin(request);
  const currentUser = await request.get('/v1/users/me', {
    headers: {
      Authorization: `Bearer ${seeded.token}`,
    },
  });
  expect(currentUser.ok()).toBeTruthy();
  const body = await currentUser.json();
  if (body?.data?.is_admin) {
    return {
      username: body.data.username as string,
      token: seeded.token,
    };
  }

  throw new Error('failed to resolve an admin session for Playwright');
}

async function registerAndLogin(page: Page) {
  const username = `evt_e2e_${Date.now()}`;
  const password = 'Passw0rd_123';

  const register = await page.request.post('/v1/auth/register', {
    data: { username, password },
  });
  expect(register.ok()).toBeTruthy();

  const login = await page.request.post('/v1/auth/login', {
    data: { username, password },
  });
  expect(login.ok()).toBeTruthy();

  const loginBody = await login.json();
  const token = loginBody.data.token as string;

  await primeClientState(page, token);

  return { username, password, token };
}

test.beforeEach(async ({ page, request }) => {
  await ensureSeedAdmin(request);
  await primeClientState(page, null);
});

async function gotoLanding(page: Page) {
  await page.goto('/#/');
  await expect(page.getByRole('heading', { name: '广场驱动的社区协作' })).toBeVisible();
}

async function gotoSpace(page: Page) {
  await page.goto('/#/space?space=public');
  await expect(page).toHaveURL(/#\/space\?space=public/);
  await expect(page.locator('.nav-title-card')).toBeVisible();
}

async function createPostFromCompose(page: Page, content: string) {
  const composer = page.locator('.ck-editor__editable').first();
  await composer.click();
  await composer.fill(content);

  const createPostResponse = page.waitForResponse((response) => {
    return (
      response.url().includes('/v1/post') &&
      response.request().method() === 'POST'
    );
  });

  await page
    .locator('.compose-editor')
    .getByRole('button', { name: '发布', exact: true })
    .click();

  const response = await createPostResponse;
  expect(response.ok()).toBeTruthy();
  await expect(page).toHaveURL(/#\/post\?id=/);
  await expect(page.getByText(content).first()).toBeVisible();
}

async function createEventFromCompose(page: Page, content: string) {
  const composer = page.locator('.ck-editor__editable').first();
  await composer.click();
  await composer.fill(content);

  const createPostResponse = page.waitForResponse((response) => {
    return response.url().includes('/v1/post') && response.request().method() === 'POST';
  });

  await page
    .locator('.compose-editor')
    .getByRole('button', { name: '创建事件', exact: true })
    .click();

  const response = await createPostResponse;
  expect(response.ok()).toBeTruthy();
  await expect(page).toHaveURL(/#\/post\?id=/);
  await expect(page.getByText('事件时间轴').first()).toBeVisible();
  await expect(page.getByRole('heading', { name: content }).first()).toBeVisible();
}

async function appendEventNode(page: Page, content: string) {
  const composer = page.locator('.event-composer-wrap .n-mention textarea, .event-composer-wrap textarea').first();
  await composer.click();
  await composer.fill(content);

  const createCommentResponse = page.waitForResponse((response) => {
    return response.url().includes('/v1/post/comment') && response.request().method() === 'POST';
  });

  await page
    .locator('.event-composer-wrap')
    .getByRole('button', { name: '添加节点', exact: true })
    .click();

  const response = await createCommentResponse;
  expect(response.ok()).toBeTruthy();
  await expect(page.getByText(content).first()).toBeVisible();
  await expect(page.getByText('#1').first()).toBeVisible();
}

async function goToComposeFromSpace(page: Page) {
  await expect(page.locator('.floating-compose')).toBeVisible();
  const currentUrl = new URL(page.url());
  const expectedSpaceSlug = currentUrl.hash.includes('?')
    ? new URLSearchParams(currentUrl.hash.split('?')[1] || '').get('space') || 'public'
    : 'public';
  await page.locator('.floating-compose').click();
  await expect(page).toHaveURL(new RegExp(`#\\/compose\\?space=${expectedSpaceSlug}`));
  await expect(page.locator('.compose-page')).toBeVisible();
  await expect(page.locator('.ck-editor__editable').first()).toBeVisible();
}

async function createSpaceFromSpace(page: Page, slug: string) {
  await page.getByRole('button', { name: '新建广场' }).click();
  await expect(page).toHaveURL(/#\/spaces\/create\?space=public/);

  await page.getByPlaceholder('例如：设计协作组').fill(`Playwright ${slug}`);
  await page.getByPlaceholder('例如：design-team').fill(slug);
  await page.getByPlaceholder('简单介绍这个广场的用途').fill('playwright scoped space');

  const createSpaceResponse = page.waitForResponse((response) => {
    return response.url().includes('/v1/spaces') && response.request().method() === 'POST';
  });

  await page.getByRole('button', { name: '创建广场', exact: true }).click();
  const response = await createSpaceResponse;
  expect(response.ok()).toBeTruthy();
  await expect(page).toHaveURL(new RegExp(`#\\/space\\?space=${slug}`));
}

test('guest can load landing page and enter public space without runtime errors', async ({ page }) => {
  const errors = installErrorCollector(page);

  await gotoLanding(page);
  await expect(page.locator('#app > *')).toHaveCount(1);
  await page.getByRole('button', { name: '进入公共广场' }).click();
  await expect(page).toHaveURL(/#\/space\?space=public/);
  await expect(page.getByText('space not found')).toHaveCount(0);

  await expectNoRuntimeErrors(errors, 'landing -> space should not crash');
});

test('guest can open the legacy square alias without space errors', async ({ page }) => {
  const errors = installErrorCollector(page);

  await page.goto('/#/space?space=square');
  await expect(page.locator('.nav-title-card')).toBeVisible();
  await expect(page.getByText('space not found')).toHaveCount(0);

  await expectNoRuntimeErrors(errors, 'legacy square alias should not crash');
});

test('direct spa routes normalize without white screen', async ({ page }) => {
  const errors = installErrorCollector(page);
  await registerAndLogin(page);

  await page.goto('/space?space=public');
  await expect(page).toHaveURL(/#\/space\?space=public/);
  await expect(page.locator('.nav-title-card')).toBeVisible();

  await page.goto('/setting');
  await expect(page).toHaveURL(/#\/setting/);
  await expect(page.locator('.setting-card').first()).toBeVisible();

  await expectNoRuntimeErrors(errors, 'direct spa routes should normalize without crashing');
});

test('direct compose route normalizes and renders without white screen', async ({ page }) => {
  const errors = installErrorCollector(page);
  await registerAndLogin(page);

  await page.goto('/compose?space=public');
  await expect(page).toHaveURL(/#\/compose\?space=public/);
  await expect(page.locator('.compose-page')).toBeVisible();
  await expect(page.locator('.ck-editor__editable').first()).toBeVisible();

  await expectNoRuntimeErrors(errors, 'direct compose route should normalize without crashing');
});

test('authenticated user can create a post from the space page', async ({ page }) => {
  const errors = installErrorCollector(page);
  await registerAndLogin(page);

  await gotoSpace(page);
  await goToComposeFromSpace(page);
  await createPostFromCompose(page, `evt playwright post ${Date.now()}`);

  await expectNoRuntimeErrors(errors, 'post compose flow should not crash');
});

test('authenticated user can create an event and land on the timeline detail page', async ({ page }) => {
  const errors = installErrorCollector(page);
  await registerAndLogin(page);

  const content = `evt playwright event ${Date.now()}`;

  await gotoSpace(page);
  await expect(page.locator('.floating-compose')).toBeVisible();
  await page.locator('.floating-compose').click();
  await page.getByText('创建事件', { exact: true }).click();
  await expect(page).toHaveURL(/#\/compose\?mode=event&space=public/);
  await expect(page.getByText('事件时间轴').first()).toBeVisible();
  await expect(page.getByText('先发布事件主题，再持续追加节点').first()).toBeVisible();

  await createEventFromCompose(page, content);
  await expect(page.getByText('追加时间节点').first()).toBeVisible();
  await expect(page.getByText('按最早到最新查看全部节点').first()).toBeVisible();

  await expectNoRuntimeErrors(errors, 'event compose flow should not crash');
});

test('authenticated user can append a timeline node after creating an event', async ({ page }) => {
  const errors = installErrorCollector(page);
  await registerAndLogin(page);

  const eventContent = `evt timeline event ${Date.now()}`;
  const nodeContent = `追加节点 ${Date.now()}`;

  await gotoSpace(page);
  await expect(page.locator('.floating-compose')).toBeVisible();
  await page.locator('.floating-compose').click();
  await page.getByText('创建事件', { exact: true }).click();
  await expect(page).toHaveURL(/#\/compose\?mode=event&space=public/);

  await createEventFromCompose(page, eventContent);
  await appendEventNode(page, nodeContent);
  await expect(page.getByText('时间节点').first()).toBeVisible();
  await expect(page.getByText('1').first()).toBeVisible();

  await expectNoRuntimeErrors(errors, 'event node append flow should not crash');
});

test('floating compose entry is only visible on the space page', async ({ page }) => {
  const errors = installErrorCollector(page);
  await registerAndLogin(page);

  await gotoLanding(page);
  await expect(page.locator('.floating-compose')).toHaveCount(0);

  await page.goto('/#/setting');
  await expect(page.locator('.floating-compose')).toHaveCount(0);

  await gotoSpace(page);
  await expect(page.locator('.floating-compose')).toBeVisible();

  await expectNoRuntimeErrors(errors, 'floating compose visibility should be route scoped');
});

test('authenticated admin can navigate from admin settings back to settings', async ({ page }) => {
  const errors = installErrorCollector(page);
  const { username, token } = await resolveAdminSession(page.request);
  await primeClientState(page, token);

  await page.goto('/#/admin/settings');
  await expect(page).toHaveURL(/#\/admin\/settings/);
  await expect(page.getByText('系统配置').first()).toBeVisible();

  if (await page.locator('.drawer-btn').isVisible()) {
    await page.locator('.drawer-btn').click();
  }
  await page.getByRole('menuitem', { name: '设置' }).click();
  await expect(page).toHaveURL(/#\/setting/);
  await expect(page.locator('.setting-card').first()).toBeVisible();
  await expect(page.getByText(`@${username}`).first()).toBeVisible();

  await expectNoRuntimeErrors(errors, 'admin settings -> settings navigation should not crash');
});

test('authenticated user theme settings persist across reloads', async ({ page }) => {
  const errors = installErrorCollector(page);
  await registerAndLogin(page);

  await page.goto('/#/setting?t=e2e');
  await expect(page.locator('.setting-card').first()).toBeVisible();

  await page.locator('.theme-setting-block .n-radio').filter({ hasText: '深色' }).click();
  await expect
    .poll(() => page.evaluate(() => document.body.classList.contains('dark')))
    .toBe(true);

  await page.locator('.theme-setting-block').nth(1).locator('.n-base-selection').click();
  await page.getByText('海蓝', { exact: true }).last().click();
  await expect
    .poll(() =>
      page.evaluate(() =>
        getComputedStyle(document.documentElement).getPropertyValue('--accent-primary').trim(),
      ),
    )
    .toBe('#70c0ff');

  await page.reload();
  await expect(page.locator('.setting-card').first()).toBeVisible();
  await expect
    .poll(() => page.evaluate(() => document.body.classList.contains('dark')))
    .toBe(true);
  await expect
    .poll(() =>
      page.evaluate(() =>
        getComputedStyle(document.documentElement).getPropertyValue('--accent-primary').trim(),
      ),
    )
    .toBe('#70c0ff');

  await expectNoRuntimeErrors(errors, 'theme settings should persist without runtime errors');
});

test('authenticated user can create a space and posts stay isolated by space', async ({ page }) => {
  const errors = installErrorCollector(page);
  await registerAndLogin(page);

  const slug = `space-${Date.now()}`;
  const content = `evt scoped post ${Date.now()}`;

  await gotoSpace(page);
  await createSpaceFromSpace(page, slug);
  await goToComposeFromSpace(page);
  await createPostFromCompose(page, content);

  await page.goto('/#/space?space=public');
  await expect(page.getByText(content)).toHaveCount(0);

  await page.goto(`/#/space?space=${slug}`);
  await expect(page.getByText(content).first()).toBeVisible();

  await expectNoRuntimeErrors(errors, 'space isolation flow should not crash');
});

test.use({
  viewport: { width: 390, height: 844 },
});

test('mobile guest layout can open drawer and navigate to settings shell', async ({ page }) => {
  const errors = installErrorCollector(page);

  await gotoLanding(page);
  await expect(page.locator('.drawer-btn')).toBeVisible();
  await page.locator('.drawer-btn').click();
  await expect(page.getByRole('menuitem', { name: '广场' })).toBeVisible();
  await page.getByRole('menuitem', { name: '广场' }).click();
  await expect(page).toHaveURL(/#\/space(?:\?|\?.*&)?(?:t=\d+&)?space=public/);

  await expectNoRuntimeErrors(errors, 'mobile drawer navigation should not crash');
});
