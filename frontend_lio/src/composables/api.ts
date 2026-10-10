import { ofetch } from "ofetch"
import { defineStore } from "pinia"
import { computed, readonly, ref } from "vue"

import type { User } from "@/types/user.ts"

// eslint-disable-next-line import-x/no-cycle
import router from "@/router"
import CalendarDate from "@/types/calendar-date.ts"

function navigateTo(route: string) {
  router.push(route).catch(() =>
    useToast().add({
      title: "Uh oh! Something went wrong.",
      description: "Navigation failed",
      icon: "i-lucide-triangle-alert",
    }),
  )
}

function reviveDates(key: string, value: unknown) {
  if (
    typeof value === "string" &&
    (key === "birthdate" ||
      key === "day" ||
      key.endsWith("_date") ||
      key.endsWith("_at"))
  ) {
    return CalendarDate.parse(value)
  }
  return value
}

function errorMessage(data: unknown): string {
  if (typeof data === "object" && data !== null && "error" in data) {
    return String(data.error)
  }
  return typeof data === "string" && data ? data : "Something failed"
}

const createAPI = (toastOn404: boolean) => {
  return ofetch.create({
    // Same origin by default: the backend serves the SPA in production, Vite proxies /api in dev.
    baseURL:
      (import.meta.env.VITE_API_BASE_URL as string | undefined) ?? "/api",
    credentials: "include",
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    parseResponse: (txt) => (txt ? JSON.parse(txt, reviveDates) : txt),
    onResponseError({ request, response }) {
      if (response.status === 401) {
        // Login and session rehydration handle their own 401s.
        const url = typeof request === "string" ? request : request.url
        if (url.endsWith("/admin/login") || url.endsWith("/admin/me")) return
        useAuth().clear()
        navigateTo("/login")
        return
      }
      if (!toastOn404 && response.status === 404) return
      useToast().add({
        title:
          response.status === 403
            ? "You don't have permission to do this."
            : `Uh oh! Something went wrong (${response.status.toString()}).`,
        description:
          response.status === 403 ? undefined : errorMessage(response._data),
        icon: "i-lucide-triangle-alert",
        color: "error",
      })
    },
  })
}

export const api = createAPI(true)
export const apiWith404Passthrough = createAPI(false)

export const useAuth = defineStore("auth", () => {
  const user = ref<User | null>(null)
  const checked = ref(false)
  const pending = ref(false)

  const isAuthenticated = computed(() => user.value !== null)
  const isAdmin = computed(() => user.value?.role === "admin")

  const login = async (email: string, password: string) => {
    pending.value = true
    try {
      user.value = await api<User>("/admin/login", {
        method: "POST",
        body: { email, password },
      })
    } catch (error) {
      user.value = null
      throw error
    } finally {
      checked.value = true
      pending.value = false
    }
  }

  const fetchMe = async () => {
    if (checked.value) return
    pending.value = true
    try {
      user.value = await api<User>("/admin/me")
    } catch {
      user.value = null
    } finally {
      checked.value = true
      pending.value = false
    }
  }

  const logout = async () => {
    try {
      await api("/admin/logout", { method: "POST" })
    } catch {
      // Already toasted, the local session is cleared regardless.
    } finally {
      clear()
      navigateTo("/login")
    }
  }

  const clear = () => {
    user.value = null
    checked.value = false
    pending.value = false
  }

  return {
    user: readonly(user),
    checked: readonly(checked),
    pending: readonly(pending),
    isAuthenticated,
    isAdmin,
    login,
    fetchMe,
    logout,
    clear,
  }
})
