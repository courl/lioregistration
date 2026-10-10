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
  server: {
    // Same origin as the SPA, so the SameSite=Lax auth cookie is sent in dev too.
    proxy: {
      "/api": "http://127.0.0.1:3001",
    },
  },
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("src", import.meta.url)),
    },
  },
})
