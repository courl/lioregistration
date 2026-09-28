<script setup lang="ts">
import { computed } from "vue"

import type { ActiveEdition } from "@/types/edition.ts"

import { useActiveEdition } from "@/composables/use-register.ts"

const emit = defineEmits<{
  register: [edition: ActiveEdition]
}>()

const { edition } = useActiveEdition()
const registrationState = computed(() => {
  if (!edition.value) {
    return "closed"
  }
  const today = new Date()
  if (today < edition.value.registration_opens_at.getDate()) {
    return "tooEarly"
  }

  if (today > edition.value.registration_closes_at.getDate()) {
    return "tooLate"
  }
  return "open"
})
</script>

<template>
  <div class="flex max-w-xl flex-col items-center gap-6 py-16 text-center">
    <h1 class="text-4xl font-bold">
      {{ edition ? edition.name : "LIO Registration" }}
    </h1>
    <template v-if="registrationState === 'open' && edition">
      <p class="text-muted text-lg">
        Registration is open until {{ edition.registration_closes_at }}
      </p>
      <UButton size="xl" @click="emit('register', edition)">
        Register now
      </UButton>
    </template>
    <template v-else-if="edition">
      <p class="text-muted text-lg">
        <template v-if="registrationState === 'tooEarly'">
          Registration opens on {{ edition.registration_opens_at }}
        </template>
        <template v-else>
          Registration closed on
          {{ edition.registration_closes_at }}
        </template>
      </p>
    </template>
    <template v-else>
      <p class="text-muted text-lg">No active edition. Check back later.</p>
    </template>
    <USeparator />
    <RegistrationSocials />
  </div>
</template>
