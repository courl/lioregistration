<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { reactive, useTemplateRef, watch } from "vue"

import {
  type TaskData,
  type TaskDataProps,
  type TaskModalResult,
  TaskSchema,
} from "./RoundTaskModal.schema"

const props = defineProps<TaskDataProps>()

const emit = defineEmits<{ close: [TaskModalResult] }>()

const state = reactive<Partial<TaskData>>({})
const form = useTemplateRef("form")

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
  <UModal :title="title" :ui="{ footer: 'justify-end' }">
    <template #body>
      <UForm
        ref="form"
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
            <UInput v-model="state.max_points" type="number" min="0" />
          </UFormField>
          <UFormField label="Ordinal" name="ordinal">
            <UInput v-model="state.ordinal" type="number" min="0" />
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
