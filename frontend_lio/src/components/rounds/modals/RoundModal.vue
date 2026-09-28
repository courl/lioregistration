<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { reactive, useTemplateRef, watch } from "vue"

import {
  type RoundData,
  type RoundModalProps,
  type RoundModalResult,
  RoundSchema,
} from "@/components/rounds/modals/RoundModal.schema.ts"
import CalendarDate from "@/types/calendar-date.ts"

const props = defineProps<RoundModalProps>()

const emit = defineEmits<{ close: [RoundModalResult] }>()

const state = reactive<Partial<RoundData>>({})
const form = useTemplateRef("form")

watch(
  () => props.round,
  (round) => {
    Object.assign(state, {
      name: round?.name,
      starts_at: round?.starts_at.getDate(),
      ends_at: round?.ends_at.getDate(),
    })
  },
  { immediate: true },
)

function onSubmit(event: FormSubmitEvent<RoundData>) {
  emit("close", {
    ...event.data,
    starts_at: new CalendarDate({
      date: event.data.starts_at,
    }),
    ends_at: new CalendarDate({
      date: event.data.ends_at,
    }),
  })
}
</script>

<template>
  <UModal :title="title" :ui="{ footer: 'justify-end' }">
    <template #body>
      <UForm
        ref="form"
        :schema="RoundSchema"
        :state="state"
        class="space-y-4"
        @submit="onSubmit"
      >
        <UFormField label="Name" name="name">
          <UInput v-model="state.name" class="w-full" />
        </UFormField>
        <div class="grid grid-cols-2 gap-4">
          <UFormField label="Starts at" name="starts_at">
            <UInputDate
              :range="false"
              @update:model-value="
                (value) => {
                  state.starts_at = value ? value.toDate('UTC') : undefined
                }
              "
            />
          </UFormField>
          <UFormField label="Ends at" name="ends_at">
            <UInputDate
              :range="false"
              @update:model-value="
                (value) => {
                  state.ends_at = value ? value.toDate('UTC') : undefined
                }
              "
            />
          </UFormField>
        </div>
      </UForm>
    </template>
    <template #footer>
      <UFieldGroup>
        <UButton
          label="Cancel"
          color="error"
          variant="outline"
          @click="emit('close', undefined)"
        />
        <UButton
          label="Confirm"
          color="primary"
          variant="subtle"
          @click="form?.submit()"
        />
      </UFieldGroup>
    </template>
  </UModal>
</template>
