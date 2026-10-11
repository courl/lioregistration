<script setup lang="ts">
import type { RoundModalProps } from "@/components/rounds/modals/RoundModal.schema.ts"
import type { Edition } from "@/types/edition.ts"
import type { Round, RoundId } from "@/types/round.ts"

import { useCanEdit } from "@/composables/use-can-edit.ts"
import { useRoundModal } from "@/composables/use-round-modal.ts"
import { useRounds } from "@/composables/use-round.ts"

const props = defineProps<{ edition: Edition }>()
const canEdit = useCanEdit()
const { rounds, createRound, updateRound, deleteRound } = useRounds(
  () => props.edition.id,
)
const roundDialog = useRoundModal()

async function openRoundDialog(props: RoundModalProps, id?: RoundId) {
  const result = await roundDialog(props)
  if (result) {
    if (id) {
      await updateRound({ id, ...result })
    } else {
      await createRound(result)
    }
  }
}

// RoundsList already asked for confirmation
async function processDeleteRound(round: Round) {
  await deleteRound(round.id)
}
</script>

<template>
  <UCard>
    <template #header>
      <div class="flex items-center justify-between">
        <div>
          <span class="font-bold text-lg">Rounds</span>
          <IconTextBlock
            class="mt-1"
            :items="[
              { icon: 'i-lucide-layers', text: `${rounds.length} total` },
            ]"
          />
        </div>
        <UButton
          v-if="canEdit"
          icon="i-lucide-plus"
          variant="subtle"
          @click="async () => await openRoundDialog({ title: 'New Round' })"
          >Add round</UButton
        >
      </div>
    </template>
    <RoundsList
      :rounds="rounds"
      @edit-round="
        async (round) =>
          await openRoundDialog(
            { title: `Edit '${round.name}'`, round },
            round.id,
          )
      "
      @delete-round="processDeleteRound"
    />
  </UCard>
</template>
