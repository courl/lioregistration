<script setup lang="ts">
import type { Edition } from "@/types/edition.ts"

import { useConfirmDialog } from "@/composables/use-confirm-dialog.js"

defineProps<{ editions: Edition[] }>()
const emit = defineEmits<{
  deleteEdition: [edition: Edition]
  editEdition: [edition: Edition]
  activateEdition: [edition: Edition]
}>()
const confirm = useConfirmDialog()

async function handleActivate(edition: Edition) {
  if (
    await confirm({
      title: "Activate edition",
      description: `Are you sure you want to activate '${edition.name}'?`,
    })
  ) {
    emit("activateEdition", edition)
  }
}

async function handleDelete(edition: Edition) {
  if (
    await confirm({
      title: "Delete edition",
      description: `Are you sure you want to delete '${edition.name}'?`,
    })
  ) {
    emit("deleteEdition", edition)
  }
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <RouterCard
      v-for="edition in editions"
      :key="edition.id"
      :to="edition.active ? '/admin' : `/admin/editions/${edition.id}`"
      compact
    >
      <div class="flex items-center justify-between gap-3">
        <div class="min-w-0">
          <div class="mb-1 flex items-center justify-between">
            <span class="font-medium truncate">{{ edition.name }}</span>
            <UBadge v-if="edition.active" color="success" variant="subtle"
              >active</UBadge
            >
          </div>
          <IconTextBlock
            :items="[
              {
                icon: 'i-lucide-clipboard',
                text: `${edition.registration_opens_at.toString()} – ${edition.registration_closes_at.toString()}`,
              },
            ]"
          />
        </div>
        <UFieldGroup>
          <UButton
            variant="ghost"
            size="sm"
            icon="i-lucide-trash"
            color="error"
            @click="
              async (e) => {
                e.preventDefault()
                await handleDelete(edition)
              }
            "
          />
          <UButton
            variant="ghost"
            size="sm"
            icon="i-lucide-pencil"
            @click="
              async (e) => {
                e.preventDefault()
                emit('editEdition', edition)
              }
            "
          />
          <UButton
            v-if="!edition.active"
            variant="ghost"
            size="sm"
            icon="i-lucide-circle-check"
            color="success"
            @click="
              async (e) => {
                e.preventDefault()
                await handleActivate(edition)
              }
            "
          />
        </UFieldGroup>
      </div>
    </RouterCard>
  </div>
</template>
