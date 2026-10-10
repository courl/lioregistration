<script setup lang="ts">
import type { RouteLocationRaw } from "vue-router"

const props = defineProps<{ to: RouteLocationRaw; compact?: boolean }>()
</script>

<template>
  <RouterLink :to="props.to">
    <UCard
      class="hover:bg-muted"
      :ui="compact ? { body: 'p-3 sm:p-3' } : undefined"
    >
      <slot />
    </UCard>
  </RouterLink>
</template>
