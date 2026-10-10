<script setup lang="ts">
import type { SchoolId } from "@/types/school.ts"

import { useRouteId } from "@/composables/use-route-id.ts"
import { useSchool } from "@/composables/use-school.ts"

const id = useRouteId<SchoolId>()
const { school, loading } = useSchool(id)
</script>

<template>
  <template v-if="school">
    <PageHeader :title="school.name" />
    <UEmpty
      icon="i-lucide-triangle-alert"
      title="Not implemented"
      description="The school page has not been implemented yet."
    />
    <JsonDump :value="school" />
  </template>

  <UError
    v-else-if="!loading"
    :error="{
      status: 404,
      statusMessage: 'School not found',
      message: 'This school does not exist.',
    }"
    :clear="{ label: 'Back to schools', to: '/admin/schools' }"
  />
</template>
