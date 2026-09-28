<script setup lang="ts">
import type { Student } from "@/types/student.ts"

defineProps<{
  student: Student
}>()
</script>

<template>
  <div class="flex items-center flex-wrap gap-1 mt-1">
    <UBadge
      v-for="award in student.awards"
      :key="`${award.round}-${award.award}`"
      variant="subtle"
      :icon="
        award.award === 'honorableMention' ? 'i-lucide-star' : 'i-lucide-medal'
      "
      :color="
        (
          {
            gold: 'warning',
            silver: 'neutral',
            bronze: 'primary',
            honorableMention: 'info',
          } as const
        )[award.award]
      "
      class="inline-flex items-center gap-1"
    >
      <span class="font-medium">{{ award.round }}</span>
      <span class="opacity-75">×{{ award.count }}</span>
    </UBadge>
  </div>
</template>
