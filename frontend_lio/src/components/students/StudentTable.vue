<script setup lang="ts">
import type { TableColumn, TableRow } from "@nuxt/ui"

import type { Student } from "@/types/student.ts"

import StudentAwardsBlock from "@/components/students/StudentAwardsBlock.vue"

defineProps<{ students: Student[]; loading: boolean }>()
const emit = defineEmits<{ select: [Student] }>()

const columns: TableColumn<Student>[] = [
  {
    accessorKey: "name",
    header: "Name",
  },
  {
    accessorKey: "awards",
    header: "Awards",
  },
]

function onSelect(_: Event, row: TableRow<Student>) {
  emit("select", row.original)
}
</script>

<template>
  <UTable
    :data="students"
    :columns="columns"
    class="flex-1"
    :loading="loading"
    :ui="{ base: 'table-fixed w-full', td: 'overflow-hidden text-ellipsis' }"
    @select="onSelect"
  >
    <template #name-cell="{ row }">
      {{ row.original.given_name }} {{ row.original.family_name }}
    </template>
    <template #awards-cell="{ row }">
      <StudentAwardsBlock :student="row.original" />
    </template>
  </UTable>
</template>
