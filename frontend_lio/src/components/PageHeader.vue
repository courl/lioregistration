<script setup lang="ts">
import type { Edition } from "@/types/edition.ts"

defineProps<{ title: string; edition?: Edition }>()
</script>

<template>
  <div>
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold">
        <template v-if="edition">
          <RouterLink
            class="hover:text-primary"
            :to="edition.active ? '/admin' : `/admin/edition/${edition.id}`"
            >{{ edition.name }}</RouterLink
          >
          ·
        </template>
        {{ title }}
      </h1>
      <div><slot name="header" /></div>
    </div>
    <div class="text-sm mb-6 mt-1">
      <slot name="subtitle" />
    </div>
  </div>
</template>
