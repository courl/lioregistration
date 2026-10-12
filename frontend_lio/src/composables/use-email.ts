import { type MaybeRefOrGetter, onMounted, ref, toValue, watch } from "vue"

import type {
  CustomField,
  CustomValues,
  EmailPreview,
  EmailPreviewRequest,
  EmailTemplate,
  EmailTemplateDetails,
  EmailTemplateId,
  EmailTemplateUpdateRequest,
  RoundEmailPreview,
  RoundEmailPreviewRequest,
} from "@/types/email.ts"

import { api } from "@/composables/api.ts"

export const useEmails = (
  filter?: NonNullable<EmailTemplate["template_type"]>,
) => {
  const loading = ref<boolean>(false)
  const items = ref<EmailTemplate[]>([])

  const reload = async () => {
    loading.value = true
    try {
      if (filter === undefined) {
        items.value = await api<EmailTemplate[]>(`/admin/emails`)
      } else {
        items.value = await api<EmailTemplate[]>(
          `/admin/emails?types=${filter}`,
        )
      }
    } finally {
      loading.value = false
    }
  }

  const createEmailTemplate = async () => {
    await api(`/admin/emails`, {
      method: "POST",
    })
    await reload()
  }

  onMounted(reload)

  return { emails: items, loading, createEmailTemplate }
}

export const useEmailTemplate = (
  id: MaybeRefOrGetter<EmailTemplateId | undefined>,
) => {
  const loading = ref<boolean>(false)
  const item = ref<EmailTemplateDetails>()

  const reload = async (id: EmailTemplateId) => {
    loading.value = true
    try {
      item.value = await api<EmailTemplateDetails>(
        `/admin/emails/${id.toString()}`,
      )
    } finally {
      loading.value = false
    }
  }

  const updateEmailTemplate = async (data: EmailTemplateUpdateRequest) => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return
    }
    await api(`/admin/emails/${id_.toString()}`, { method: "PUT", body: data })
    await reload(id_)
  }

  const previewEmailTemplate = async (data: EmailPreviewRequest) => {
    const id_ = toValue(id)
    if (id_ === undefined) {
      return { subject: "", attachments: [], error: "Cannot load preview" }
    }
    return await api<EmailPreview>(`/admin/emails/${id_.toString()}/preview`, {
      method: "POST",
      body: data,
    })
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
    email: item,
    loading,
    updateEmailTemplate,
    previewEmailTemplate,
  }
}

// Renders a round email exactly as one participant would receive it (read-only).
export const previewRoundEmail = async (
  templateId: EmailTemplateId,
  request: RoundEmailPreviewRequest,
) =>
  await api<RoundEmailPreview>(
    `/admin/emails/${templateId.toString()}/round-preview`,
    { method: "POST", body: request },
  )

// Initial values for a template's custom fields: the declared defaults, else `fallback`.
export const defaultCustomValues = (
  fields: CustomField[],
  fallback: (field: CustomField) => string = () => "",
): CustomValues =>
  Object.fromEntries(
    fields.map((field) => [field.key, field.default ?? fallback(field)]),
  )
