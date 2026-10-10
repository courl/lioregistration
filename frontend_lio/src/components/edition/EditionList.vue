<script setup lang="ts">
import type { Edition } from "@/types/edition.ts"

import { useAuth } from "@/composables/api.ts"
import { useConfirmDialog } from "@/composables/use-confirm-dialog.ts"

defineProps<{ editions: Edition[] }>()
const emit = defineEmits<{
  activateEdition: [edition: Edition]
}>()
const auth = useAuth()
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
        <UButton
          v-if="auth.isAdmin && !edition.active"
          variant="ghost"
          size="sm"
          icon="i-lucide-circle-check"
          color="success"
          aria-label="Activate edition"
          @click.prevent="async () => await handleActivate(edition)"
        />
      </div>
    </RouterCard>
  </div>
</template>
