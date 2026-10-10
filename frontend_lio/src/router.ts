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

  if (!to.path.startsWith("/admin")) return true

  // Try to rehydrate session from cookie
  if (!auth.isAuthenticated && !auth.checked) {
    await auth.fetchMe()
  }

  if (!auth.isAuthenticated) return "/login"

  // Settings (email templates) are admin-only, observers can't even read them
  if (to.path.startsWith("/admin/settings") && !auth.isAdmin) return "/admin"

  return true
})

export default router
