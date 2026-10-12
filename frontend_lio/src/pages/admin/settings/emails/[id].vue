<script setup lang="ts">
import { reactive, ref, watch } from "vue"

import type {
  CustomField,
  CustomValues,
  EmailPreview,
  EmailPreviewRequest,
  EmailTemplateId,
} from "@/types/email.ts"

import { useConfirmDialog } from "@/composables/use-confirm-dialog.ts"
import { useAttachments } from "@/composables/use-email-attachment.ts"
import {
  defaultCustomValues,
  useEmailTemplate,
} from "@/composables/use-email.ts"
import { useRouteId } from "@/composables/use-route-id.ts"
import CalendarDate from "@/types/calendar-date.ts"

const id = useRouteId<EmailTemplateId>()
const confirm = useConfirmDialog()
const { email, loading, previewEmailTemplate, updateEmailTemplate } =
  useEmailTemplate(id)
const { attachments, createAttachment, updateAttachment, deleteAttachment } =
  useAttachments(id)
const preview = ref<EmailPreview>({
  subject: "",
  attachments: [],
  error: "Not computed yet",
})
const tab = ref<string>("template")
// Test values for the template's custom fields.
const customValues = ref<CustomValues>({})

const global_context: EmailPreviewRequest["global_context"] = reactive({
  edition: "LIO 2026",
  round_starts_at: new CalendarDate({ year: 2025, month: 11, day: 25 }),
  round_ends_at: new CalendarDate({ year: 2026, month: 2, day: 15 }),
  students: [
    ["Alice Müller", 12],
    ["Peter Schmidt", 8],
    ["Thierry Hubert", 3.21],
  ],
})

const instance_context: EmailPreviewRequest["instance_context"] = reactive({
  given_name: "Alice",
  family_name: "Becker",
  credentials_username: undefined,
  credentials_password: undefined,
  grade: "2B",
  school: "Lycée des Informaticians",
})

const previewLoading = ref(false)

async function refreshPreview() {
  if (!email.value) {
    preview.value = { subject: "", attachments: [], error: "Not computed yet" }
    return
  }
  previewLoading.value = true
  try {
    preview.value = await previewEmailTemplate({
      global_context,
      instance_context,
      custom: customValues.value,
    })
  } finally {
    previewLoading.value = false
  }
}

// Every field is required: without a default, preview with the label (or today).
const sampleValue = (field: CustomField) =>
  field.kind === "date" ? new CalendarDate({}).toString() : field.label

// The preview renders the saved template, refresh it on load and save. Values typed in
// for the preview are kept.
watch(email, async (email) => {
  const typed = customValues.value
  customValues.value = Object.fromEntries(
    Object.entries(
      defaultCustomValues(email?.custom_fields ?? [], sampleValue),
    ).map(([key, value]) => [key, typed[key] || value]),
  )
  await refreshPreview()
})

async function removeAttachment(index: number) {
  const attachment = attachments.value[index]
  if (
    await confirm({
      title: "Delete attachment",
      description: `Are you sure you want to delete '${attachment.name}'?`,
    })
  ) {
    await deleteAttachment(attachment)
  }
}

watch(
  () => email.value?.template_type,
  (newType) => {
    if (newType === "round") {
      global_context.students = [
        ["Alice Müller", 12],
        ["Peter Schmidt", 8],
        ["Thierry Hubert", 3.21],
      ]
      instance_context.credentials_username = undefined
      instance_context.credentials_password = undefined
    } else {
      global_context.students = []
      instance_context.credentials_username = "lio1"

      instance_context.credentials_password = "password-1"
    }
  },
)

async function save() {
  for (const attachment of attachments.value) {
    await updateAttachment(attachment)
  }
  if (email.value) {
    await updateEmailTemplate(email.value)
  }
}

function setContextDate(
  key: "round_starts_at" | "round_ends_at",
  date: Parameters<typeof CalendarDate.fromDateValue>[0],
) {
  global_context[key] = CalendarDate.fromDateValue(date) ?? new CalendarDate({})
}

const TEMPLATE_TYPES = ["round", "registration", "error"] as const

function setTemplateType(value: unknown) {
  if (!email.value) return
  email.value.template_type =
    TEMPLATE_TYPES.find((type) => type === value) ?? null
}
</script>

