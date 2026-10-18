import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["ui/**/*.test.ts", "scripts/**/*.test.ts", "tests/**/*.test.ts", "extensions/**/*.test.ts"],
  },
});
