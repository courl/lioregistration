<script setup lang="ts">
import { useEdition } from "@/composables/use-edition.ts"

const { edition } = useEdition("active")
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
