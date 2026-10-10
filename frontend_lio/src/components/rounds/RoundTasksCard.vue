<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui"

import { computed } from "vue"

import type { TaskData } from "@/components/rounds/modals/RoundTaskModal.schema.ts"
import type { Task } from "@/types/round.ts"

import { useAuth } from "@/composables/api.ts"
import { useConfirmDialog } from "@/composables/use-confirm-dialog.ts"
import { useRoundTaskModal } from "@/composables/use-round-task-modal.ts"

defineProps<{ tasks: Task[] }>()
const emit = defineEmits<{
  newTask: [data: TaskData]
  editTask: [task: Task, data: TaskData]
  deleteTask: [task: Task]
}>()

const auth = useAuth()
const taskDialog = useRoundTaskModal()
const confirm = useConfirmDialog()

const columns = computed<TableColumn<Task>[]>(() => [
  {
    header: "",
    accessorKey: "ordinal",
  },
  {
    header: "Task",
    accessorKey: "name",
  },
  {
    header: "Max Points",
    accessorKey: "max_points",
  },
  ...(auth.isAdmin ? [{ id: "actions" }] : []),
])

async function deleteTask(task: Task) {
  const result = await confirm({
    title: "Delete task",
    description: `Are you sure you want to delete task '${task.name}'?`,
  })
  if (result) {
    emit("deleteTask", task)
  }
}

async function editTask(task: Task) {
  const result = await taskDialog({ title: `Edit '${task.name}'`, task })
  if (result) {
    emit("editTask", task, result)
  }
}

async function newTask() {
  const result = await taskDialog({ title: `New Task` })
  if (result) {
    emit("newTask", result)
  }
}
</script>

<template>
  <UCard :ui="{ body: 'p-0 sm:p-0' }">
    <template #header>
      <div class="flex items-center justify-between">
        <span class="font-bold">Tasks</span>
        <UButton
          v-if="auth.isAdmin"
          size="xs"
          icon="i-lucide-plus"
          @click="newTask"
          >New task</UButton
        >
      </div>
    </template>
    <UTable :data="tasks" :columns="columns">
      <template #actions-cell="{ row }">
        <div class="flex justify-end items-center">
          <UFieldGroup>
            <UButton
              variant="ghost"
              size="sm"
              icon="i-lucide-trash"
              color="error"
              @click="async () => await deleteTask(row.original)"
            />
            <UButton
              variant="ghost"
              size="sm"
              icon="i-lucide-pencil"
              @click="async () => await editTask(row.original)"
            />
          </UFieldGroup>
        </div>
      </template>
    </UTable>
  </UCard>
</template>
