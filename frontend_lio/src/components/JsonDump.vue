<script setup lang="ts">
withDefaults(defineProps<{ value: unknown; label?: string }>(), {
  label: "Raw data",
})
</script>

<template>
  <UCollapsible class="mt-4">
    <UButton
      :label="label"
      color="neutral"
      variant="subtle"
      icon="i-lucide-braces"
      trailing-icon="i-lucide-chevron-down"
      block
      :ui="{ trailingIcon: 'ms-auto' }"
    />
    <template #content>
      <pre
        class="mt-2 max-h-[70vh] overflow-auto rounded-md border border-muted bg-elevated p-4 text-xs"
        >{{ JSON.stringify(value, null, 2) }}</pre>
    </template>
  </UCollapsible>
</template>
