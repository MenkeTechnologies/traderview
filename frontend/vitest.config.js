import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    // Only match .spec.js so we don't double-run the existing node:test
    // .mjs files (those continue to be invoked by scripts/test.sh via
    // `node --test`). New tests should be written as `*.spec.js` using
    // vitest's expect/test API.
    include: ['tests/**/*.spec.js'],
    // The existing tests are pure (no DOM) so node env is correct.
    // When per-file overrides are needed, use the `// @vitest-environment jsdom`
    // directive at the top of that test file.
    environment: 'node',
    globals: false,
    setupFiles: ['tests/setup-i18n.js'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      include: ['js/**/*.js'],
      exclude: ['js/views/**', 'js/lib/**', 'lib/**'],
    },
  },
});
