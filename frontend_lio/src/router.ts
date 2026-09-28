import "./assets/main.css"
// eslint-disable-next-line import-x/no-unresolved
import { setupLayouts } from "virtual:vue-layouts"
import { createRouter, createWebHistory } from "vue-router"
import { routes } from "vue-router/auto-routes"

// eslint-disable-next-line import-x/no-cycle
import { useAuth } from "@/composables/api.ts"

const router = createRouter({
  routes: setupLayouts(routes),
  history: createWebHistory(),
})

router.beforeEach(async (to) => {
  const auth = useAuth()

  if (!to.name.startsWith("/admin")) return true
  if (auth.isAuthenticated) return true

  // Try to rehydrate session from cookie
  if (!auth.checked) {
    await auth.fetchMe()
  }

  // After check, still not authenticated
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
  if (!auth.isAuthenticated) {
    return "/login"
  }

  return true
})

export default router
