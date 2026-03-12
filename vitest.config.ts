import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov', 'json-summary'],
      include: ['src/lib/**/*.ts'],
      exclude: ['src/lib/types/**', '**/*.test.ts'],
    },
  },
  resolve: {
    alias: {
      $lib: new URL('./src/lib', import.meta.url).pathname,
    },
  },
});
