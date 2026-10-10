<script setup lang="ts">
import { computed, ref } from "vue"
import { AreaChart, CurveType, LegendPosition } from "vue-chrts"

import type { Edition } from "@/types/edition.ts"

import { useCredentials } from "@/composables/use-credentials.ts"
import { useEditionStatistics } from "@/composables/use-edition-statistics.ts"
import CalendarDate from "@/types/calendar-date.ts"

const props = defineProps<{
  edition: Edition
}>()

const { statistics } = useEditionStatistics(() => props.edition.id)
const { credentials } = useCredentials(() => props.edition.id)

const freeCredentials = computed(
  () => credentials.value.filter((x) => x.registration === null).length,
)

const accumulated = ref<boolean>(false)

const data = computed(() => {
  if (!statistics.value) return []
  const range = CalendarDate.minmax([
    props.edition.registration_opens_at,
    CalendarDate.minmax([
      props.edition.registration_closes_at,
      new CalendarDate({}),
    ])[0],
    ...statistics.value.registrations.map((x) => x.day),
  ])
  const data = CalendarDate.generate(...range).map((x) => ({
    date: x,
    registrations: 0,
    reviewed: 0,
  }))

  let accumulated_registrations = 0
  let accumulated_reviewed = 0
  const lookupMap: Record<
    string,
    | {
        day: CalendarDate
        new: number
        reviewed: number
      }
    | undefined
  > = Object.fromEntries(
    statistics.value.registrations.map((x) => [x.day.toString(), x]),
  )
  for (const datum of data) {
    const stats = lookupMap[datum.date.toString()]
    if (stats) {
      accumulated_registrations += stats.new
      accumulated_reviewed += stats.reviewed
    }
    if (accumulated.value) {
      datum.registrations = accumulated_registrations
      datum.reviewed = accumulated_reviewed
    } else if (stats) {
      datum.registrations = stats.new
      datum.reviewed = stats.reviewed
    }
  }
  return data
})

const remainingReviews = computed(() => {
  let remaining = 0
  for (const x of statistics.value?.registrations ?? []) {
    remaining += x.new - x.reviewed
  }
  return remaining
})

const xFormatter = (tick: number): string => {
  return data.value[tick].date.toString()
}

const categories = {
  registrations: {
    name: "Registrations",
    color: "var(--ui-primary)",
  },
  reviewed: {
    name: "Reviewed",
    color: "var(--ui-success)",
  },
}
</script>

<template>
  <div v-if="statistics === undefined" class="flex justify-center p-8">
    <UIcon name="i-lucide-loader-circle" class="size-6 animate-spin" />
  </div>
  <template v-else>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <RouterCard :to="{ query: { tab: 'registrations', state: 'pending' } }">
        <div class="text-3xl font-bold text-primary">
          {{ remainingReviews }}
        </div>
        <div class="text-muted text-sm mt-1">Pending reviews</div>
      </RouterCard>
      <RouterCard :to="{ query: { tab: 'credentials' } }">
        <div
          class="text-3xl font-bold"
          :class="freeCredentials === 0 ? 'text-error' : ''"
        >
          {{ freeCredentials }}
        </div>
        <div class="text-muted text-sm mt-1">Remaining Credentials</div>
      </RouterCard>
      <RouterCard :to="{ query: { tab: 'participants' } }">
        <div class="text-3xl font-bold">{{ statistics.students }}</div>
        <div class="text-muted text-sm mt-1">Participants</div>
      </RouterCard>
      <RouterCard :to="{ query: { tab: 'rounds' } }">
        <div class="text-3xl font-bold">{{ statistics.rounds }}</div>
        <div class="text-muted text-sm mt-1">Rounds</div>
      </RouterCard>
      <UCard class="col-span-full">
        <div class="flex justify-end">
          <UButton variant="link" @click.prevent="accumulated = !accumulated">{{
            accumulated ? "accumulated" : "daily"
          }}</UButton>
        </div>
        <AreaChart
          :data="data"
          :height="200"
          :categories="categories"
          :x-formatter="xFormatter"
          :curve-type="CurveType.MonotoneX"
          :legend-position="LegendPosition.TopCenter"
        />
      </UCard>
    </div>
  </template>
</template>
