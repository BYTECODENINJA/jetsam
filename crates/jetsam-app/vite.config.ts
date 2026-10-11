import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

const host = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process
  ?.env?.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*']
});
