<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui"

import { h, resolveComponent } from "vue"

import type { Registration } from "@/types/registration.ts"

defineProps<{ registrations: Registration[] }>()
const emit = defineEmits<{ select: [Registration] }>()
const UBadge = resolveComponent("UBadge")
const columns: TableColumn<Registration>[] = [
  {
    accessorKey: "given_name",
    header: "Given Name",
  },
  {
    accessorKey: "family_name",
    header: "Family Name",
  },
  {
    accessorKey: "email",
    header: "Email",
  },
  {
    accessorKey: "school",
    header: "School",
  },
  {
    accessorKey: "grade",
    header: "Grade",
  },
  {
    accessorKey: "registration_date",
    header: "Registration Date",
  },
  {
    accessorKey: "review_status",
    header: "Status",
    cell: ({ row }) => {
      const color = {
        pending: "primary" as const,
        validated: "success" as const,
        refused: "error" as const,
      }[row.original.review_status]

      return h(
        UBadge,
        { variant: "subtle", color },
        () => row.original.review_status,
      )
    },
  },
]
</script>

<template>
  <UTable
    :data="registrations"
    :columns="columns"
    class="flex-1"
    @select="(_, row) => emit('select', row.original)"
  />
</template>
