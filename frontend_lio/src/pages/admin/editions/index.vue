<script setup lang="ts">
import type { EditionModalProps } from "@/components/edition/modals/EditionModal.schema.ts"
import type { Edition } from "@/types/edition.ts"

import { useEditionModal } from "@/composables/use-edition-modal.ts"
import { useEditions } from "@/composables/use-edition.ts"

const { editions, activateEdition, createEdition } = useEditions()
const editionModal = useEditionModal()

async function handleActivateEdition(edition: Edition) {
  await activateEdition(edition.id)
}

async function openDialog(props: EditionModalProps) {
  const result = await editionModal(props)
  if (result) {
    await createEdition(result)
  }
}
</script>

<template>
  <PageHeader title="Editions">
    <template #header>
      <UButton
        icon="i-lucide-plus"
        @click="openDialog({ title: 'New edition' })"
        >Add edition</UButton
      >
    </template>
  </PageHeader>
  <EditionList :editions="editions" @activate-edition="handleActivateEdition" />
</template>
