import { type MaybeRefOrGetter, onMounted, ref, toValue, watch } from "vue"

import type { ActiveEdition } from "@/types/edition.ts"
import type { NewRegistration } from "@/types/registration.ts"

import { api } from "@/composables/api.ts"

export const useActiveEdition = () => {
  const edition = ref<ActiveEdition | undefined>(undefined)
  const loading = ref<boolean>(false)
  const loadActiveEdition = async () => {
    loading.value = true
    try {
      edition.value = await api<ActiveEdition | undefined>(`/edition`)
    } finally {
      loading.value = false
    }
  }
  onMounted(loadActiveEdition)
  return { edition, loading }
}

export const useRegister = (
  schoolQuery: MaybeRefOrGetter<string | undefined>,
) => {
  const loading = ref<boolean>(false)
  const schools = ref<string[]>([])

  const loadSchools = async (query: string | undefined) => {
    if (query === undefined) {
      schools.value = []
      return
    }
    loading.value = true
    try {
      schools.value = await api<string[]>(`/schools?query=${query}`)
    } finally {
      loading.value = false
    }
  }

  const register = async (registration: NewRegistration) => {
    await api("/register", { method: "POST", body: registration })
    await loadSchools(toValue(schoolQuery))
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
