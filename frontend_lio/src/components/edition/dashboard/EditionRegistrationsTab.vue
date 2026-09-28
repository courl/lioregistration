<script setup lang="ts">
import { computed } from "vue"
import { useRoute, useRouter } from "vue-router"

import type { Edition } from "@/types/edition.ts"
import type { Registration, ReviewStatus } from "@/types/registration.ts"

import { useRegistrations } from "@/composables/use-registration.ts"

const props = defineProps<{ edition: Edition }>()
const router = useRouter()
const route = useRoute()
const toast = useToast()
const { registrations } = useRegistrations(() => props.edition.id)

const STATES = ["all", "pending", "validated", "refused"] as const
const ICONS = {
  all: "i-lucide-user",
  pending: "i-lucide-user",
  validated: "i-lucide-user-plus",
  refused: "i-lucide-user-x",
}

const state = computed<ReviewStatus | "all">({
  get: () => {
    const value = route.query.state
    return STATES.includes(value as ReviewStatus | "all")
      ? (value as ReviewStatus)
      : "all"
  },
  set: (value) => {
    router
      .replace({
        query: { ...route.query, state: value === "all" ? undefined : value },
      })
      .catch(() =>
        toast.add({
          title: "Uh oh! Something went wrong.",
          description: "Navigation failed",
          icon: "i-lucide-triangle-alert",
        }),
      )
  },
})

const statistics = computed(() => {
  const counts = {
    all: registrations.value.length,
    pending: 0,
    validated: 0,
    refused: 0,
  }
  for (const registration of registrations.value) {
    counts[registration.review_status] += 1
  }
  return counts
})

const filteredRegistrations = computed(() => {
  if (state.value === "all") {
    return registrations.value
  }
  return registrations.value.filter((x) => x.review_status === state.value)
})

const tabs = computed(() => {
  return STATES.map((state) => ({
    icon: ICONS[state],
    text: `${statistics.value[state].toString()} ${state === "all" ? "total" : state}`,
    value: state,
  }))
})

async function onSelect(registration: Registration) {
  await router.push(`/admin/registrations/${registration.id.toString()}`)
}
</script>

<template>
  <UCard :ui="{ body: 'p-0 sm:p-0' }">
    <template #header>
      <div class="flex items-center justify-between">
        <div>
          <span class="font-bold text-lg">Registrations</span>
          <IconTextBlock v-model="state" class="mt-1" :items="tabs" />
        </div>
      </div>
    </template>
    <RegistrationsTable
      :registrations="filteredRegistrations"
      @select="onSelect"
    />
  </UCard>
</template>
