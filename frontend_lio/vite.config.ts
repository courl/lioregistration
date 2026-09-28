import ui from "@nuxt/ui/vite"
import vue from "@vitejs/plugin-vue"
import { fileURLToPath } from "node:url"
import VueLayouts from "unplugin-vue-layouts"
import { defineConfig } from "vite"
import vitePluginChecker from "vite-plugin-checker"
import VueRouter from "vue-router/vite"

export default defineConfig({
  plugins: [
    VueRouter(),
    vue(),
    VueLayouts(),
    ui({
      ui: {
        colors: {
          primary: "orange",
          neutral: "zinc",
        },
      },
    }),
    vitePluginChecker({
      vueTsc: true,
      eslint: {
        lintCommand: "eslint .",
        useFlatConfig: true,
      },
      stylelint: {
        lintCommand: 'stylelint "src/**/*.{css,scss,vue}"',
      },
    }),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("src", import.meta.url)),
    },
  },
})
