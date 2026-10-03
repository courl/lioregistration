<script setup lang="ts">
import { ref } from "vue"
import { useRouter } from "vue-router"

import type { ParticipationData } from "@/components/rounds/modals/RoundParticipationModal.schema.ts"

import RoundEmailCard from "@/components/rounds/RoundEmailCard.vue"
import { useRound, useRounds } from "@/composables/use-round.ts"
import { useRouteId } from "@/composables/use-route-id.js"
import { type Participant, type RoundId } from "@/types/round.ts"

const router = useRouter()

const roundId = useRouteId<RoundId>()
const {
  round,
  createTask,
  updateTask,
  deleteTask,
  updateParticipation,
  bulkUpdateScores,
} = useRound(roundId)
const { rounds, moveTo } = useRounds(() => round.value?.edition.id)

const state = ref("stats")

async function processUpdateParticipation(
  participant: Participant,
  data: ParticipationData,
) {
  if (round.value === undefined) {
    return
  }
  await updateParticipation(participant.student.id, {
    state: data.state,
    points: round.value.tasks.map((task, index) => [
      task.id,
      data.points[index],
    ]),
  })
}
</script>

<template>
  <div v-if="round">
    <PageHeader :title="round.name" :edition="round.edition">
      <template #subtitle>
        <IconTextBlock
          :items="[
            {
              icon: 'i-lucide-calendar',
              text: `${round.starts_at.toString()} – ${round.ends_at.toString()}`,
            },
            { icon: 'i-lucide-target', text: `${round.total_points} pts` },
            { icon: 'i-lucide-logs', text: `${round.task_count} tasks` },
            {
              icon: 'i-lucide-users',
              text: `${round.participant_count} participants`,
            },
          ]"
        />
      </template>
    </PageHeader>
    <UTabs
      v-model="state"
      :items="[
        { icon: 'i-lucide-activity', label: 'Statistics', value: 'stats' },
        { icon: 'i-lucide-logs', label: 'Tasks', value: 'tasks' },
        { icon: 'i-lucide-user', label: 'Scores', value: 'participants' },
        { icon: 'i-lucide-circle-arrow-right', label: 'Move', value: 'move' },
        { icon: 'i-lucide-mail', label: 'Mail', value: 'comm' },
      ]"
    />
    <template v-if="state === 'tasks'">
      <RoundTasksCard
        :tasks="round.tasks"
        @new-task="createTask"
        @edit-task="
          async (task, data) => await updateTask({ id: task.id, ...data })
        "
        @delete-task="async (task) => await deleteTask(task.id)"
      />
    </template>
    <template v-else-if="state === 'participants'">
      <RoundScoresCard
        :round="round"
        @bulk-update-score="bulkUpdateScores"
        @update-participation="processUpdateParticipation"
      />
    </template>
    <template v-else-if="state === 'move'">
      <RoundParticipantsMoveCard
        :participants="round.participants"
        :rounds="rounds.filter((x) => (round ? x.id != round.id : true))"
        @move-to="async (round, students) => await moveTo(round.id, students)"
      />
    </template>
    <template v-else-if="state === 'comm'">
      <RoundEmailCard />
    </template>
    <template v-else>
      <UEmpty
        icon="i-lucide-triangle-alert"
        title="No implemented"
        description="It looks like you are accessing a page that has not yet been implemented."
      />
    </template>
  </div>

  <UError
    v-else
    :error="{
      status: 404,
      statusMessage: 'Round not found',
      message: 'This round does not exist.',
    }"
    @clear="router.push('/admin/rounds')"
  />
</template>
