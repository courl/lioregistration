<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { reactive, watch } from "vue"

import {
  type RoundModalProps,
  type RoundModalResult,
  RoundSchema,
} from "@/components/rounds/modals/RoundModal.schema.ts"
import CalendarDate from "@/types/calendar-date.ts"

type RoundData = NonNullable<RoundModalResult>

const props = defineProps<RoundModalProps>()

const emit = defineEmits<{ close: [RoundModalResult] }>()

const state = reactive<Partial<RoundData>>({})

watch(
  () => props.round,
  (round) => {
    Object.assign(state, {
      name: round?.name,
      starts_at: round?.starts_at,
      ends_at: round?.ends_at,
    })
  },
  { immediate: true },
)

function onSubmit(event: FormSubmitEvent<RoundData>) {
  emit("close", event.data)
}

const fromInputDate = (
  date: Parameters<typeof CalendarDate.fromDateValue>[0],
) => CalendarDate.fromDateValue(date)
</script>

<template>
  <UModal :title="title">
    <template #body>
      <UForm
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
              :model-value="state.starts_at?.toDateValue()"
              @update:model-value="
                (date) => (state.starts_at = fromInputDate(date))
              "
            />
          </UFormField>
          <UFormField label="Ends at" name="ends_at">
            <UInputDate
              :range="false"
              :model-value="state.ends_at?.toDateValue()"
              @update:model-value="
                (date) => (state.ends_at = fromInputDate(date))
              "
            />
          </UFormField>
        </div>
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
              label="Confirm"
              color="primary"
              variant="subtle"
            />
          </UFieldGroup>
        </div>
      </UForm>
    </template>
  </UModal>
</template>
