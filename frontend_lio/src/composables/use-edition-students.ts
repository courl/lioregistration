import { type MaybeRefOrGetter, ref, toValue, watch } from "vue"

import type { EditionId } from "@/types/edition.ts"
import type { Student } from "@/types/student.ts"

import { api } from "@/composables/api.ts"

export const useEditionStudents = (
  id: MaybeRefOrGetter<EditionId | undefined>,
) => {
  const loading = ref<boolean>(false)
  const items = ref<Student[]>([])

  const reload = async (id: EditionId | undefined) => {
    if (id === undefined) {
      items.value = []
      return
    }
    loading.value = true
    try {
      items.value = await api<Student[]>(
        `/admin/editions/${id.toString()}/students`,
      )
    } finally {
      loading.value = false
    }
  }

  watch(() => toValue(id), reload, { immediate: true })

  return { students: items, loading }
}
