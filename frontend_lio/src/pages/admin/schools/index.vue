<script setup lang="ts">
import { ref } from "vue"
import { useRouter } from "vue-router"

import { useSchools } from "@/composables/use-school.ts"

const globalFilter = ref("")
const { schools, mergeSchools } = useSchools(globalFilter)
const router = useRouter()
</script>

<template>
  <PageHeader title="Schools">
    <template #header>
      <UInput
        v-model="globalFilter"
        icon="i-lucide-search"
        placeholder="Search schools..."
        class="max-w-xs"
      />
    </template>
  </PageHeader>
  <SchoolsTable
    :schools="schools"
    @select="(school) => router.push(`/admin/schools/${school.id}`)"
    @merge-schools="mergeSchools"
  />
</template>
