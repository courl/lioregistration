import "./assets/main.css"

import ui from "@nuxt/ui/vue-plugin"
import { createPinia } from "pinia"
import { createApp } from "vue"

import router from "@/router"

import App from "./App.vue"

const pinia = createPinia()

const app = createApp(App)

app.use(router)
app.use(ui)
app.use(pinia)

app.mount("#app")
