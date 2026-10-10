import { FetchError } from "ofetch"
import { type MaybeRefOrGetter, onMounted, ref, toValue, watch } from "vue"

import type { Edition, EditionId, NewEdition } from "@/types/edition.ts"

import { api, apiWith404Passthrough } from "@/composables/api.ts"

export const useEditions = () => {
  const loading = ref<boolean>(false)
  const error = ref<boolean>(false)
  const items = ref<Edition[]>([])

  const reload = async () => {
    loading.value = true
    try {
      items.value = await api<Edition[]>("/admin/editions")
    } finally {
      loading.value = false
    }
  }

  const activate = async (id: EditionId) => {
    await api(`/admin/editions/${id.toString()}/activate`, { method: "PUT" })
    await reload()
  }

  const createEdition = async (data: NewEdition) => {
    await api("/admin/editions", { method: "POST", body: data })
    await reload()
  }

  onMounted(reload)

  return {
    editions: items,
    loading,
    error,
    activateEdition: activate,
    createEdition,
  }
}

export const useEdition = (
  id: MaybeRefOrGetter<EditionId | "active" | undefined>,
) => {
  const loading = ref<boolean>(false)
  const error = ref<boolean>(false)
  const item = ref<Edition>()

  const reload = async (id: EditionId | "active") => {
    loading.value = true
    try {
      item.value = await apiWith404Passthrough<Edition>(
        `/admin/editions/${id.toString()}`,
      )
    } catch (error_) {
      // No (active) edition is a normal state, rendered by the page
      if (error_ instanceof FetchError && error_.status === 404) {
        item.value = undefined
        return
      }
      throw error_
    } finally {
      loading.value = false
    }
  }

  const refresh = async () => {
    const id_ = toValue(id)
    if (id_ !== undefined) await reload(id_)
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

  return { edition: item, loading, error, refresh }
}
