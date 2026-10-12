<script setup lang="ts">
import { computed, ref, watch } from "vue"

import type { RoundId } from "@/types/round.ts"
import type { StudentId } from "@/types/student.ts"

import { useConfirmDialog } from "@/composables/use-confirm-dialog.ts"
import {
  defaultCustomValues,
  previewRoundEmail,
  useEmails,
  useEmailTemplate,
} from "@/composables/use-email.ts"
import {
  type CustomValues,
  type EmailTemplateId,
  type RoundEmailPreview,
} from "@/types/email.ts"

const props = defineProps<{
  roundId: RoundId
}>()

const emit = defineEmits<{
  sendMail: [email: EmailTemplateId, custom: CustomValues]
}>()

const confirm = useConfirmDialog()
const { emails } = useEmails("round")
const selectedEmailId = ref<EmailTemplateId>()
const { email: selectedEmail } = useEmailTemplate(selectedEmailId)
const custom = ref<CustomValues>({})

const fields = computed(() => selectedEmail.value?.custom_fields ?? [])

// Each template has its own fields: start from its defaults when switching.
watch(selectedEmail, (email) => {
  custom.value = defaultCustomValues(email?.custom_fields ?? [])
})

// Every field is required, for sending as well as for the preview.
const canSend = computed(
  () =>
    selectedEmail.value !== undefined &&
    fields.value.every((field) => custom.value[field.key].trim() !== ""),
)

async function sendMail() {
  if (
    selectedEmailId.value !== undefined &&
    (await confirm({
      title: "Send Email",
      description: `Are you sure you want to send this email?`,
    }))
  ) {
    emit("sendMail", selectedEmailId.value, custom.value)
  }
}

// ---- read-only preview with the round's real data ------------------------
const previewOpen = ref(false)
const previewLoading = ref(false)
const preview = ref<RoundEmailPreview>()

// Without `studentId`, the backend previews the first participant.
async function loadPreview(studentId?: StudentId) {
  if (selectedEmailId.value === undefined) return
  previewLoading.value = true
  try {
    preview.value = await previewRoundEmail(selectedEmailId.value, {
      round_id: props.roundId,
      student_id: studentId,
      custom: custom.value,
    })
  } catch {
    // Already toasted by the API client
    previewOpen.value = false
  } finally {
    previewLoading.value = false
  }
}

async function openPreview() {
  preview.value = undefined
  previewOpen.value = true
  await loadPreview()
}
</script>

<template>
  <UCard>
    <template #header>
      <div class="flex items-center justify-between gap-2">
        <span class="font-bold">Send a mail to participants</span>
        <div class="flex gap-2">
          <UButton
            size="xs"
            variant="subtle"
            icon="i-lucide-eye"
            :disabled="!canSend"
            @click="openPreview"
            >Preview</UButton
          >
          <UButton
            size="xs"
            icon="i-lucide-send-horizontal"
            :disabled="!canSend"
            @click="sendMail"
            >Send</UButton
          >
        </div>
      </div>
    </template>
    <div class="space-y-4">
      <UFormField label="Template">
        <USelect
          v-model="selectedEmailId"
          :items="emails.map((x) => ({ label: x.name, value: x.id }))"
          class="w-full"
        />
      </UFormField>
      <template v-if="fields.length > 0">
        <USeparator />
        <CustomFieldInputs v-model="custom" :fields="fields" />
      </template>
    </div>
  </UCard>

  <USlideover
    v-model:open="previewOpen"
    title="Email preview"
    description="Exactly what the selected participant receives."
    :ui="{ content: 'max-w-4xl' }"
  >
    <template #body>
      <div class="space-y-4">
        <UFormField label="Participant">
          <USelect
            :model-value="preview?.student_id"
            :items="
              (preview?.recipients ?? []).map((r) => ({
                label: r.name,
                value: r.student_id,
              }))
            "
            :loading="previewLoading"
            class="w-full"
            @update:model-value="(id) => loadPreview(id as StudentId)"
          />
        </UFormField>
        <EmailPreviewPane v-if="preview" :preview="preview" />
        <div v-else class="flex justify-center p-8">
          <UIcon name="i-lucide-loader-circle" class="size-6 animate-spin" />
        </div>
      </div>
    </template>
  </USlideover>
</template>
