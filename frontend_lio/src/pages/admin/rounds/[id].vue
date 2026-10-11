<script setup lang="ts">
import { computed, ref } from "vue"

import type { ParticipationData } from "@/components/rounds/modals/RoundParticipationModal.schema.ts"
import type { EmailTemplateId } from "@/types/email.ts"
import type { Participant, RoundId, TaskId } from "@/types/round.ts"

import { provideCanEdit } from "@/composables/use-can-edit.ts"
import { useRound, useRounds } from "@/composables/use-round.ts"
import { useRouteId } from "@/composables/use-route-id.ts"

const toast = useToast()
const roundId = useRouteId<RoundId>()
const {
  round,
  loading,
  createTask,
  updateTask,
  deleteTask,
  updateParticipation,
  bulkUpdateScores,
  sendEmail,
} = useRound(roundId)
const canEdit = provideCanEdit(() => round.value?.edition)
const { rounds, moveTo } = useRounds(() => round.value?.edition.id)

const state = ref("stats")

const tabs = computed(() => [
  { icon: "i-lucide-activity", label: "Statistics", value: "stats" },
  { icon: "i-lucide-logs", label: "Tasks", value: "tasks" },
  { icon: "i-lucide-user", label: "Scores", value: "participants" },
  ...(canEdit.value
    ? [
        {
          icon: "i-lucide-circle-arrow-right",
          label: "Move",
          value: "move",
        },
        { icon: "i-lucide-mail", label: "Mail", value: "comm" },
      ]
    : []),
])

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

async function processBulkUpdateScores(
  data: { credentials_username: string; task_scores: [TaskId, number][] }[],
) {
  const unmatched = await bulkUpdateScores(data)
  if (unmatched.length > 0) {
    toast.add({
      title: `${unmatched.length.toString()} rows were not imported`,
      description: `No participant of this round has the username: ${unmatched.join(", ")}`,
      icon: "i-lucide-triangle-alert",
      color: "warning",
      duration: 0,
    })
  } else {
    toast.add({ title: "Scores imported", color: "success" })
  }
}

async function processSendEmail(
  email: EmailTemplateId,
  headline: string,
  body: string,
) {
  try {
    await sendEmail(email, headline, body)
  } catch {
    // Already toasted by the API client
    return
  }
  toast.add({ title: "Emails queued for sending", color: "success" })
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
    <UTabs v-model="state" :items="tabs" />
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
        @bulk-update-score="processBulkUpdateScores"
        @update-participation="processUpdateParticipation"
      />
    </template>
    <template v-else-if="state === 'move' && canEdit">
      <RoundParticipantsMoveCard
        :participants="round.participants"
        :rounds="rounds.filter((x) => x.id !== round?.id)"
        @move-to="async (target, students) => await moveTo(target.id, students)"
      />
    </template>
    <template v-else-if="state === 'comm' && canEdit">
      <RoundEmailCard @send-mail="processSendEmail" />
    </template>
    <template v-else>
      <UEmpty
        icon="i-lucide-triangle-alert"
        title="Not implemented"
        description="Round statistics have not been implemented yet."
      />
      <JsonDump :value="round" />
    </template>
  </div>

  <UError
    v-else-if="!loading"
    :error="{
      status: 404,
      statusMessage: 'Round not found',
      message: 'This round does not exist.',
    }"
    :clear="{ label: 'Back to dashboard', to: '/admin' }"
  />
</template>
