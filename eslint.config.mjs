import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["dist/", ".cache/", "src-tauri/", "node_modules/", "test-results/"] },
  js.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: { allowDefaultProject: ["eslint.config.mjs"] },
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    files: ["eslint.config.mjs"],
    ...tseslint.configs.disableTypeChecked,
  },
  {
    // Extensions are plain browser JavaScript (and the SDK's global `littleJoan`); tsc checks
    // them (checkJs), including every name they use, so ESLint's own undefined-name rule — which
    // knows no browser globals without another package — is left to it, as for the TS files.
    files: ["extensions/**/*.js"],
    rules: { "no-undef": "off" },
  },
);
