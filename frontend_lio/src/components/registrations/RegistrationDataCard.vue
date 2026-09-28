<script setup lang="ts">
import { computed } from "vue"

import type { RegistrationDetails } from "@/types/registration.js"

const props = defineProps<{
  registration: RegistrationDetails
  credentialsAvailable?: boolean
}>()
const emit = defineEmits<{
  assignCredentials: []
  unassignCredentials: []
}>()

const genderIcon = computed(() => {
  switch (props.registration.gender) {
    case "female": {
      return "i-lucide-venus"
    }
    case "male": {
      return "i-lucide-mars"
    }
    case "other": {
      return "i-lucide-venus-and-mars"
    }
  }
})
</script>

<template>
  <UCard>
    <template #header>
      <span class="font-bold">Registration Data</span>
    </template>

    <div class="space-y-5 text-sm">
      <section class="space-y-2">
        <h3 class="text-xs font-semibold uppercase tracking-wide text-dimmed">
          Personal
        </h3>
        <div class="flex items-center gap-2 capitalize">
          <UIcon :name="genderIcon" class="size-4 shrink-0 text-dimmed" />
          {{ registration.gender }}
        </div>
        <div class="flex items-center gap-2">
          <UIcon name="i-lucide-cake" class="size-4 shrink-0 text-dimmed" />
          {{ registration.birthdate }}
        </div>
      </section>

      <section class="space-y-2">
        <h3 class="text-xs font-semibold uppercase tracking-wide text-dimmed">
          Contact
        </h3>
        <div class="flex items-center gap-2">
          <UIcon name="i-lucide-mail" class="size-4 shrink-0 text-dimmed" />
          {{ registration.email }}
        </div>
        <div class="flex items-center gap-2">
          <UIcon name="i-lucide-phone" class="size-4 shrink-0 text-dimmed" />
          <span>{{ registration.phone }}</span>
        </div>
        <div class="flex items-start gap-2">
          <UIcon
            name="i-lucide-map-pin"
            class="mt-0.5 size-4 shrink-0 text-dimmed"
          />
          <div>
            <div>{{ registration.address_street }}</div>
            <div>
              {{ registration.address_postal_code }}
              {{ registration.address_locality }}
            </div>
          </div>
        </div>
      </section>

      <section class="space-y-2">
        <h3 class="text-xs font-semibold uppercase tracking-wide text-dimmed">
          Background
        </h3>
        <div class="flex items-center gap-2">
          <UIcon name="i-lucide-school" class="size-4 shrink-0 text-dimmed" />
          {{ registration.school }}
          <span class="text-muted">({{ registration.grade }})</span>
        </div>
        <div class="flex items-start gap-2">
          <UIcon
            name="i-lucide-code"
            class="mt-0.5 size-4 shrink-0 text-dimmed"
          />
          {{ registration.programming_languages.join(", ") }}
        </div>
        <div class="flex items-center gap-2">
          <UIcon
            name="i-lucide-megaphone"
            class="size-4 shrink-0 text-dimmed"
          />
          {{ registration.first_contact_source }}
        </div>
        <div class="flex items-center gap-2 -mt-2">
          <div
              class="size-4 shrink-0"
          />
        <span class="text-muted">{{ registration.first_contact_other ?? ''}}</span>
        </div>
      </section>

      <section class="space-y-2">
        <h3 class="text-xs font-semibold uppercase tracking-wide text-dimmed">
          Credentials
        </h3>
        <template v-if="registration.credentials">
          <div class="flex justify-between items-center">
            <div>
              <div class="flex items-center gap-2">
                <UIcon
                  name="i-lucide-user"
                  class="size-4 shrink-0 text-dimmed"
                />
                <span class="font-mono">{{
                  registration.credentials?.username
                }}</span>
              </div>
              <div class="flex items-center gap-2">
                <UIcon
                  name="i-lucide-key-round"
                  class="size-4 shrink-0 text-dimmed"
                />
                <span class="font-mono">
                  {{ registration.credentials?.password }}</span
                >
              </div>
            </div>
            <UButton
              size="xs"
              icon="i-lucide-user-key"
              variant="ghost"
              color="error"
              @click="emit('unassignCredentials')"
              >Unassign</UButton
            >
          </div>
        </template>
        <template v-else>
          <UAlert
            v-if="credentialsAvailable"
            variant="outline"
            icon="i-lucide-triangle-alert"
            title="No assigned credentials"
            description="There are no credentials that have been assigned to this user. This can happen if the pool has been depleated."
            orientation="horizontal"
            :actions="[
              {
                icon: 'i-lucide-user-key',
                label: 'Assign',
                onClick: () => {
                  emit('assignCredentials')
                },
              },
            ]"
          />
          <UAlert
            v-else
            variant="subtle"
            icon="i-lucide-triangle-alert"
            title="No free credentials"
            description="The credentials pool for this edition is empty."
          />
        </template>
      </section>
    </div>
  </UCard>
</template>
