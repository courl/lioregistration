<script setup lang="ts">
import { ref } from "vue"

import type { ActiveEdition } from "@/types/edition.ts"

const mode = ref<"banner" | "register" | "registered">("banner")
const activeEdition = ref<ActiveEdition>()
const data = ref({ name: "", email: "" })
function onRegister(edition: ActiveEdition) {
  activeEdition.value = edition
  mode.value = "register"
}
function onRegistered({
  given_name,
  email,
}: {
  given_name: string
  email: string
}) {
  mode.value = "registered"
  data.value = { name: given_name, email }
}
</script>

<template>
  <div
    class="min-h-screen flex flex-col items-center justify-center bg-default"
  >
    <RegistrationBanner v-if="mode === 'banner'" @register="onRegister" />
    <RegistrationForm
      v-if="mode === 'register' && activeEdition"
      :edition="activeEdition"
      @registered="onRegistered"
    />
    <RegistrationFinal
      v-if="mode === 'registered' && activeEdition"
      :edition="activeEdition"
      :name="data.name"
      :email="data.email"
    />
  </div>
</template>

<route>
{
  meta: {
    layout: 'plain'
  }
}
</route>