<template>
  <PageHeader :title="email?.name || 'Unknown Template'">
    <template #header>
      <UButton v-if="email" icon="i-lucide-save" @click="save"> Save </UButton>
    </template>
  </PageHeader>
  <div v-if="email" class="grid grid-cols-1 gap-4 md:grid-cols-2">
    <div>
      <UTabs
        v-model="tab"
        :items="[
          { icon: 'i-lucide-mail', label: 'Template', value: 'template' },
          {
            icon: 'i-lucide-paperclip',
            label: 'Attachments',
            value: 'attachment',
          },
          {
            icon: 'i-lucide-text-cursor-input',
            label: 'Fields',
            value: 'fields',
          },
          {
            icon: 'i-lucide-sliders-horizontal',
            label: 'Preview',
            value: 'preview',
          },
        ]"
      />
      <template v-if="tab === 'template'">
        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 mb-2">
          <UFormField label="Template Name">
            <UInput v-model="email.name" class="w-full" />
          </UFormField>
          <UFormField label="Template Type" class="w-full">
            <USelect
              :model-value="email.template_type ?? 'unset'"
              class="w-full"
              :items="[
                [
                  { label: 'round', value: 'round' },
                  { label: 'registration', value: 'registration' },
                  { label: 'error', value: 'error' },
                ],
                [{ label: 'Unset the template', value: 'unset' }],
              ]"
              @update:model-value="setTemplateType"
            />
          </UFormField>
        </div>
        <UFormField
          label="Subject"
          hint="Placeholders allowed, e.g. {{?headline}}"
          class="mb-2"
        >
          <UInput v-model="email.subject" class="w-full" />
        </UFormField>
        <UFormField label="Template">
          <UTextarea
            v-model="email.template"
            class="w-full font-mono"
            autoresize
          />
        </UFormField>
      </template>
      <template v-else-if="tab === 'attachment'">
        <UAccordion
          :items="attachments.map((x, idx) => ({ label: x.name, idx }))"
        >
          <template #content="{ item }">
            <div class="space-y-2">
              <UFormField label="Template Name">
                <UInput v-model="attachments[item.idx].name" class="w-full" />
              </UFormField>
              <UFormField label="Template">
                <UTextarea
                  v-model="attachments[item.idx].template"
                  class="w-full font-mono"
                  autoresize
                />
              </UFormField>
              <div class="flex justify-end">
                <UButton
                  icon="i-lucide-trash"
                  color="error"
                  variant="ghost"
                  size="xs"
                  aria-label="Delete attachment"
                  @click="async () => await removeAttachment(item.idx)"
                />
              </div>
            </div>
          </template>
        </UAccordion>
        <UButton
          icon="i-lucide-plus"
          label="Add attachment"
          variant="subtle"
          class="mt-2"
          @click="createAttachment"
        />
      </template>
      <template v-else-if="tab === 'fields'">
        <CustomFieldsEditor v-model="email.custom_fields" />
      </template>
      <template v-else-if="tab === 'preview'">
        <div class="space-y-4">
          <section class="space-y-2">
            <div>
              <h2 class="text-base font-semibold text-highlighted">
                Global Context
              </h2>
            </div>
            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <UFormField label="Edition">
                <UInput
                  v-model="global_context.edition"
                  class="w-full"
                ></UInput>
              </UFormField>
            </div>
            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <UFormField label="Round Start">
                <UInputDate
                  :range="false"
                  class="w-full"
                  :model-value="global_context.round_starts_at.toDateValue()"
                  @update:model-value="
                    (date) => setContextDate('round_starts_at', date)
                  "
                />
              </UFormField>
              <UFormField label="Round End">
                <UInputDate
                  :range="false"
                  class="w-full"
                  :model-value="global_context.round_ends_at.toDateValue()"
                  @update:model-value="
                    (date) => setContextDate('round_ends_at', date)
                  "
                />
              </UFormField>
            </div>
          </section>

          <template v-if="email.custom_fields.length > 0">
            <USeparator />
            <section class="space-y-2">
              <h2 class="text-base font-semibold text-highlighted">
                Custom Fields
              </h2>
              <CustomFieldInputs
                v-model="customValues"
                :fields="email.custom_fields"
              />
            </section>
          </template>

          <USeparator />
          <section class="space-y-2">
            <div>
              <h2 class="text-base font-semibold text-highlighted">Instance</h2>
            </div>
            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <UFormField label="Given Name">
                <UInput
                  v-model="instance_context.given_name"
                  class="w-full"
                ></UInput>
              </UFormField>
              <UFormField label="Family Name">
                <UInput
                  v-model="instance_context.family_name"
                  class="w-full"
                ></UInput>
              </UFormField>
            </div>
            <div
              class="grid grid-cols-1 gap-4 sm:grid-cols-2"
              :hidden="email.template_type === 'round'"
            >
              <UFormField label="Credentials Username">
                <UInput
                  v-model="instance_context.credentials_username"
                  class="w-full"
                ></UInput>
              </UFormField>
              <UFormField label="Credentials Password">
                <UInput
                  v-model="instance_context.credentials_password"
                  class="w-full"
                ></UInput>
              </UFormField>
            </div>
            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <UFormField label="School">
                <UInput
                  v-model="instance_context.school"
                  class="w-full"
                ></UInput>
              </UFormField>
              <UFormField label="Grade">
                <UInput
                  v-model="instance_context.grade"
                  class="w-full"
                ></UInput>
              </UFormField>
            </div>
          </section>
          <div class="flex justify-end">
            <UButton
              icon="i-lucide-refresh-cw"
              label="Refresh preview"
              variant="subtle"
              :loading="previewLoading"
              @click="refreshPreview"
            />
          </div>
        </div>
      </template>
    </div>
    <EmailPreviewPane :preview="preview" />
  </div>
  <UEmpty
    v-else-if="!loading"
    icon="i-lucide-mail"
    title="Email template not found"
    description="It looks like this template does not exist."
    :actions="[
      {
        icon: 'i-lucide-circle-arrow-right',
        label: 'Templates',
        to: '/admin/settings/emails',
      },
    ]"
  />
</template>
