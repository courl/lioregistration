<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { reactive, watch } from "vue"

import {
  type EditionData,
  type EditionModalProps,
  type EditionModalResult,
  EditionSchema,
} from "@/components/edition/modals/EditionModal.schema.ts"
import CalendarDate from "@/types/calendar-date.ts"

const props = defineProps<EditionModalProps>()

const emit = defineEmits<{ close: [EditionModalResult] }>()

const state = reactive<Partial<EditionData>>({})

watch(
  () => props.edition,
  (edition) => {
    Object.assign(state, {
      name: edition?.name,
      year: edition?.year,
      registration_opens_at: edition?.registration_opens_at,
      registration_closes_at: edition?.registration_closes_at,
    })
  },
  { immediate: true },
)

function onSubmit(event: FormSubmitEvent<EditionData>) {
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
        :schema="EditionSchema"
        :state="state"
        class="space-y-4"
        @submit="onSubmit"
      >
        <UFormField label="Name" name="name">
          <UInput v-model="state.name" class="w-full" />
        </UFormField>
        <UFormField label="Year" name="year">
          <UInput
            v-model.number="state.year"
            type="number"
            min="0"
            class="w-full"
          />
        </UFormField>
        <div class="grid grid-cols-2 gap-4">
          <UFormField
            label="Registration opens at"
            name="registration_opens_at"
          >
            <UInputDate
              :range="false"
              :model-value="state.registration_opens_at?.toDateValue()"
              @update:model-value="
                (date) => (state.registration_opens_at = fromInputDate(date))
              "
            />
          </UFormField>
          <UFormField
            label="Registration ends at"
            name="registration_closes_at"
          >
            <UInputDate
              :range="false"
              :model-value="state.registration_closes_at?.toDateValue()"
              @update:model-value="
                (date) => (state.registration_closes_at = fromInputDate(date))
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
