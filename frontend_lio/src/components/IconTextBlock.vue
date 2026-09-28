<script setup lang="ts">
const model = defineModel<string>()
defineProps<{ items: { icon: string; text: string; value?: string }[] }>()
</script>

<template>
  <div class="flex flex-wrap items-center gap-3 text-sm text-muted">
    <template v-for="(item, i) in items" :key="`icon-label-${i}`">
      <span
        :class="[
          'inline-flex',
          'items-center',
          'gap-1.5',
          'cursor-default',
          ...(item.value
            ? [model === item.value ? 'text-primary' : 'hover:text-white']
            : []),
        ]"
        @click="() => (item.value ? (model = item.value) : undefined)"
      >
        <UIcon :name="item.icon" class="size-4" />{{ item.text }}
      </span>
      <span v-if="i !== items.length - 1" class="h-4 w-px bg-accented" />
    </template>
  </div>
</template>
