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

//TODO: include access rights

const createAPI = (toastOn404: boolean) => {
  return ofetch.create({
    baseURL: import.meta.env.VITE_API_BASE_URL as string,
    credentials: "include",
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    parseResponse: (txt) => (txt ? JSON.parse(txt, reviveDates) : txt),
    onResponseError({ response }) {
      if (response.status === 401) {
        const { clear } = useAuth()
        clear()
        navigateTo("/login")
      } else if (!toastOn404 && response.status === 404) {
      } else {
        response
          .text()
          .then((value) => {
            useToast().add({
              title: `Uh oh! Something went wrong ${response.status.toString()}.`,
              description: value,
              icon: "i-lucide-triangle-alert",
            })
            return null
          })
          .catch(() => {
            useToast().add({
              title: `Uh oh! Something went wrong ${response.status.toString()}.`,
              description: "Something failed",
              icon: "i-lucide-triangle-alert",
            })
          })
      }
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
    login,
    fetchMe,
    logout,
    clear,
  }
})
