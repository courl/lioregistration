<script setup lang="ts">
import { computed } from "vue"
import { useRoute, useRouter } from "vue-router"

import type { Edition } from "@/types/edition.ts"

defineProps<{ edition: Edition }>()

const router = useRouter()
const route = useRoute()
const toast = useToast()

const TABS = [
  "stats",
  "registrations",
  "participants",
  "rounds",
  "credentials",
] as const

const state = computed<(typeof TABS)[number]>({
  get: () => {
    const value = route.query.tab
    return TABS.includes(value as (typeof TABS)[number])
      ? (value as (typeof TABS)[number])
      : "stats"
  },
  set: (value) => {
    router
      .replace({
        query: { ...route.query, tab: value === "stats" ? undefined : value },
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
</script>

<template>
  <PageHeader :title="edition.name">
    <template #subtitle>
      <IconTextBlock
        :items="[
          {
            icon: 'i-lucide-clipboard',
            text: `${edition.registration_opens_at} – ${edition.registration_closes_at}`,
          },
        ]"
      />
    </template>
    <template #header>
      <UBadge v-if="edition.active" variant="subtle" color="success"
        >active</UBadge
      >
    </template>
  </PageHeader>

  <UTabs
    v-model="state"
    :items="[
      { icon: 'i-lucide-activity', label: 'Statistics', value: 'stats' },
      {
        icon: 'i-lucide-clipboard',
        label: 'Registrations',
        value: 'registrations',
      },
      { icon: 'i-lucide-user', label: 'Participants', value: 'participants' },
      { icon: 'i-lucide-layers', label: 'Rounds', value: 'rounds' },
      { icon: 'i-lucide-key', label: 'Credentials', value: 'credentials' },
    ]"
  />

  <template v-if="state === 'stats'">
    <EditionStatisticsTab :edition="edition" />
  </template>
  <template v-else-if="state === 'registrations'">
    <EditionRegistrationsTab :edition="edition" />
  </template>
  <template v-else-if="state === 'rounds'">
    <EditionRoundsTab :edition="edition" />
  </template>
  <template v-else-if="state === 'credentials'">
    <EditionCredentialsTab :edition="edition" />
  </template>
  <template v-else-if="state === 'participants'">
    <EditionStudentsTab :edition="edition" />
  </template>
  <template v-else>
    <UEmpty
      icon="i-lucide-triangle-alert"
      title="No implemented"
      description="It looks like you are accessing a page that has not yet been implemented."
    />
  </template>
</template>
