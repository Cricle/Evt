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
  await page.getByRole('button', { name: /发布动态/ }).first().click();
  await expect(page).toHaveURL(/#\/compose/);
  const composer = page.locator('.ck-editor__editable').first();
  await composer.click();
  await composer.fill(content);
  const createPostResponse = page.waitForResponse((response) => {
    return response.url().includes('/v1/post') && response.request().method() === 'POST';
  });
  await page.locator('.compose-editor').getByRole('button', { name: '发布', exact: true }).click();
  const response = await createPostResponse;
  expect(response.ok()).toBeTruthy();
  await expect(page).toHaveURL(/#\/post\?id=/);
  await expect(page.getByText(content).first()).toBeVisible();
}

async function createSpaceFromHome(
  page: import('@playwright/test').Page,
  name: string,
  slug: string,
  description: string,
) {
  await page.getByRole('button', { name: '新建广场', exact: true }).click();
  await page.getByPlaceholder('广场名称').fill(name);
  await page.getByPlaceholder('slug，例如 design-team').fill(slug);
  await page.getByPlaceholder('简介').fill(description);
  const createSpaceResponse = page.waitForResponse((response) => {
    return response.url().includes('/v1/spaces') && response.request().method() === 'POST';
  });
  await page.getByRole('button', { name: '创建', exact: true }).click();
  const response = await createSpaceResponse;
  expect(response.ok()).toBeTruthy();
  await expect(page).toHaveURL(new RegExp(`space=${slug}`));
}

async function expectAuthenticatedHome(
  page: import('@playwright/test').Page,
) {
  await expect(page.getByRole('button', { name: /发布动态/ }).first()).toBeVisible();
  await expect(page.locator('.floating-compose')).toBeVisible();
}

test('guest can load spa shell and public site profile', async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.removeItem('EVT_TOKEN');
  });
  const siteProfile = await page.request.get('/v1/site/profile');
  expect(siteProfile.ok()).toBeTruthy();
  const profileBody = await siteProfile.json();
  expect(profileBody.code).toBe(0);

  await page.goto('/#/');
  await expect(page.locator('.nav-title-card, .post-item, .empty-wrap').first()).toBeVisible();
  await expect(
    page.locator('.post-item, .empty-wrap').first(),
  ).toBeVisible();
});

test('guest can open the legacy square alias without space errors', async ({ page }) => {
  await page.goto('/#/?space=square');
  await expect(page.locator('.nav-title-card, .post-item, .empty-wrap').first()).toBeVisible();
  await expect(page.getByText('space not found')).toHaveCount(0);
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

  await expect(page.locator('.detail-item')).toBeVisible();
  await expect(page.locator('.comment-title-item')).toHaveText('评论');
  const commentTabs = page.locator('.comment-opts-wrap');
  await commentTabs.getByText('热门', { exact: true }).click();
  await commentTabs.getByText('最新', { exact: true }).click();
  await commentTabs.getByText('推荐', { exact: true }).click();

  const commentBox = page.locator('textarea').last();
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

  await page.goto('/#/setting?t=e2e');
  await expect(page).toHaveURL(/#\/setting/);
  await expect(page.locator('.setting-card').first()).toBeVisible();
  await expect(page.getByText(`@${username}`)).toBeVisible();
});

test('authenticated user can create a space and posts stay isolated by space', async ({ page }) => {
  await registerAndLogin(page);

  const spaceSlug = `space-${Date.now()}`;
  const spaceName = `Playwright ${Date.now()}`;
  const content = `evt scoped post ${Date.now()}`;

  await page.goto('/#/');
  await expectAuthenticatedHome(page);

  await createSpaceFromHome(page, spaceName, spaceSlug, 'playwright scoped space');
  await expect(page).toHaveURL(new RegExp(`space=${spaceSlug}`));

  await createPostFromHome(page, content);

  await page.goto('/#/?space=public');
  await expectAuthenticatedHome(page);
  await expect(page.getByText(content)).toHaveCount(0);

  await page.goto(`/#/?space=${spaceSlug}`);
  await expectAuthenticatedHome(page);
  await expect(page.getByText(content).first()).toBeVisible();
});

test('authenticated user can create a post from the legacy square alias route', async ({ page }) => {
  await registerAndLogin(page);

  await page.goto('/#/?space=square');
  await expectAuthenticatedHome(page);
  await createPostFromHome(page, `evt legacy alias post ${Date.now()}`);
  await expect(page.getByText('space not found')).toHaveCount(0);
});

test.use({
  viewport: { width: 390, height: 844 },
});

test('mobile guest layout can open drawer and navigate', async ({ page }) => {
  await page.goto('/#/');
  await expect(page.locator('.drawer-btn')).toBeVisible();
  await page.locator('.drawer-btn').click();
  await expect(page.getByRole('menuitem', { name: '话题' })).toBeVisible();
  await page.getByRole('menuitem', { name: '话题' }).click();
  await expect(page.getByText('话题', { exact: true }).first()).toBeVisible();
});
