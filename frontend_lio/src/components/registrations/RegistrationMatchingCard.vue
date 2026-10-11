<script setup lang="ts">
import type { RegistrationDetails } from "@/types/registration.ts"
import type { Student, StudentId } from "@/types/student.ts"

import { useCanEdit } from "@/composables/use-can-edit.ts"
import { useConfirmDialog } from "@/composables/use-confirm-dialog.ts"

defineProps<{ registration: RegistrationDetails; match?: [number, Student] }>()
const emit = defineEmits<{
  selectStudent: [StudentId | undefined]
  refuse: []
  linkAndValidate: [Student | undefined]
  unlink: []
}>()

const canEdit = useCanEdit()
const confirm = useConfirmDialog()

async function refuse() {
  if (
    await confirm({
      title: `Refuse registration`,
      description: `Are you sure you want to refuse this registration?`,
    })
  ) {
    emit("refuse")
  }
}

async function unlink() {
  if (
    await confirm({
      title: `Unlink student`,
      description: `Unlinking removes the student's participations in this edition. Continue?`,
    })
  ) {
    emit("unlink")
  }
}

function selectStudent(value: unknown) {
  const id = Number(value)
  emit(
    "selectStudent",
    value !== "" && value !== null && Number.isInteger(id) && id > 0
      ? (id as StudentId)
      : undefined,
  )
}
</script>

<template>
  <UCard>
    <template #header>
      <div class="flex items-center justify-between">
        <span class="font-bold">Matching</span>
        <UButton
          v-if="canEdit && registration.student"
          size="xs"
          icon="i-lucide-unlink"
          variant="ghost"
          color="error"
          @click="unlink"
          >Unlink</UButton
        >
      </div>
    </template>
    <template v-if="registration.review_status === 'refused'">
      <UAlert
        variant="subtle"
        icon="i-lucide-triangle-alert"
        title="Registration has been refused"
        description="This registration has been refused."
      />
    </template>
    <template v-else>
      <template v-if="!registration.student">
        <RegistrationStudentMatchCard
          v-for="[score, student] in registration.matches ?? []"
          :key="student.id"
          class="mb-2"
          :student="student"
          :match="score"
          @link-and-validate="(x) => emit('linkAndValidate', x)"
        />
        <p
          v-if="(registration.matches ?? []).length === 0"
          class="text-sm text-muted mb-2"
        >
          No matching students found.
        </p>

        <UFieldGroup v-if="canEdit" orientation="vertical" class="w-full">
          <UInput
            type="number"
            placeholder="Enter student-id to link"
            min="1"
            class="w-full"
            @update:model-value="selectStudent"
          />
          <RegistrationStudentMatchCard
            v-if="match"
            :student="match[1]"
            :match="match[0]"
            class="rounded-t-none"
            @link-and-validate="(x) => emit('linkAndValidate', x)"
          />
        </UFieldGroup>
      </template>
      <template v-else>
        <RegistrationStudentMatchCard
          :student="registration.student"
          :match="100"
          matched
        />
      </template>
    </template>
    <template
      v-if="
        canEdit &&
        registration.review_status !== 'refused' &&
        !registration.student
      "
      #footer
    >
      <UFieldGroup orientation="horizontal">
        <UButton
          icon="i-lucide-circle-x"
          variant="outline"
          color="error"
          @click="refuse"
          >Refuse</UButton
        >
        <UButton
          icon="i-lucide-plus"
          variant="subtle"
          @click="emit('linkAndValidate', undefined)"
          >Create new</UButton
        >
      </UFieldGroup>
    </template>
  </UCard>
</template>
