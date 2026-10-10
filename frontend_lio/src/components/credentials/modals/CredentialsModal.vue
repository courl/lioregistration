<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { reactive } from "vue"

import {
  type CredentialsData,
  type CredentialsModalProps,
  type CredentialsModalResult,
  CredentialsSchema,
  parseCredentials,
} from "@/components/credentials/modals/CredentialsModal.schema.ts"

defineProps<CredentialsModalProps>()

const emit = defineEmits<{ close: [CredentialsModalResult] }>()

const state = reactive<Partial<CredentialsData>>({})

function onSubmit(event: FormSubmitEvent<CredentialsData>) {
  emit("close", parseCredentials(event.data.text))
}
</script>

<template>
  <UModal :title="title ?? 'Add credentials'">
    <template #body>
      <UForm
        :schema="CredentialsSchema"
        :state="state"
        class="space-y-4"
        @submit="onSubmit"
      >
        <UFormField
          label="Credentials"
          name="text"
          help="One 'username,password' pair per line."
        >
          <UTextarea
            v-model="state.text"
            class="w-full font-mono"
            :rows="8"
            placeholder="lio1,secret-password"
          />
        </UFormField>
        <div class="flex justify-end">
          <UFieldGroup>
            <UButton
              label="Cancel"
              color="error"
              variant="outline"
              @click="emit('close', undefined)"
            />
            <UButton
              type="submit"
              label="Add"
              color="primary"
              variant="subtle"
            />
          </UFieldGroup>
        </div>
      </UForm>
    </template>
  </UModal>
</template>
