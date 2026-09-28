<script setup lang="ts">
import type { Student } from "@/types/student.js"

import StudentAwardsBlock from "@/components/students/StudentAwardsBlock.vue"

defineProps<{ student: Student; match: number; matched?: boolean }>()
const emit = defineEmits<{ linkAndValidate: [Student] }>()
</script>

<template>
  <UCard
    variant="soft"
    class="relative overflow-hidden"
    :ui="{ body: 'p-3 sm:p-3' }"
  >
    <div class="flex items-center justify-between gap-3">
      <div class="min-w-0">
        <p class="font-medium text-highlighted truncated">
          <RouterLink
            :to="`/admin/students/${student.id}`"
            class="hover:text-primary"
            >{{ student.given_name }} {{ student.family_name }}</RouterLink
          >
        </p>
        <p class="text-sm text-muted truncate">{{ student.email }}</p>
        <p class="text-sm text-muted truncate">
          {{ student.birthdate }} {{ !matched ? `· ${match}% match` : "" }}
        </p>
      </div>

      <UButton
        v-if="!matched"
        variant="ghost"
        size="sm"
        icon="i-lucide-link"
        @click="emit('linkAndValidate', student)"
      >
        Link & Validate
      </UButton>
    </div>

    <StudentAwardsBlock :student="student" />

    <div class="absolute inset-x-0 bottom-0 h-1 bg-accented">
      <div class="h-full bg-primary" :style="{ width: `${match}%` }" />
    </div>
  </UCard>
</template>
