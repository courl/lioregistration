<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { reactive, useTemplateRef, watch } from "vue"

import {
  type EditionData,
  type EditionModalProps,
  type EditionModalResult,
  EditionSchema,
} from "@/components/edition/modals/EditionModal.schema"
import CalendarDate from "@/types/calendar-date.ts"

const props = defineProps<EditionModalProps>()

const emit = defineEmits<{ close: [EditionModalResult] }>()

const state = reactive<Partial<EditionData>>({})
const form = useTemplateRef("form")

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
  emit("close", {
    ...event.data,
    registration_opens_at: new CalendarDate({
      date: event.data.registration_opens_at,
    }),
    registration_closes_at: new CalendarDate({
      date: event.data.registration_closes_at,
    }),
  })
}
</script>

<template>
  <UModal :title="title" :ui="{ footer: 'justify-end' }">
    <template #body>
      <UForm
        ref="form"
        :schema="EditionSchema"
        :state="state"
        class="space-y-4"
        @submit="onSubmit"
      >
        <UFormField label="Name" name="name">
          <UInput v-model="state.name" class="w-full" />
        </UFormField>
        <UFormField label="Year" name="year">
          <UInput v-model="state.year" type="number" min="0" class="w-full" />
        </UFormField>
        <div class="grid grid-cols-2 gap-4">
          <UFormField
            label="Registration opens at"
            name="registration_opens_at"
          >
            <UInputDate
              :range="false"
              @update:model-value="
                (value) => {
                  state.registration_opens_at = value
                    ? value.toDate('UTC')
                    : undefined
                }
              "
            />
          </UFormField>
          <UFormField
            label="Registration ends at"
            name="registration_closes_at"
          >
            <UInputDate
              :range="false"
              @update:model-value="
                (value) => {
                  state.registration_closes_at = value
                    ? value.toDate('UTC')
                    : undefined
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
