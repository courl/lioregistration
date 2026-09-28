<script setup lang="ts">
import { useRouter } from "vue-router"

import type { SchoolId } from "@/types/school.ts"

import { useRouteId } from "@/composables/use-route-id.ts"
import { useSchool } from "@/composables/use-school.ts"

const router = useRouter()

const id = useRouteId<SchoolId>()
const { school } = useSchool(id)
</script>

<template>
  <template v-if="school">
    <PageHeader :title="school.name" />
    <UEmpty
      icon="i-lucide-triangle-alert"
      title="No implemented"
      description="It looks like you are accessing a page that has not yet been implemented."
    />
  </template>

  <UError
    v-else
    :error="{
      status: 404,
      statusMessage: 'School not found',
      message: 'This school does not exist.',
    }"
    @clear="router.push('/admin/schools')"
  />
</template>
