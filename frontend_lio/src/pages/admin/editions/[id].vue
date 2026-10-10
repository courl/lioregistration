<script setup lang="ts">
import type { EditionId } from "@/types/edition.ts"

import { useEdition } from "@/composables/use-edition.ts"
import { useRouteId } from "@/composables/use-route-id.ts"

const id = useRouteId<EditionId>()
const { edition, loading } = useEdition(id)
</script>

<template>
  <EditionDashboard v-if="edition" :edition="edition" />
  <UEmpty
    v-else-if="!loading"
    icon="i-lucide-book-open"
    title="Edition not found"
    description="This edition does not exist."
    :actions="[
      {
        icon: 'i-lucide-circle-arrow-right',
        label: 'Editions',
        to: '/admin/editions',
      },
    ]"
  />
</template>
