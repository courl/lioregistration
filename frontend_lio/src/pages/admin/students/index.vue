<script setup lang="ts">
import { ref } from "vue"
import { useRouter } from "vue-router"

import type { Student } from "@/types/student.ts"

import { useDebounced } from "@/composables/use-debounced.ts"
import { useStudents } from "@/composables/use-student.ts"

const router = useRouter()
const query = ref<string>("")
const { students, loading } = useStudents(useDebounced(query))

async function onSelect(student: Student) {
  await router.push(`/admin/students/${student.id.toString()}`)
}
</script>

<template>
  <PageHeader title="Students">
    <template #header>
      <UInput
        v-model="query"
        icon="i-lucide-search"
        placeholder="Search students..."
        class="max-w-xs"
      />
    </template>
    <template #subtitle> Every participant, across all editions. </template>
  </PageHeader>
  <StudentTable :students="students" :loading="loading" @select="onSelect" />
</template>
