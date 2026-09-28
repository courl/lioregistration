import { type MaybeRefOrGetter, ref, toValue, watch } from "vue"

import type { EditionId, EditionStatistics } from "@/types/edition.ts"

import { api } from "@/composables/api.ts"

export const useEditionStatistics = (
  id: MaybeRefOrGetter<EditionId | undefined>,
) => {
  const loading = ref<boolean>(false)
  const item = ref<EditionStatistics>()

  const reload = async (id: EditionId | undefined) => {
    if (id === undefined) {
      item.value = undefined
      return
    }
    loading.value = true
    try {
      item.value = await api<EditionStatistics>(
        `/admin/editions/${id.toString()}/statistics`,
      )
    } finally {
      loading.value = false
    }
  }

  watch(() => toValue(id), reload, { immediate: true })

  return { statistics: item, loading }
}
