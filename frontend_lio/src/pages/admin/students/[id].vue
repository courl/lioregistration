<script setup lang="ts">
import { useRouter } from "vue-router"

import type { StudentId } from "@/types/student.ts"

import { useRouteId } from "@/composables/use-route-id.js"
import { useStudent } from "@/composables/use-student.ts"

const router = useRouter()

const studentId = useRouteId<StudentId>()

const { student } = useStudent(studentId)
</script>

<template>
  <template v-if="student">
    <PageHeader :title="student.given_name + ' ' + student.family_name">
      <template #subtitle>
        <IconTextBlock
          :items="[
            { icon: 'i-lucide-mail', text: `${student.email}` },
            { icon: 'i-lucide-cake', text: `${student.birthdate}` },
          ]"
        />
      </template>
    </PageHeader>
    <UEmpty
      icon="i-lucide-triangle-alert"
      title="Not implemented"
      description="You are accessing a page that has not yet been implented."
    />
    <pre>
      {{ student }}
    </pre>
  </template>

  <UError
    v-else
    :error="{
      status: 404,
      statusMessage: 'Student not found',
      message: 'This student does not exist.',
    }"
    @clear="router.push('/admin/students')"
  />
</template>
