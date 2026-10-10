<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue"

import type {
  EmailPreview,
  EmailPreviewRequest,
  EmailTemplateId,
} from "@/types/email.ts"

import { useConfirmDialog } from "@/composables/use-confirm-dialog.ts"
import { useAttachments } from "@/composables/use-email-attachment.ts"
import { useEmailTemplate } from "@/composables/use-email.ts"
import { useRouteId } from "@/composables/use-route-id.ts"
import CalendarDate from "@/types/calendar-date.ts"

const id = useRouteId<EmailTemplateId>()
const confirm = useConfirmDialog()
const { email, loading, previewEmailTemplate, updateEmailTemplate } =
  useEmailTemplate(id)
const { attachments, createAttachment, updateAttachment, deleteAttachment } =
  useAttachments(id)
const preview = ref<EmailPreview>({
  attachments: [],
  error: "Not computed yet",
})
const tab = ref<string>("template")
const previewTab = ref<string>("preview")

const body = ref<string>("This is a **test**")
const global_context: EmailPreviewRequest["global_context"] = reactive({
  edition: "LIO 2026",
  headline: "LIO - NEWS",
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
    preview.value = { attachments: [], error: "Not computed yet" }
    return
  }
  previewLoading.value = true
  try {
    preview.value = await previewEmailTemplate({
      body: body.value,
      global_context,
      instance_context,
    })
  } finally {
    previewLoading.value = false
  }
}

// The preview renders the saved template, refresh it on load and save
watch(email, refreshPreview)

const attachmentPreview = computed(() =>
  previewTab.value.startsWith("attachment-")
    ? preview.value.attachments.at(Number(previewTab.value.slice(11)))
    : undefined,
)

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
      global_context.headline = "Registration"
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
              <UFormField
                label="Headline"
                :hidden="email.template_type === 'registration'"
              >
                <UInput
                  v-model="global_context.headline"
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
            <UFormField
              label="Body"
              :hidden="email.template_type === 'registration'"
            >
              <UTextarea v-model="body" class="w-full" autoresize />
            </UFormField>
          </section>

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
    <div>
      <UTabs
        v-model="previewTab"
        :items="[
          { icon: 'i-lucide-eye', label: 'Preview', value: 'preview' },
          ...(preview.attachments
            ? preview.attachments.map((x, idx) => ({
                label: x.name,
                value: `attachment-${idx}`,
              }))
            : []),
        ]"
        variant="link"
      />
      <div class="rounded border border-muted h-full">
        <template v-if="previewTab === 'preview'">
          <!-- eslint-disable-next-line vue/no-v-html -->
          <div v-if="preview.html" v-html="preview.html" />
          <UError
            v-if="preview.error"
            :error="{
              statusMessage: 'Cannot load preview',
              message: preview.error,
            }"
            :clear="false"
          />
        </template>
        <template v-else-if="attachmentPreview">
          <PdfViewer
            v-if="attachmentPreview.data_base64"
            :src="attachmentPreview.data_base64"
          />
          <UError
            v-if="attachmentPreview.error"
            :error="{
              statusMessage: 'Cannot load preview',
              message: attachmentPreview.error,
            }"
            :clear="false"
          />
        </template>
      </div>
    </div>
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
