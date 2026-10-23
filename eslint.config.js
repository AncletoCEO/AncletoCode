import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import prettier from "eslint-config-prettier";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: [
      "dist",
      "node_modules",
      "target",
      "src-tauri/target",
      "src-tauri/gen",
      "third_party",
      "src-tauri/resources/ancleto",
      "src-tauri/binaries",
      ".cache",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2022,
      globals: globals.browser,
    },
    plugins: {
      "react-hooks": reactHooks,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
    },
  },
  {
    files: ["scripts/**/*.mjs", "packages/**/*.{mjs,js}"],
    languageOptions: {
      ecmaVersion: 2022,
      globals: globals.node,
    },
  },
  prettier,
);
