<script setup lang="ts">
import type { RegistrationDetails } from "@/types/registration.js"
import type { Student, StudentId } from "@/types/student.ts"

import { useConfirmDialog } from "@/composables/use-confirm-dialog.js"

defineProps<{ registration: RegistrationDetails; match?: [number, Student] }>()
const emit = defineEmits<{
  selectStudent: [StudentId | undefined]
  refuse: []
  linkAndValidate: [Student | undefined]
  unlink: []
}>()

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
</script>

<template>
  <UCard>
    <template #header>
      <span class="font-bold">Matching</span>
    </template>
    <template v-if="registration.review_status === 'refused'">
      <UAlert
        variant="subtle"
        icon="i-lucide-triangle-alert"
        title="Registration has been refused"
        description="This registration has been refused. "
      />
    </template>
    <template v-else>
      <template v-if="!registration.student">
        <RegistrationStudentMatchCard
          v-for="[score, student] in registration.matches"
          :key="student.id"
          class="mb-2"
          :student="student"
          :match="score"
          @link-and-validate="(x) => emit('linkAndValidate', x)"
        />

        <UFieldGroup orientation="vertical">
          <UInput
            type="number"
            placeholder="Enter student-id to link"
            min="0"
            class="w-full"
            @update:model-value="
              (value) =>
                emit(
                  'selectStudent',
                  value == '' ? undefined : (value as StudentId),
                )
            "
          />
          <RegistrationStudentMatchCard
            v-if="match"
            :student="match[1]"
            :match="match[0]"
            class="rounded-t-none"
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
      v-if="registration.review_status !== 'refused' && !registration.student"
      #footer
    >
      <UFieldGroup orientation="horizontal">
        <UButton
          icon="i-lucide-circle-x"
          variant="outline"
          color="error"
          @click="async () => await refuse()"
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
