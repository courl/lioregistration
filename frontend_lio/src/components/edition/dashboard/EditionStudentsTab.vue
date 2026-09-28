<script setup lang="ts">
import { computed } from "vue"
import { useRouter } from "vue-router"

import type { Edition } from "@/types/edition.ts"
import type { Student } from "@/types/student.ts"

import { useEditionStudents } from "@/composables/use-edition-students.ts"

const props = defineProps<{ edition: Edition }>()
const router = useRouter()
const { students, loading } = useEditionStudents(() => props.edition.id)

async function onSelect(student: Student) {
  await router.push(`/admin/students/${student.id.toString()}`)
}

const tabs = computed(() => {
  return [
    {
      icon: "i-lucide-user",
      text: `${students.value.length.toString()} total`,
    },
  ]
})
</script>

<template>
  <UCard :ui="{ body: 'p-0 sm:p-0' }">
    <template #header>
      <div class="flex items-center justify-between">
        <div>
          <span class="font-bold text-lg">Students</span>
          <IconTextBlock class="mt-1" :items="tabs" />
        </div>
      </div>
    </template>
    <StudentTable :students="students" :loading="loading" @select="onSelect" />
  </UCard>
</template>
