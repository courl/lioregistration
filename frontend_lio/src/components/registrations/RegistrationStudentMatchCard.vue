<script setup lang="ts">
import type { Student } from "@/types/student.ts"

import StudentAwardsBlock from "@/components/students/StudentAwardsBlock.vue"
import { useCanEdit } from "@/composables/use-can-edit.ts"

defineProps<{ student: Student; match: number; matched?: boolean }>()
const emit = defineEmits<{ linkAndValidate: [Student] }>()
const canEdit = useCanEdit()
</script>

<template>
  <UCard
    variant="soft"
    class="relative overflow-hidden"
    :ui="{ body: 'p-3 sm:p-3' }"
  >
    <div class="flex items-center justify-between gap-3">
      <div class="min-w-0">
        <p class="font-medium text-highlighted truncate">
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
        v-if="canEdit && !matched"
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
