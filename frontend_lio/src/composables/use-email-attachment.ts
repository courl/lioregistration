import { type MaybeRefOrGetter, ref, toValue, watch } from "vue"

import type { EmailAttachment, EmailTemplateId } from "@/types/email.ts"

import { api } from "@/composables/api.ts"

export const useAttachments = (
  id: MaybeRefOrGetter<EmailTemplateId | undefined>,
) => {
  const loading = ref<boolean>(false)
  const items = ref<EmailAttachment[]>([])

  const reload = async () => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      items.value = []
      loading.value = false
      return
    }
    loading.value = true
    try {
      items.value = await api<EmailAttachment[]>(
        `/admin/emails/${id_.toString()}/attachment`,
      )
    } finally {
      loading.value = false
    }
  }

  const createAttachment = async () => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }
    await api(`/admin/emails/${id_.toString()}/attachment`, {
      method: "POST",
    })
    await reload()
  }

  const updateAttachment = async (attachment: EmailAttachment) => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }

    await api(
      `/admin/emails/${id_.toString()}/attachment/${attachment.id.toString()}`,
      {
        method: "PATCH",
        body: { ...attachment, id: undefined },
      },
    )
    await reload()
  }

  const deleteAttachment = async (attachment: EmailAttachment) => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }

    await api(
      `/admin/emails/${id_.toString()}/attachment/${attachment.id.toString()}`,
      {
        method: "DELETE",
      },
    )
    await reload()
  }
  watch(() => toValue(id), reload, { immediate: true })

  return {
    attachments: items,
    loading,
    createAttachment,
    updateAttachment,
    deleteAttachment,
  }
}
