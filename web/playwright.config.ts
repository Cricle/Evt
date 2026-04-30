import { defineConfig, devices } from '@playwright/test';

const httpPort = process.env.EVT_E2E_HTTP_PORT ?? '18008';
const grpcPort = process.env.EVT_E2E_GRPC_PORT ?? '19020';

export default defineConfig({
  testDir: './e2e',
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
  fullyParallel: false,
  retries: 0,
  use: {
    baseURL: `http://127.0.0.1:${httpPort}`,
    trace: 'retain-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: `bash -lc "cd .. && EVT_RS__DATABASE__URL='mysql://evt:evt@127.0.0.1:3306/evt' EVT_RS__SERVER__HTTP__HOST='127.0.0.1' EVT_RS__SERVER__HTTP__PORT='${httpPort}' EVT_RS__SERVER__GRPC__HOST='127.0.0.1' EVT_RS__SERVER__GRPC__PORT='${grpcPort}' EVT_RS__STORAGE__LOCAL_DIR='./custom/data/attachments' cargo run --quiet -p evt-app"`,
    url: `http://127.0.0.1:${httpPort}/healthz`,
    reuseExistingServer: false,
    timeout: 120_000,
    stdout: 'pipe',
    stderr: 'pipe',
  },
});
