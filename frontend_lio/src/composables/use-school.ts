import { type MaybeRefOrGetter, ref, toValue, watch } from "vue"

import type { School, SchoolId } from "@/types/school.ts"

import { api } from "@/composables/api.ts"

export const useSchools = (query: MaybeRefOrGetter<string>) => {
  const loading = ref<boolean>(false)
  const items = ref<School[]>([])

  let request = 0
  const reload = async (q: string) => {
    const current = ++request
    loading.value = true
    try {
      const result = await api<School[]>(
        `/admin/schools?query=${encodeURIComponent(q)}`,
      )
      // Ignore responses overtaken by a newer query
      if (current === request) items.value = result
    } finally {
      if (current === request) loading.value = false
    }
  }
  const mergeSchools = async (
    into_school_id: SchoolId,
    other_school_id: SchoolId,
  ) => {
    await api(
      `/admin/schools/${into_school_id.toString()}/merge/${other_school_id.toString()}`,
      { method: "POST" },
    )
    await reload(toValue(query))
  }

  watch(() => toValue(query), reload, { immediate: true })

  return {
    schools: items,
    loading,
    mergeSchools,
  }
}

export const useSchool = (id: MaybeRefOrGetter<SchoolId | undefined>) => {
  const loading = ref<boolean>(false)
  const item = ref<School>()

  const reload = async (id: SchoolId | undefined) => {
    if (id === undefined) {
      item.value = undefined
      return
    }
    loading.value = true
    try {
      item.value = await api<School>(`/admin/schools/${id.toString()}`)
    } finally {
      loading.value = false
    }
  }

  watch(() => toValue(id), reload, { immediate: true })

  return { school: item, loading }
}
