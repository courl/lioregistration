import { type MaybeRefOrGetter, onMounted, ref, toValue, watch } from "vue"

import type { ActiveEdition } from "@/types/edition.ts"
import type { NewRegistration } from "@/types/registration.ts"

import { api } from "@/composables/api.ts"

export const useActiveEdition = () => {
  const edition = ref<ActiveEdition | undefined>(undefined)
  // Starts true so the page doesn't flash "no active edition" before loading
  const loading = ref<boolean>(true)
  const loadActiveEdition = async () => {
    loading.value = true
    try {
      edition.value = (await api<ActiveEdition | null>(`/edition`)) ?? undefined
    } finally {
      loading.value = false
    }
  }
  onMounted(loadActiveEdition)
  return { edition, loading }
}

const register = async (registration: NewRegistration) => {
  await api("/register", { method: "POST", body: registration })
}

export const useRegister = (
  schoolQuery: MaybeRefOrGetter<string | undefined>,
) => {
  const loading = ref<boolean>(false)
  const schools = ref<string[]>([])

  let request = 0
  const loadSchools = async (query: string | undefined) => {
    const current = ++request
    if (query === undefined) {
      schools.value = []
      loading.value = false
      return
    }
    loading.value = true
    try {
      const result = await api<string[]>(
        `/schools?query=${encodeURIComponent(query)}`,
      )
      // Ignore responses overtaken by a newer query
      if (current === request) schools.value = result
    } finally {
      if (current === request) loading.value = false
    }
  }

  watch(
    () => toValue(schoolQuery),
    async (query) => {
      await loadSchools(query)
    },
    { immediate: true },
  )

  return { schools, loading, register }
}
