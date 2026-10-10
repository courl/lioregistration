<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui"

import { computed, ref } from "vue"

import type { Credentials } from "@/types/credentials.ts"
import type { Edition } from "@/types/edition.ts"

import { useAuth } from "@/composables/api.ts"
import { useConfirmDialog } from "@/composables/use-confirm-dialog.ts"
import { useCredentialsModal } from "@/composables/use-credentials-modal.ts"
import { useCredentials } from "@/composables/use-credentials.ts"

const props = defineProps<{ edition: Edition }>()

const auth = useAuth()
const confirm = useConfirmDialog()
const credentialsDialog = useCredentialsModal()
const {
  credentials,
  loading,
  createCredentials,
  unassignCredentials,
  deleteCredentials,
} = useCredentials(() => props.edition.id)

const filter = ref<"all" | "assigned" | "free">("all")
const filteredCredentials = computed(() => {
  switch (filter.value) {
    case "all": {
      return credentials.value
    }
    case "assigned": {
      return credentials.value.filter((x) => x.registration !== null)
    }
    case "free": {
      return credentials.value.filter((x) => x.registration === null)
    }
  }
})

async function addCredentials() {
  const result = await credentialsDialog({})
  if (result) {
    await createCredentials(result)
  }
}

async function processUnassign(value: Credentials) {
  if (
    await confirm({
      title: "Unassign credentials",
      description: `Unassign '${value.username}' from ${value.registration?.given_name ?? ""} ${value.registration?.family_name ?? ""}?`,
    })
  ) {
    await unassignCredentials(value.id)
  }
}

async function processDelete(value: Credentials) {
  if (
    await confirm({
      title: "Delete credentials",
      description: `Are you sure you want to delete '${value.username}'?`,
    })
  ) {
    await deleteCredentials(value.id)
  }
}

const counts = computed(() => {
  const result = { free: 0, assigned: 0 }
  for (const value of credentials.value) {
    if (value.registration) {
      result.assigned += 1
    } else {
      result.free += 1
    }
  }
  return result
})

const columns: TableColumn<Credentials>[] = [
  { id: "username", header: "Username" },
  { id: "password", header: "Password" },
  { id: "registrant", header: "Assigned to" },
  ...(auth.isAdmin ? [{ id: "actions", header: "" }] : []),
]
</script>

<template>
  <UCard :ui="{ body: 'p-0 sm:p-0' }">
    <template #header>
      <div class="flex items-center justify-between">
        <div>
          <span class="font-bold text-lg">Credentials</span>
          <IconTextBlock
            v-model="filter"
            class="mt-1"
            :items="[
              {
                icon: 'i-lucide-key-round',
                text: `${credentials.length} total`,
                value: 'all',
              },
              {
                icon: 'i-lucide-user-check',
                text: `${counts['assigned']} assigned`,
                value: 'assigned',
              },
              {
                icon: 'i-lucide-user-plus',
                text: `${counts['free']} free`,
                value: 'free',
              },
            ]"
          />
        </div>
        <UButton
          v-if="auth.isAdmin"
          icon="i-lucide-plus"
          variant="subtle"
          @click="addCredentials"
          >Add Credentials</UButton
        >
      </div>
    </template>

    <UTable
      :data="filteredCredentials"
      :columns="columns"
      :loading="loading"
      class="flex-1"
    >
      <template #username-cell="{ row }">
        <span class="font-mono">{{ row.original.username }}</span>
      </template>

      <template #password-cell="{ row }">
        <span class="font-mono">
          {{ row.original.password }}
        </span>
      </template>

      <template #registrant-cell="{ row }">
        <RouterLink
          v-if="row.original.registration"
          :to="`/admin/registrations/${row.original.registration.id}`"
          class="hover:text-primary"
        >
          {{ row.original.registration.given_name }}
          {{ row.original.registration.family_name }}
        </RouterLink>
        <UBadge v-else variant="subtle" color="neutral">Free</UBadge>
      </template>

      <template #actions-cell="{ row }">
        <div class="flex items-center justify-end gap-1">
          <UButton
            v-if="row.original.registration"
            icon="i-lucide-user-round-minus"
            color="primary"
            variant="ghost"
            size="xs"
            title="Unassign"
            aria-label="Unassign"
            @click="async () => await processUnassign(row.original)"
          />
          <UButton
            v-else
            icon="i-lucide-trash"
            color="error"
            variant="ghost"
            size="xs"
            title="Delete"
            aria-label="Delete"
            @click="async () => await processDelete(row.original)"
          />
        </div>
      </template>
    </UTable>
    <UEmpty
      v-if="!loading && credentials.length === 0"
      icon="i-lucide-key-round"
      title="No credentials"
      description="This edition has no credentials yet. Generate some to get started."
    />
  </UCard>
</template>
