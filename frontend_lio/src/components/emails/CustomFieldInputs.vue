<script setup lang="ts">
import type { CustomField, CustomValues } from "@/types/email.ts"

import CalendarDate from "@/types/calendar-date.ts"

const values = defineModel<CustomValues>({ required: true })

defineProps<{
  fields: CustomField[]
}>()

function dateValue(key: string) {
  try {
    return CalendarDate.parse(values.value[key] ?? "").toDateValue()
  } catch {
    return
  }
}

function setDate(
  key: string,
  date: Parameters<typeof CalendarDate.fromDateValue>[0],
) {
  values.value[key] = CalendarDate.fromDateValue(date)?.toString() ?? ""
}
</script>

<template>
  <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
    <UFormField
      v-for="field in fields"
      :key="field.key"
      :label="field.label"
      :required="field.required"
      :hint="`{{?${field.key}}}`"
      :class="field.kind === 'multiline' ? 'sm:col-span-2' : ''"
    >
      <UTextarea
        v-if="field.kind === 'multiline'"
        v-model="values[field.key]"
        class="w-full"
        autoresize
      />
      <UInputDate
        v-else-if="field.kind === 'date'"
        :range="false"
        class="w-full"
        :model-value="dateValue(field.key)"
        @update:model-value="(date) => setDate(field.key, date)"
      />
      <UInput v-else v-model="values[field.key]" class="w-full" />
    </UFormField>
  </div>
</template>
