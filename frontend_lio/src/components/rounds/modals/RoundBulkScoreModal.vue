<script setup lang="ts">
import { computed, ref } from "vue"

import type { TaskId } from "@/types/round.ts"

import {
  type ParsedData,
  type ParsedHeader,
  parseInput,
  type RoundBulkScoreProps,
  type RoundBulkScoreResult,
  type RoundScoreData,
} from "@/components/rounds/modals/RoundBulkScoreModal.ts"

const props = defineProps<RoundBulkScoreProps>()
const emit = defineEmits<{ close: [RoundBulkScoreResult] }>()
const data = ref<string>("")
const step = ref<"data" | "validate">("data")
const parsed = ref<ParsedData>()

/** A score cell, accepting a decimal comma ("7,5"). NaN when it doesn't fully parse. */
function parseScore(value: string): number {
  return Number(value.trim().replace(",", "."))
}

/** The scores of one row's task columns, reporting cells that aren't numbers. */
function rowScores(
  row: Record<string, string>,
  nameToTaskId: Record<string, TaskId>,
  onInvalid: (key: string, value: string) => void,
): [TaskId, number][] {
  const task_scores: [TaskId, number][] = []
  for (const [key, value] of Object.entries(row)) {
    // Empty cells are left unscored
    if (!Object.hasOwn(nameToTaskId, key) || value.trim() === "") continue
    const score = parseScore(value)
    if (Number.isFinite(score)) {
      task_scores.push([nameToTaskId[key], score])
    } else {
      onInvalid(key, value)
    }
  }
  return task_scores
}

const evaluation = computed<{
  result: RoundBulkScoreResult
  invalid: string[]
}>(() => {
  const invalid: string[] = []
  if (!parsed.value) {
    return { result: undefined, invalid }
  }

  const nameToTaskId = Object.fromEntries(
    parsed.value.headers
      .filter((x): x is typeof x & { taskId: TaskId } => x.taskId !== undefined)
      .map((x) => [x.name, x.taskId]),
  )
  const userColumn = parsed.value.headers.find((h) => h.isUserColumn)

  if (!userColumn) {
    return { result: undefined, invalid }
  }

  const result: RoundScoreData[] = []
  for (const row of parsed.value.rows) {
    const username = row[userColumn.name]
    if (!username) continue
    result.push({
      credentials_username: username,
      task_scores: rowScores(row, nameToTaskId, (key, value) =>
        invalid.push(`${username} / ${key}: '${value}'`),
      ),
    })
  }

  // Never import a partially understood file
  return {
    result: invalid.length === 0 && result.length > 0 ? result : undefined,
    invalid,
  }
})
const result = computed(() => evaluation.value.result)

function confirmOrValidate() {
  if (step.value === "validate") {
    emit("close", result.value)
  } else {
    parsed.value = parseInput(data.value, props.tasks)
    step.value = "validate"
  }
}

function toggleUserColumn(header: ParsedHeader) {
  if (!parsed.value) {
    return
  }
  for (const h of parsed.value.headers) {
    h.isUserColumn = false
  }
  header.isUserColumn = true
}

function toggleTaskColumn(header: ParsedHeader, taskId: TaskId) {
  if (!parsed.value) {
    return
  }
  for (const h of parsed.value.headers) {
    if (h.taskId === taskId) {
      h.taskId = undefined
    }
  }
  header.taskId = taskId
}

function removeHeader(header: ParsedHeader) {
  header.isUserColumn = false
  header.taskId = undefined
}
</script>

<template>
  <UModal title="Add scores" :ui="{ footer: 'justify-end' }">
    <template #body>
      <template v-if="step === 'data'">
        <p class="text-muted mb-1">
          Paste CSV or space separated data. The first line is the header: one
          column with the contest usernames, one column per task name.
        </p>
        <UTextarea v-model="data" class="w-full font-mono" :rows="8" />
      </template>
      <template v-if="step === 'validate' && parsed"
        ><div>
          <span class="text-xs font-medium uppercase tracking-wide text-muted"
            >Columns</span
          >
          <div class="flex flex-col gap-1 mt-2">
            <div
              v-for="header in parsed.headers"
              :key="header.name"
              class="flex items-center gap-2 px-2.5 py-1.5 rounded-md border border-default transition-colors"
              :class="
                header.isUserColumn || header.taskId !== undefined
                  ? 'bg-default'
                  : 'bg-default/50 opacity-60'
              "
            >
              <span class="flex-1 text-sm truncate">
                {{ header.name }}
                <UBadge
                  v-if="header.isUserColumn"
                  size="xs"
                  variant="soft"
                  color="primary"
                  class="ml-1.5"
                >
                  user
                </UBadge>
              </span>
              <USelect
                class="flex-1"
                :model-value="header.isUserColumn ? 'user' : header.taskId"
                variant="ghost"
                size="xs"
                :items="[
                  [{ label: 'User', value: 'user' }],
                  tasks.map((x) => ({ label: x.name, value: x.id })),
                ]"
                @update:model-value="
                  (x: TaskId | string) => {
                    if (x === 'user') {
                      toggleUserColumn(header)
                    } else if (typeof x !== 'string') {
                      toggleTaskColumn(header, x)
                    }
                  }
                "
              >
              </USelect>
              <UButton
                size="xs"
                variant="link"
                color="neutral"
                :disabled="!header.isUserColumn && header.taskId === undefined"
                icon="i-lucide-x"
                @click="() => removeHeader(header)"
              />
            </div>
          </div>

          <UAlert
            v-if="evaluation.invalid.length > 0"
            class="mt-3"
            color="error"
            variant="subtle"
            icon="i-lucide-triangle-alert"
            title="Some scores are not numbers"
            :description="evaluation.invalid.join(', ')"
          />

          <UTable
            class="mt-3"
            :data="parsed.rows"
            :columns="
              parsed.headers
                .filter((h) => h.isUserColumn || h.taskId !== undefined)
                .map((h) => ({ accessorKey: h.name, header: h.name }))
            "
          />
        </div>
      </template>
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
          :label="step === 'validate' ? 'Confirm' : 'Next'"
          color="primary"
          variant="subtle"
          :disabled="step === 'validate' ? !result : !data.trim()"
          @click="confirmOrValidate"
        />
      </UFieldGroup>
    </template>
  </UModal>
</template>
