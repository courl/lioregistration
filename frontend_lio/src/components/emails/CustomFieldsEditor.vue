<script setup lang="ts">
import type { CustomField, CustomFieldKind } from "@/types/email.ts"

const fields = defineModel<CustomField[]>({ required: true })

const PLACEHOLDER = "{{?key}}"

const KINDS: { label: string; value: CustomFieldKind }[] = [
  { label: "Text", value: "text" },
  { label: "Multi-line (Markdown)", value: "multiline" },
  { label: "Date", value: "date" },
]

function addField() {
  fields.value.push({
    key: `field_${(fields.value.length + 1).toString()}`,
    label: "New field",
    kind: "text",
    default: null,
  })
}
</script>

<template>
  <div class="space-y-3">
    <p class="text-sm text-muted">
      Values filled in when sending this template, all required. Use
      <code v-text="PLACEHOLDER" /> in the subject, the template or an
      attachment. Dates are inserted as <code>YYYY-MM-DD</code>.
    </p>
    <UCard v-for="(field, index) in fields" :key="index">
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-4">
        <UFormField label="Key">
          <UInput v-model="field.key" class="w-full font-mono" />
        </UFormField>
        <UFormField label="Label">
          <UInput v-model="field.label" class="w-full" />
        </UFormField>
        <UFormField label="Kind">
          <USelect v-model="field.kind" :items="KINDS" class="w-full" />
        </UFormField>
        <UFormField label="Default">
          <div class="flex gap-1">
            <UInput
              :model-value="field.default ?? ''"
              class="w-full"
              @update:model-value="(v) => (field.default = String(v) || null)"
            />
            <UButton
              icon="i-lucide-trash"
              color="error"
              variant="ghost"
              aria-label="Delete field"
              @click="fields.splice(index, 1)"
            />
          </div>
        </UFormField>
      </div>
    </UCard>
    <UButton
      icon="i-lucide-plus"
      label="Add field"
      variant="subtle"
      @click="addField"
    />
  </div>
</template>
