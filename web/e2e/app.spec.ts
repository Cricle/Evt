import { expect, test } from '@playwright/test';

async function registerAndLogin(page: import('@playwright/test').Page) {
  const username = `evt_e2e_${Date.now()}`;
  const password = 'Passw0rd_123';
  const base = test.info().project.use.baseURL!;

  const register = await page.request.post(`${base}/v1/auth/register`, {
    data: { username, password },
  });
  expect(register.ok()).toBeTruthy();

  const login = await page.request.post(`${base}/v1/auth/login`, {
    data: { username, password },
  });
  expect(login.ok()).toBeTruthy();

  const loginBody = await login.json();
  const token = loginBody.data.token as string;

  await page.addInitScript(
    ([tokenKey, tokenValue]) => {
      window.localStorage.setItem(tokenKey, tokenValue);
    },
    ['EVT_TOKEN', token],
  );

  return { username, token };
}

async function createPostFromHome(
  page: import('@playwright/test').Page,
  content: string,
) {
  const composer = page.locator('textarea').first();
  await composer.fill(content);
  await page.getByRole('button', { name: '发布' }).first().click();
  await expect(page.getByText('发布成功')).toBeVisible();
  await expect(page.getByText(content).first()).toBeVisible();
}

async function expectAuthenticatedHome(
  page: import('@playwright/test').Page,
) {
  await expect(page.locator('textarea').first()).toBeVisible();
  await expect(page.getByRole('button', { name: '发布' }).first()).toBeVisible();
}

test('guest can load spa shell and public site profile', async ({ page }) => {
  const siteProfile = await page.request.get('/v1/site/profile');
  expect(siteProfile.ok()).toBeTruthy();
  const profileBody = await siteProfile.json();
  expect(profileBody.code).toBe(0);

  await page.goto('/#/');
  await expect(page.locator('#app')).toBeVisible();
  await expect(page.getByText('Evt 广场', { exact: true }).first()).toBeVisible();
  await expect(page.getByRole('button', { name: '登录' }).first()).toBeVisible();
});

test('authenticated user can load profile and create a post from the web app', async ({ page }) => {
  const { username } = await registerAndLogin(page);

  await page.goto('/#/');
  await expectAuthenticatedHome(page);

  await createPostFromHome(page, 'evt playwright post');

  await page.goto('/#/profile');
  await expect(page.getByText(`@${username}`).first()).toBeVisible();
  await expect(page.getByText('evt playwright post')).toBeVisible();
});

test('authenticated user can open post detail and comment through the web app', async ({ page }) => {
  await registerAndLogin(page);

  await page.goto('/#/');
  await expectAuthenticatedHome(page);
  await createPostFromHome(page, 'evt playwright detail post');

  await expect(page.locator('.comment-title-item')).toHaveText('评论');
  const commentTabs = page.locator('.comment-opts-wrap');
  await commentTabs.getByText('热门', { exact: true }).click();
  await commentTabs.getByText('最新', { exact: true }).click();
  await commentTabs.getByText('推荐', { exact: true }).click();

  const commentBox = page.locator('textarea').first();
  const commentComposer = page.locator('.compose-wrap').first();
  await commentBox.fill('evt playwright comment');
  await expect(commentComposer.getByRole('button', { name: '发布' })).toBeVisible();
  const createCommentResponse = page.waitForResponse((response) => {
    return response.url().includes('/v1/post/comment') && response.request().method() === 'POST';
  });
  await commentComposer.getByRole('button', { name: '发布' }).click();
  const response = await createCommentResponse;
  expect(response.ok()).toBeTruthy();
  await expect(page.getByText('发布成功').first()).toBeVisible();
  await page.reload();
  await expect(page.locator('.comment-item').getByText('evt playwright comment').first()).toBeVisible();
});

test('authenticated user can open settings page', async ({ page }) => {
  const { username } = await registerAndLogin(page);

  await page.goto('/#/');
  await expectAuthenticatedHome(page);

  await page.goto('/#/setting');
  await expect(page.getByText('设置', { exact: true }).first()).toBeVisible();
  await expect(page.getByText('基本信息')).toBeVisible();
  await expect(page.getByText(`@${username}`).first()).toBeVisible();
});

test.use({
  viewport: { width: 390, height: 844 },
});

test('mobile guest layout can open drawer and navigate', async ({ page }) => {
  await page.goto('/#/');
  await expect(page.getByText('Evt 广场', { exact: true }).first()).toBeVisible();
  await page.locator('.drawer-btn').click();
  await expect(page.getByRole('menuitem', { name: '话题' })).toBeVisible();
  await page.getByRole('menuitem', { name: '话题' }).click();
  await expect(page.getByText('话题', { exact: true }).first()).toBeVisible();
});
