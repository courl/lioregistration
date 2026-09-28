<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { computed, reactive, useTemplateRef, watch } from "vue"

import type { ParticipationState } from "@/types/round.ts"

import {
  calculateParticipationSchema,
  type ParticipationData,
  type ParticipationProps,
  type ParticipationResult,
} from "@/components/rounds/modals/RoundParticipationModal.schema.ts"

const props = defineProps<ParticipationProps>()

const emit = defineEmits<{
  close: [ParticipationResult]
}>()

const schema = computed(() => calculateParticipationSchema(props.tasks))

const form = useTemplateRef("form")

const stateItems: {
  label: string
  value: ParticipationState
}[] = [
  { label: "Accepted", value: "accepted" },
  { label: "Invited", value: "invited" },
  { label: "Declined", value: "declined" },
  { label: "No Show", value: "noShow" },
]

// eslint-disable-next-line vue/no-setup-props-reactivity-loss
const state = reactive<ParticipationData>({
  state: props.participant.state,
  points: props.participant.points,
})

watch(
  () => props.participant,
  (participant) => {
    Object.assign(state, {
      state: participant.state,
      points: [...participant.points],
    })
  },
  { immediate: true },
)

function onSubmit(event: FormSubmitEvent<ParticipationData>) {
  emit("close", event.data)
}

const total = computed(() =>
  state.points.reduce((sum, value) => sum + value, 0),
)
</script>

<template>
  <UModal
    :title="
      title ??
      `${participant.student.given_name} ${participant.student.family_name}`
    "
  >
    <template #body>
      <UForm
        ref="form"
        :schema="schema"
        :state="state"
        class="space-y-4"
        @submit="onSubmit"
      >
        <UFormField label="Participation" name="state">
          <USelect v-model="state.state" :items="stateItems" class="w-full" />
        </UFormField>

        <USeparator />

        <div class="grid grid-cols-2 gap-2">
          <div v-for="(task, index) in tasks" :key="task.id">
            <UFormField :label="task.name" :name="`points.${index}`">
              <UInput
                v-model.number="state.points[index]"
                type="number"
                :min="0"
                :max="task.max_points"
                :disabled="state.state === 'declined'"
              />
              <span class="ml-1"> / {{ task.max_points }}</span>
            </UFormField>
          </div>
        </div>

        <div class="flex justify-between border-t pt-4">
          <span class="text-muted"> Total </span>

          <span class="font-semibold tabular-nums">
            {{ total }}
          </span>
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
          label="Save"
          color="primary"
          variant="subtle"
          @click="form?.submit()"
        />
      </UFieldGroup>
    </template>
  </UModal>
</template>
