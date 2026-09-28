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

const result = computed<RoundBulkScoreResult>(() => {
  if (!parsed.value) {
    return
  }

  const nameToTaskId = Object.fromEntries(
    parsed.value.headers
      .filter((x): x is typeof x & { taskId: TaskId } => x.taskId !== undefined)
      .map((x) => [x.name, x.taskId]),
  )
  const userColumn = parsed.value.headers.find((h) => h.isUserColumn)

  if (!userColumn) {
    return
  }

  const result: RoundScoreData[] = []
  for (const row of parsed.value.rows) {
    result.push({
      credentials_username: row[userColumn.name],
      task_scores: Object.entries(row)
        .filter(([key, _]) => key in nameToTaskId)
        .map(([key, value]) => [nameToTaskId[key], Number.parseFloat(value)]),
    })
  }

  return result
})

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
        <span class="text-muted mb-1">Add csv or space separated data</span>
        <UTextarea v-model="data" class="w-full" />
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
          :disabled="step === 'validate' ? !result : !data"
          @click="confirmOrValidate"
        />
      </UFieldGroup>
    </template>
  </UModal>
</template>
