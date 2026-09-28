<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui"

import { h, ref, resolveComponent } from "vue"

import type { School, SchoolId } from "@/types/school.ts"

defineProps<{ schools: School[] }>()
const emit = defineEmits<{
  mergeSchools: [source: SchoolId, target: SchoolId]
  select: [school: School]
}>()
const UButton = resolveComponent("UButton")
const mergeModalOpen = ref(false)
const mergeSource = ref<School | null>(null)
const mergeTarget = ref<SchoolId | undefined>(undefined)
const merging = ref(false)

function confirmMerge() {
  if (!mergeSource.value || !mergeTarget.value) return
  merging.value = true
  try {
    emit("mergeSchools", mergeSource.value.id, mergeTarget.value)
    mergeModalOpen.value = false
  } finally {
    merging.value = false
  }
}

function startMerge(school: School) {
  mergeSource.value = school
  mergeTarget.value = undefined
  mergeModalOpen.value = true
}

const columns: TableColumn<School>[] = [
  {
    accessorKey: "id",
    header: "ID",
  },
  {
    accessorKey: "name",
    header: "Name",
  },
  {
    id: "actions",
    cell: ({ row }) =>
      h("div", { class: "text-right" }, [
        h(UButton, {
          label: "Merge with",
          icon: "i-lucide-merge",
          variant: "ghost",
          size: "sm",
          onClick: () => {
            startMerge(row.original)
          },
        }),
      ]),
  },
]
</script>

<template>
  <UTable
    :data="schools"
    :columns="columns"
    class="flex-1"
    @select="(_, row) => emit('select', row.original)"
  />

  <UModal
    v-model:open="mergeModalOpen"
    :title="`Merge with ${mergeSource?.name}`"
    description="Select another school to merge into this one."
  >
    <template #body>
      <UInput
        v-model="mergeTarget"
        type="number"
        min="0"
        placeholder="Enter id of school..."
        class="w-full"
      />
    </template>

    <template #footer>
      <UFieldGroup orientation="horizontal">
        <UButton
          label="Cancel"
          color="neutral"
          variant="outline"
          @click="mergeModalOpen = false"
        />
        <UButton
          label="Merge"
          icon="i-lucide-merge"
          variant="subtle"
          :loading="merging"
          :disabled="!mergeTarget || mergeTarget === mergeSource?.id"
          @click="confirmMerge"
        />
      </UFieldGroup>
    </template>
  </UModal>
</template>
