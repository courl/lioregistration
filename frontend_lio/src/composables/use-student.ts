import { type MaybeRefOrGetter, ref, toValue, watch } from "vue"

import type { Student, StudentDetails, StudentId } from "@/types/student.ts"

import { api } from "@/composables/api.ts"

export const useStudents = (query: MaybeRefOrGetter<string>) => {
  const loading = ref<boolean>(false)
  const items = ref<Student[]>([])

  const reload = async (q: string) => {
    loading.value = true
    try {
      items.value = await api<Student[]>(`/admin/students?query=${q}`)
    } finally {
      loading.value = false
    }
  }

  watch(() => toValue(query), reload, { immediate: true })

  return {
    students: items,
    loading,
  }
}

export const useStudent = (id: MaybeRefOrGetter<StudentId | undefined>) => {
  const loading = ref<boolean>(false)
  const item = ref<StudentDetails>()

  const reload = async (id: StudentId | undefined) => {
    if (id === undefined) {
      item.value = undefined
      return
    }
    loading.value = true
    try {
      item.value = await api<StudentDetails>(`/admin/students/${id.toString()}`)
    } finally {
      loading.value = false
    }
  }

  watch(() => toValue(id), reload, { immediate: true })

  return { student: item, loading }
}
