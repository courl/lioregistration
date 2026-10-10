<script setup lang="ts">
import { useEdition } from "@/composables/use-edition.ts"

const { edition, loading, refresh } = useEdition("active")
</script>

<template>
  <EditionDashboard v-if="edition" :edition="edition" />
  <UEmpty
    v-else-if="!loading"
    icon="i-lucide-book-open"
    title="No active edition"
    description="It looks like there is no active edition. Declare an edition as active to see the dashboard."
    :actions="[
      {
        icon: 'i-lucide-circle-arrow-right',
        label: 'Editions',
        to: '/admin/editions',
      },
      {
        icon: 'i-lucide-refresh-cw',
        label: 'Refresh',
        color: 'neutral',
        variant: 'subtle',
        onClick: refresh,
      },
    ]"
  />
</template>
