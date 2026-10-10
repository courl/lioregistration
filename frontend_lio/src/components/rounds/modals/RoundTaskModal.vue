<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { reactive, watch } from "vue"

import {
  type TaskData,
  type TaskDataProps,
  type TaskModalResult,
  TaskSchema,
} from "@/components/rounds/modals/RoundTaskModal.schema.ts"

const props = defineProps<TaskDataProps>()

const emit = defineEmits<{ close: [TaskModalResult] }>()

const state = reactive<Partial<TaskData>>({})

watch(
  () => props.task,
  (task) => {
    Object.assign(state, {
      name: task?.name,
      ordinal: task?.ordinal,
      max_points: task?.max_points,
    })
  },
  { immediate: true },
)

function onSubmit(event: FormSubmitEvent<TaskData>) {
  emit("close", event.data)
}
</script>

<template>
  <UModal :title="title">
    <template #body>
      <UForm
        :schema="TaskSchema"
        :state="state"
        class="space-y-4"
        @submit="onSubmit"
      >
        <UFormField label="Name" name="name">
          <UInput v-model="state.name" class="w-full" />
        </UFormField>
        <div class="grid grid-cols-2 gap-4">
          <UFormField label="Max Points" name="max_points">
            <UInput
              v-model.number="state.max_points"
              type="number"
              min="0"
              class="w-full"
            />
          </UFormField>
          <UFormField label="Ordinal" name="ordinal">
            <UInput
              v-model.number="state.ordinal"
              type="number"
              min="0"
              class="w-full"
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
