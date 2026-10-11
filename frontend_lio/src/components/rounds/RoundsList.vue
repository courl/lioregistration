<script setup lang="ts">
import { useCanEdit } from "@/composables/use-can-edit.ts"
import { useConfirmDialog } from "@/composables/use-confirm-dialog.ts"
import { type Round } from "@/types/round.ts"

defineProps<{ rounds: Round[] }>()
const emit = defineEmits<{
  deleteRound: [round: Round]
  editRound: [round: Round]
}>()
const canEdit = useCanEdit()
const confirm = useConfirmDialog()

async function handleDelete(round: Round) {
  if (
    await confirm({
      title: "Delete round",
      description: `Are you sure you want to delete '${round.name}'?`,
    })
  ) {
    emit("deleteRound", round)
  }
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <RouterCard
      v-for="round in rounds"
      :key="round.id"
      :to="`/admin/rounds/${round.id}`"
      compact
    >
      <div class="flex items-center justify-between gap-3">
        <div class="min-w-0">
          <div class="mb-1">
            <span class="font-medium truncate">{{ round.name }}</span>
          </div>
          <IconTextBlock
            :items="[
              {
                icon: 'i-lucide-calendar',
                text: `${round.starts_at.toString()} – ${round.ends_at.toString()}`,
              },
              {
                icon: 'i-lucide-target',
                text: `${round.total_points} pts`,
              },
              { icon: 'i-lucide-logs', text: `${round.task_count} tasks` },
            ]"
          />
        </div>
        <UFieldGroup v-if="canEdit">
          <UButton
            variant="ghost"
            size="sm"
            icon="i-lucide-trash"
            aria-label="Delete round"
            color="error"
            @click.prevent="async () => await handleDelete(round)"
          />
          <UButton
            variant="ghost"
            size="sm"
            icon="i-lucide-pencil"
            aria-label="Edit round"
            @click.prevent="emit('editRound', round)"
          />
        </UFieldGroup>
      </div>
    </RouterCard>
  </div>
</template>
