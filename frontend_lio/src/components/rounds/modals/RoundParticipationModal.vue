<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { computed, reactive, watch } from "vue"

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

const schema = computed(() =>
  calculateParticipationSchema(props.tasks, props.participant.points),
)

const stateItems: {
  label: string
  value: ParticipationState
}[] = [
  { label: "Accepted", value: "accepted" },
  { label: "Invited", value: "invited" },
  { label: "Declined", value: "declined" },
  { label: "No Show", value: "noShow" },
]

const state = reactive<ParticipationData>({ state: "invited", points: [] })

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

function setPoints(index: number, value: unknown) {
  const points = ["", null, undefined].includes(value as string)
    ? null
    : Number(value)
  state.points[index] = Number.isFinite(points) ? points : null
}

function onSubmit(event: FormSubmitEvent<ParticipationData>) {
  emit("close", event.data)
}

const total = computed(() =>
  state.points.reduce<number>((sum, value) => sum + (value ?? 0), 0),
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
                :model-value="state.points[index] ?? undefined"
                type="number"
                step="any"
                placeholder="Not scored"
                :min="0"
                :max="task.max_points"
                :disabled="state.state === 'declined'"
                @update:model-value="(value) => setPoints(index, value)"
              />
              <span class="ml-1"> / {{ task.max_points }}</span>
            </UFormField>
          </div>
        </div>

        <div class="flex justify-between border-t border-default pt-4">
          <span class="text-muted"> Total </span>

          <span class="font-semibold tabular-nums">
            {{ total }}
          </span>
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
              label="Save"
              color="primary"
              variant="subtle"
            />
          </UFieldGroup>
        </div>
      </UForm>
    </template>
  </UModal>
</template>
