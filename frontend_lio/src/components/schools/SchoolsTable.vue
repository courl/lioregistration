<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui"

import { computed, h, ref, resolveComponent } from "vue"

import type { School, SchoolId } from "@/types/school.ts"

import { useAuth } from "@/composables/api.ts"

defineProps<{ schools: School[] }>()
const emit = defineEmits<{
  mergeSchools: [source: SchoolId, target: SchoolId]
  select: [school: School]
}>()
const auth = useAuth()
const UButton = resolveComponent("UButton")
const mergeModalOpen = ref(false)
const mergeSource = ref<School | null>(null)
const mergeTarget = ref<SchoolId | "">("")

const canMerge = computed(
  () =>
    typeof mergeTarget.value === "number" &&
    Number.isInteger(mergeTarget.value) &&
    mergeTarget.value > 0 &&
    mergeTarget.value !== mergeSource.value?.id,
)

function confirmMerge() {
  if (!mergeSource.value || typeof mergeTarget.value !== "number") return
  emit("mergeSchools", mergeSource.value.id, mergeTarget.value)
  mergeModalOpen.value = false
}

function startMerge(school: School) {
  mergeSource.value = school
  mergeTarget.value = ""
  mergeModalOpen.value = true
}

const columns = computed<TableColumn<School>[]>(() => [
  {
    accessorKey: "id",
    header: "ID",
  },
  {
    accessorKey: "name",
    header: "Name",
  },
  ...(auth.isAdmin
    ? [
        {
          id: "actions",
          cell: ({ row }) =>
            h("div", { class: "text-right" }, [
              h(UButton, {
                label: "Merge with",
                icon: "i-lucide-merge",
                variant: "ghost",
                size: "sm",
                onClick: (event: Event) => {
                  // Don't also open the school
                  event.stopPropagation()
                  startMerge(row.original)
                },
              }),
            ]),
        } satisfies TableColumn<School>,
      ]
    : []),
])
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
    description="Enter the id of the school to merge into this one."
  >
    <template #body>
      <UInput
        v-model.number="mergeTarget"
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
          :disabled="!canMerge"
          @click="confirmMerge"
        />
      </UFieldGroup>
    </template>
  </UModal>
</template>
