import { type MaybeRefOrGetter, ref, toValue, watch } from "vue"

import type { EditionId } from "@/types/edition.ts"
import type {
  Registration,
  RegistrationDetails,
  RegistrationId,
} from "@/types/registration.ts"
import type { Student, StudentId } from "@/types/student.ts"

import { api, apiWith404Passthrough } from "@/composables/api.ts"

export const useRegistrations = (id: MaybeRefOrGetter<EditionId>) => {
  const loading = ref<boolean>(false)
  const items = ref<Registration[]>([])

  const reload = async (id: EditionId) => {
    loading.value = true
    try {
      items.value = await api<Registration[]>(
        `/admin/editions/${id.toString()}/registrations`,
      )
    } finally {
      loading.value = false
    }
  }

  watch(() => toValue(id), reload, { immediate: true })

  return { registrations: items, loading }
}

export const useRegistration = (
  id: MaybeRefOrGetter<RegistrationId | undefined>,
) => {
  const loading = ref<boolean>(false)
  const item = ref<RegistrationDetails>()

  const reload = async (id: RegistrationId) => {
    loading.value = true
    try {
      item.value = await api<RegistrationDetails>(
        `/admin/registrations/${id.toString()}`,
      )
      if (!item.value.student) {
        item.value.matches = await api<[number, Student][]>(
          `/admin/registrations/${id.toString()}/candidates`,
        )
      }
    } finally {
      loading.value = false
    }
  }

  const assignCredentials = async () => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }
    await api(`/admin/credentials/assign/${id_.toString()}`, { method: "POST" })
    await reload(id_)
  }

  const unassignCredentials = async () => {
    const id_ = toValue(id)
    if (id_ === undefined || !item.value?.credentials) {
      return
    }
    await api(
      `/admin/credentials/${item.value.credentials.id.toString()}/unassign`,
      { method: "POST" },
    )
    await reload(id_)
  }

  const linkAndValidate = async (student: Student | undefined) => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }
    if (student === undefined) {
      await api(`/admin/registrations/${id_.toString()}/link`, {
        method: "POST",
      })
    } else {
      await api(
        `/admin/registrations/${id_.toString()}/link/${student.id.toString()}`,
        { method: "POST" },
      )
    }
    await reload(id_)
  }

  const unlink = async () => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }
    await api(`/admin/registrations/${id_.toString()}/link`, {
      method: "DELETE",
    })
    await reload(id_)
  }

  const refuse = async () => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }
    await api(`/admin/registrations/${id_.toString()}/refuse`, {
      method: "DELETE",
    })
    await reload(id_)
  }

  const loadCandidate = async (studentId: StudentId) => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }
    try {
      return await apiWith404Passthrough<[number, Student]>(
        `/admin/registrations/${id_.toString()}/candidates/${studentId.toString()}`,
      )
    } catch (error) {
      if ((error as { status?: number }).status === 404) return
      throw error
    }
  }

  watch(
    () => toValue(id),
    async (id) => {
      if (id === undefined) {
        item.value = undefined
        return
      }
      await reload(id)
    },
    { immediate: true },
  )

  return {
    registration: item,
    loading,
    assignCredentials,
    unassignCredentials,
    linkAndValidate,
    unlink,
    refuse,
    loadCandidate,
  }
}
