<script setup lang="ts">
import type { StudentId } from "@/types/student.ts"

import { useRouteId } from "@/composables/use-route-id.ts"
import { useStudent } from "@/composables/use-student.ts"

const studentId = useRouteId<StudentId>()
const { student, loading } = useStudent(studentId)
</script>

<template>
  <template v-if="student">
    <PageHeader :title="`${student.given_name} ${student.family_name}`">
      <template #subtitle>
        <IconTextBlock
          :items="[
            { icon: 'i-lucide-mail', text: student.email },
            { icon: 'i-lucide-cake', text: student.birthdate.toString() },
          ]"
        />
      </template>
    </PageHeader>
    <UEmpty
      icon="i-lucide-triangle-alert"
      title="Not implemented"
      description="The student page has not been implemented yet."
    />
    <JsonDump :value="student" />
  </template>

  <UError
    v-else-if="!loading"
    :error="{
      status: 404,
      statusMessage: 'Student not found',
      message: 'This student does not exist.',
    }"
    :clear="{ label: 'Back to students', to: '/admin/students' }"
  />
</template>
