<script setup lang="ts">
import type { ActiveEdition } from "@/types/edition.ts"

defineProps<{
  edition: ActiveEdition
  name: string
  email: string
}>()
</script>

<template>
  <div class="flex max-w-xl flex-col items-center gap-6 py-16 text-center">
    <div class="space-y-2">
      <h1 class="text-2xl font-bold text-primary">
        You're registered, {{ name }}!
      </h1>
      <p class="text-muted">
        Thanks for signing up for
        <span class="font-medium text-default">{{ edition.name }}</span
        >. We've sent a confirmation with instructions to
        <span class="font-medium text-default">{{ email }}</span
        >.
      </p>
    </div>

    <UAlert
      icon="i-lucide-mail-check"
      color="neutral"
      variant="subtle"
      title="Check your inbox"
      description="Didn't get anything within a few minutes? Check spam, or reach out to us so we can help."
      class="w-full text-left"
    />
    <USeparator />
    <RegistrationSocials />
  </div>
</template>
