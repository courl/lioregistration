<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui"

import { h } from "vue"

import type { ParticipationData } from "@/components/rounds/modals/RoundParticipationModal.schema.ts"
import type { Participant, RoundDetails, TaskId } from "@/types/round.ts"

import ParticipationBadge from "@/components/rounds/ParticipationBadge.vue"
import { useRoundBulkScoreModal } from "@/composables/use-round-bulk-score-modal.ts"
import { useRoundParticipationModal } from "@/composables/use-round-participation-modal.ts"

const props = defineProps<{ round: RoundDetails }>()
const emit = defineEmits<{
  updateParticipation: [Participant, ParticipationData]
  bulkUpdateScore: [
    { credentials_username: string; task_scores: [TaskId, number][] }[],
  ]
}>()
const participationEditor = useRoundParticipationModal()
const bulkScoreEditor = useRoundBulkScoreModal()

const columns: TableColumn<Participant>[] = [
  {
    id: "state",
  },
  {
    id: "name",
    header: "Student",
  },
  {
    id: "total",
    header: () => h("div", { class: "text-right" }, "Total"),
  },
]

async function editParticipation(participant: Participant) {
  const result = await participationEditor({
    participant,
    tasks: props.round.tasks,
  })
  if (result) {
    emit("updateParticipation", participant, result)
  }
}
async function editScores() {
  const result = await bulkScoreEditor({ tasks: props.round.tasks })
  if (result) {
    const tasks = Object.fromEntries(
      props.round.tasks.map((x) => [x.name, x.id]),
    )
    emit(
      "bulkUpdateScore",
      result.map((x) => ({
        ...x,
        task_scores: x.task_scores.map((x) => [tasks[x[0]], x[1]]),
      })),
    )
  }
}
</script>

<template>
  <UCard :ui="{ body: 'p-0 sm:p-0' }">
    <template #header>
      <div class="flex items-center justify-between">
        <span class="font-bold">Scores</span>
        <UButton size="xs" icon="i-lucide-upload" @click="editScores"
          >Add scores</UButton
        >
      </div>
    </template>
    <UTable
      :data="round.participants"
      :columns="columns"
      class="flex-1"
      @select="async (_, row) => await editParticipation(row.original)"
    >
      <template #state-cell="{ row }">
        <ParticipationBadge :state="row.original.state" />
      </template>
      <template #name-cell="{ row }">
        {{ row.original.student.given_name }}
        {{ row.original.student.family_name }}
      </template>
      <template #total-cell="{ row }">
        <div class="text-right font-semibold tabular-nums">
          {{ row.original.total }}
        </div>
      </template>
    </UTable>
  </UCard>
</template>
