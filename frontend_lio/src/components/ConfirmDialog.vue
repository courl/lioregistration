<script setup lang="ts">
import type { ConfirmDialogProps } from "@/components/ConfirmDialog.schema.ts"

defineProps<ConfirmDialogProps>()

const emit = defineEmits<{
  close: [value: boolean]
  other: []
}>()
</script>

<template>
  <UModal
    :title="title"
    :description="description"
    :ui="{ footer: 'justify-end' }"
  >
    <template #footer>
      <UFieldGroup>
        <UButton
          label="Cancel"
          color="error"
          variant="outline"
          @click="emit('close', false)"
        />
        <UButton
          label="Confirm"
          color="primary"
          variant="subtle"
          @click="emit('close', true)"
        />
      </UFieldGroup>
    </template>
  </UModal>
</template>
