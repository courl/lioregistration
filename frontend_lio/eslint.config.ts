import prettierConfig from "eslint-config-prettier"
import { flatConfigs as import_x_configs } from "eslint-plugin-import-x"
import { configs as perfectionist_configs } from "eslint-plugin-perfectionist"
import pluginPromise from "eslint-plugin-promise"
import * as regexpPlugin from "eslint-plugin-regexp"
import { configs as sonar_configs } from "eslint-plugin-sonarjs"
import unicorn from "eslint-plugin-unicorn"
import pluginVue from "eslint-plugin-vue"
import { defineConfig } from "eslint/config"
import { configs as ts_configs, parser as ts_parser } from "typescript-eslint"
import * as vueParser from "vue-eslint-parser"

export default defineConfig(
  // ─── Ignores ────────────────────────────────────────────────────────────
  {
    ignores: ["dist/**", "coverage/**", "node_modules/**", "*.min.*"],
  },

  // ─── Base rule sets (order matters: later overrides earlier) ───────────
  ...pluginVue.configs["flat/recommended"],
  ...ts_configs.strictTypeChecked,
  ...ts_configs.stylisticTypeChecked,
  sonar_configs.recommended,
  unicorn.configs.recommended,
  import_x_configs.recommended,
  import_x_configs.typescript,
  pluginPromise.configs["flat/recommended"],
  regexpPlugin.configs["flat/recommended"],
  perfectionist_configs["recommended-natural"],

  // ─── Global calibrations (all file types) ──────────────────────────────
  {
    rules: {
      // Perfectionist: keep import/export sorting, relax aggressive object sorting
      "perfectionist/sort-objects": "off",
      "perfectionist/sort-modules": "off",
      "perfectionist/sort-interfaces": "off",
      "perfectionist/sort-object-types": "off",
      "perfectionist/sort-union-types": "off",
      "perfectionist/sort-classes": "off",
      "unicorn/no-abusive-eslint-disable": "off",
    },
  },

  // ─── Parser wiring: typed linting inside .vue and .ts ──────────────────
  {
    files: ["**/*.{ts,mts,cts,tsx,vue}"],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: ts_parser,
        projectService: {
          allowDefaultProject: ["eslint.config.ts"],
        },
        extraFileExtensions: [".vue"],
      },
    },
  },

  // ─── Project rules: strict by default ───────────────────────────────────
  {
    files: ["**/*.{ts,mts,cts,tsx,vue}"],
    rules: {
      // Vue: enforce modern, uniform component style
      "vue/no-undef-components": "off",
      "vue/component-api-style": ["error", ["script-setup"]],
      "vue/block-order": ["error", { order: ["script", "template", "style"] }],
      "vue/define-macros-order": [
        "error",
        {
          order: [
            "defineOptions",
            "defineModel",
            "defineProps",
            "defineEmits",
            "defineSlots",
          ],
        },
      ],
      "vue/define-emits-declaration": ["error", "type-based"],
      "vue/define-props-declaration": ["error", "type-based"],
      "vue/require-typed-ref": "error",
      "vue/no-unused-properties": [
        "error",
        { groups: ["props", "data", "computed", "methods"] },
      ],
      "vue/no-unused-refs": "error",
      "vue/no-undef-properties": "error",
      "vue/no-ref-object-reactivity-loss": "error",
      "vue/no-setup-props-reactivity-loss": "error",
      "vue/no-required-prop-with-default": "error",
      "vue/no-useless-v-bind": "error",
      "vue/no-useless-mustaches": "error",
      "vue/prefer-true-attribute-shorthand": "error",
      "vue/component-name-in-template-casing": ["error", "PascalCase"],
      "vue/custom-event-name-casing": ["error", "camelCase"],
      "vue/v-on-handler-style": "off",
      "vue/padding-line-between-blocks": "error",
      "vue/multi-word-component-names": "off",

      // TypeScript: tighten beyond strictTypeChecked
      "@typescript-eslint/consistent-type-imports": [
        "error",
        { fixStyle: "inline-type-imports", prefer: "type-imports" },
      ],
      "@typescript-eslint/consistent-type-exports": "error",
      "@typescript-eslint/explicit-function-return-type": "off",
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/switch-exhaustiveness-check": "error",
      "@typescript-eslint/no-unnecessary-condition": "error",
      "@typescript-eslint/strict-boolean-expressions": [
        "error",
        { allowNullableBoolean: true, allowNullableString: false },
      ],
      "@typescript-eslint/prefer-readonly": "error",
      "@typescript-eslint/promise-function-async": "error",
      "@typescript-eslint/require-array-sort-compare": "error",
      "@typescript-eslint/method-signature-style": ["error", "property"],

      // Imports: hygiene + no cycles caught early (dependency-cruiser is the
      // authoritative architecture check; this catches cycles in-editor)
      "import-x/no-cycle": ["error", { maxDepth: 4 }],
      "import-x/no-self-import": "error",
      "import-x/no-useless-path-segments": "error",
      "import-x/no-extraneous-dependencies": "error",
      "import-x/first": "error",
      "import-x/newline-after-import": "error",
      "import-x/no-duplicates": "error",
      "import-x/no-default-export": "off", // Vue SFCs & configs need them

      // Unicorn: calibrate the noisy ones
      "unicorn/filename-case": [
        "error",
        {
          cases: { kebabCase: true, pascalCase: true }, // PascalCase for SFCs, kebab for the rest
        },
      ],
      "unicorn/prevent-abbreviations": [
        "error",
        {
          replacements: {
            props: false,
            ref: false,
            params: false,
            env: false,
            dev: false,
            prod: false,
            args: false,
          },
        },
      ],
      "unicorn/no-null": "off", // null is idiomatic in Vue/DOM APIs
      "unicorn/prefer-ternary": "off",

      // General anti-footguns
      "no-console": ["error", { allow: ["warn", "error"] }],
      "no-debugger": "error",
      eqeqeq: ["error", "always", { null: "ignore" }],
      "prefer-template": "error",
      "object-shorthand": "error",
    },
  },
  // ─── Tests: pragmatic relaxations ───────────────────────────────────────
  {
    files: ["**/*.{test,spec}.{ts,tsx}", "tests/**"],
    rules: {
      "@typescript-eslint/no-non-null-assertion": "off",
      "@typescript-eslint/no-unsafe-assignment": "off",
      "sonarjs/no-duplicate-string": "off",
      "unicorn/consistent-function-scoping": "off",
    },
  },

  // ─── Plain JS files: no type information available ─────────────────────
  {
    files: ["**/*.{js,mjs,cjs}"],
    extends: [ts_configs.disableTypeChecked],
  },

  // ─── Prettier: must be last, disables all stylistic conflicts ──────────
  prettierConfig,
)
