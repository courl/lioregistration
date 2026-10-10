<script setup lang="ts">
import { ref } from "vue"

import { useConfirmDialog } from "@/composables/use-confirm-dialog.ts"
import { useEmails } from "@/composables/use-email.ts"
import { type EmailTemplateId } from "@/types/email.ts"

const emit = defineEmits<{
  sendMail: [email: EmailTemplateId, headline: string, body: string]
}>()

const confirm = useConfirmDialog()
const { emails } = useEmails("round")
const selectedEmailId = ref<EmailTemplateId>()
const headline = ref<string>("LIO - NEWS")
const body = ref<string>("")

async function sendMail() {
  if (
    selectedEmailId.value !== undefined &&
    (await confirm({
      title: "Send Email",
      description: `Are you sure you want to send this email?`,
    }))
  ) {
    emit("sendMail", selectedEmailId.value, headline.value, body.value)
  }
}
</script>

<template>
  <UCard>
    <template #header>
      <div class="flex items-center justify-between">
        <span class="font-bold">Send a mail to participants</span>
        <UButton
          size="xs"
          icon="i-lucide-send-horizontal"
          :disabled="selectedEmailId === undefined || headline === ''"
          @click="sendMail"
          >Send</UButton
        >
      </div>
    </template>
    <div class="space-y-4">
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <UFormField label="Template">
          <USelect
            v-model="selectedEmailId"
            :items="emails.map((x) => ({ label: x.name, value: x.id }))"
            class="w-full"
          />
        </UFormField>
        <UFormField label="Headline">
          <UInput v-model="headline" class="w-full" />
        </UFormField>
      </div>
      <UFormField label="Body">
        <UTextarea v-model="body" class="w-full" autoresize />
      </UFormField>
    </div>
  </UCard>
</template>
