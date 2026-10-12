<script setup lang="ts">
import type { CustomField, CustomFieldKind } from "@/types/email.ts"

const fields = defineModel<CustomField[]>({ required: true })

const PLACEHOLDER = "{{?key}}"
const TYPST_EXAMPLE = 'long-date("{{?key}}")'

const KINDS: { label: string; value: CustomFieldKind }[] = [
  { label: "Text", value: "text" },
  { label: "Multi-line text", value: "multiline" },
  { label: "Date", value: "date" },
]

function addField() {
  fields.value.push({
    key: `field_${(fields.value.length + 1).toString()}`,
    label: "New field",
    kind: "text",
    required: false,
    default: null,
  })
}

function move(index: number, offset: number) {
  const [field] = fields.value.splice(index, 1)
  fields.value.splice(index + offset, 0, field)
}
</script>

<template>
  <div class="space-y-3">
    <p class="text-sm text-muted">
      Values filled in when sending this template to a round. Use
      <code v-text="PLACEHOLDER" /> in the template, the body or an attachment.
      Dates are inserted as <code>YYYY-MM-DD</code>, so a Typst attachment can
      format them, e.g. <code v-text="TYPST_EXAMPLE" />.
    </p>
    <UCard v-for="(field, index) in fields" :key="index">
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <UFormField label="Key" required>
          <UInput v-model="field.key" class="w-full font-mono" />
        </UFormField>
        <UFormField label="Label" required>
          <UInput v-model="field.label" class="w-full" />
        </UFormField>
        <UFormField label="Kind">
          <USelect v-model="field.kind" :items="KINDS" class="w-full" />
        </UFormField>
        <UFormField
          label="Default"
          :hint="field.kind === 'date' ? 'YYYY-MM-DD' : undefined"
          class="sm:col-span-2"
        >
          <UInput
            :model-value="field.default ?? ''"
            class="w-full"
            @update:model-value="(v) => (field.default = String(v) || null)"
          />
        </UFormField>
        <div class="flex items-end justify-between gap-2">
          <UCheckbox v-model="field.required" label="Required" />
          <div class="flex">
            <UButton
              icon="i-lucide-arrow-up"
              variant="ghost"
              size="xs"
              aria-label="Move up"
              :disabled="index === 0"
              @click="move(index, -1)"
            />
            <UButton
              icon="i-lucide-arrow-down"
              variant="ghost"
              size="xs"
              aria-label="Move down"
              :disabled="index === fields.length - 1"
              @click="move(index, 1)"
            />
            <UButton
              icon="i-lucide-trash"
              color="error"
              variant="ghost"
              size="xs"
              aria-label="Delete field"
              @click="fields.splice(index, 1)"
            />
          </div>
        </div>
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
