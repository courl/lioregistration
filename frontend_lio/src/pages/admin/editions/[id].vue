<script setup lang="ts">
import type { EditionId } from "@/types/edition.ts"

import { useEdition } from "@/composables/use-edition.ts"
import { useRouteId } from "@/composables/use-route-id.js"

const id = useRouteId<EditionId>()
const { edition } = useEdition(id)
</script>

<template>
  <template v-if="edition === undefined">
    <UEmpty
      icon="i-lucide-book-open"
      title="No active edition"
      description="It looks like there is no active edition. Declare an edition as active to see the dashboard."
      :actions="[
        {
          icon: 'i-lucide-circle-arrow-right',
          label: 'Editions',
        },
        {
          icon: 'i-lucide-refresh-cw',
          label: 'Refresh',
          color: 'neutral',
          variant: 'subtle',
        },
      ]"
    />
  </template>
  <template v-else>
    <EditionDashboard :edition="edition" />
  </template>
</template>
