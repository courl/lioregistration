<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui"

import { computed, h, ref, triggerRef, watch } from "vue"

import type { Participant, Round, RoundId } from "@/types/round.ts"
import type { StudentId } from "@/types/student.ts"

import ParticipationBadge from "@/components/rounds/ParticipationBadge.vue"
import { useConfirmDialog } from "@/composables/use-confirm-dialog.js"

const props = defineProps<{ participants: Participant[]; rounds: Round[] }>()
const emit = defineEmits<{
  moveTo: [round: Round, students: StudentId[]]
}>()

const confirm = useConfirmDialog()
const threshold = ref<number>()
const targetRoundId = ref<RoundId>()
const targetRound = computed(() =>
  props.rounds.find((x) => x.id === targetRoundId.value),
)
const selected = ref<boolean[]>([])
watch(
  () => props.participants,
  (participants) => {
    selected.value = Array.from({ length: participants.length }, () => false)
  },
  { immediate: true },
)

const selectedCount = computed(() => selected.value.filter(Boolean).length)

const columns = computed<TableColumn<Participant>[]>(() => [
  {
    id: "select",
  },
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
])

function applyThreshold() {
  for (const [index, participant] of props.participants.entries()) {
    selected.value[index] =
      threshold.value === undefined
        ? false
        : participant.total >= threshold.value
  }
  triggerRef(selected)
}

async function move() {
  if (selectedCount.value === 0 || targetRound.value === undefined) {
    return
  }
  const result = await confirm({
    title: "Are you sure?",
    description: `You are going to move ${selectedCount.value.toString()} students to ${targetRound.value.name}`,
  })

  if (result) {
    emit(
      "moveTo",
      targetRound.value,
      props.participants
        .filter((_, index) => selected.value[index])
        .map((x) => x.student.id),
    )
  }
}
</script>

<template>
  <UCard :ui="{ body: 'p-0 sm:p-0' }">
    <template #header>
      <div class="flex items-center justify-between">
        <span class="font-bold">Move participants</span>
        <span class="text-xs text-dimmed">click a student to select</span>
      </div>
    </template>
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 m-4">
      <div>
        <UFormField label="Select round to move students to" class="flex-1">
          <USelect
            v-model="targetRoundId"
            :items="rounds.map((x) => ({ label: x.name, value: x.id }))"
            class="w-full"
          />
        </UFormField>
      </div>
      <div class="flex items-end gap-2">
        <UFormField label="Preselect by score" class="flex-1">
          <UInput
            v-model="threshold"
            type="number"
            :min="0"
            class="w-full"
            placeholder="e.g. 150"
          />
        </UFormField>
        <UButton icon="i-lucide-filter" variant="subtle" @click="applyThreshold"
          >Preselect</UButton
        >
      </div>
    </div>
    <UTable
      :data="participants"
      :columns="columns"
      class="flex-1"
      @select="(_, row) => (selected[row.index] = !selected[row.index])"
    >
      <template #select-cell="{ row }">
        <UCheckbox v-model="selected[row.index]" />
      </template>
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
    <template #footer>
      <UButton
        icon="i-lucide-circle-arrow-right"
        :disabled="selectedCount === 0 || targetRound === undefined"
        @click="move"
        >Move {{ selectedCount }} students</UButton
      >
    </template>
  </UCard>
</template>
