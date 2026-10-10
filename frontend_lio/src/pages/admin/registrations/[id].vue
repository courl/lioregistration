<script setup lang="ts">
import { ref } from "vue"

import type { RegistrationId, ReviewStatus } from "@/types/registration.ts"
import type { Student, StudentId } from "@/types/student.ts"

import { useRegistration } from "@/composables/use-registration.ts"
import { useRouteId } from "@/composables/use-route-id.ts"

const id = useRouteId<RegistrationId>()
const {
  registration,
  loading,
  assignCredentials,
  unassignCredentials,
  linkAndValidate,
  unlink,
  refuse,
  loadCandidate,
} = useRegistration(id)

const match = ref<[number, Student] | undefined>()

async function processLinkAndValidate(student: Student | undefined) {
  await linkAndValidate(student)
  match.value = undefined
}

let candidateRequest = 0
async function selectStudent(studentId: StudentId | undefined) {
  const current = ++candidateRequest
  // Hide the previous candidate at once, so it can't be linked while loading
  match.value = undefined
  if (studentId === undefined) return
  const candidate = await loadCandidate(studentId)
  // Ignore responses overtaken by a newer id
  if (current === candidateRequest) match.value = candidate
}

const reviewStatusColor = (
  status: ReviewStatus,
): "primary" | "success" | "error" => {
  if (status === "validated") return "success"
  else if (status === "refused") return "error"
  return "primary"
}
</script>

<template>
  <div v-if="registration">
    <PageHeader
      :title="`${registration.given_name} ${registration.family_name}`"
      :edition="registration.edition"
    >
      <template #header>
        <UBadge
          variant="subtle"
          :color="reviewStatusColor(registration.review_status)"
        >
          {{ registration.review_status }}
        </UBadge>
      </template>
      <template #subtitle>
        Registered on {{ registration.registration_date }}
      </template>
    </PageHeader>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <RegistrationDataCard
          :registration="registration"
          @assign-credentials="assignCredentials"
          @unassign-credentials="unassignCredentials"
        />
      </div>
      <div>
        <RegistrationMatchingCard
          :registration="registration"
          :match="match"
          @select-student="selectStudent"
          @refuse="refuse"
          @link-and-validate="processLinkAndValidate"
          @unlink="unlink"
        />
      </div>
    </div>
  </div>

  <UError
    v-else-if="!loading"
    :error="{
      status: 404,
      statusMessage: 'Registration not found',
      message: 'This registration does not exist.',
    }"
    :clear="{ label: 'Back to dashboard', to: '/admin' }"
  />
</template>
