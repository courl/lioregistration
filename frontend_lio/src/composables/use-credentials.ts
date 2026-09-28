import { type MaybeRefOrGetter, ref, toValue, watch } from "vue"

import type { CredentialId, Credentials } from "@/types/credentials.ts"
import type { EditionId } from "@/types/edition.ts"

import { api } from "@/composables/api.ts"

export const useCredentials = (id: MaybeRefOrGetter<EditionId | undefined>) => {
  const loading = ref<boolean>(false)
  const items = ref<Credentials[]>([])

  const reload = async (id: EditionId | undefined) => {
    if (id === undefined) {
      items.value = []
      return
    }
    loading.value = true
    try {
      items.value = await api<Credentials[]>(
        `/admin/editions/${id.toString()}/credentials`,
      )
    } finally {
      loading.value = false
    }
  }

  const createCredentials = async (
    request: { username: string; password: string }[],
  ) => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }
    await api(`/admin/editions/${id_.toString()}/credentials`, {
      method: "POST",
      body: request,
    })
    await reload(id_)
  }

  const deleteCredentials = async (id_: CredentialId) => {
    await api(`/admin/credentials/${id_.toString()}`, { method: "DELETE" })
    await reload(toValue(id))
  }

  watch(() => toValue(id), reload, { immediate: true })

  return { credentials: items, loading, createCredentials, deleteCredentials }
}
