<script setup lang="ts">

import type {TableColumn} from "@nuxt/ui";

import {useRouter} from "vue-router";

import type {EmailTemplate} from "@/types/email.ts";

import {useEmails} from "@/composables/use-email.ts";

const {emails, loading, createEmailTemplate} = useEmails()
const router = useRouter()

const columns: TableColumn<EmailTemplate>[] = [
  {
    accessorKey: "name",
    header: "Name",
  },
  {
    accessorKey: "template_type",
    header: "Type",
  }
]

function getColor(template_type: EmailTemplate["template_type"]): "success" | "neutral" | "primary" | "error" {
  switch (template_type) {
    case "error": {
      return "error"
    }
    case "registration": {
      return "success"
    }
    case "round": {
      return "primary"
    }
    case undefined: {
      return "neutral"
    }
  }
}
</script>

<template>
  <PageHeader title="Email Templates">
    <template #header>
      <UButton icon="i-lucide-plus" @click="createEmailTemplate">
        New Template
      </UButton>
    </template>
  </PageHeader>
  <UTable
      :data="emails"
      :columns="columns"
      :loading="loading"
      class="flex-1"
      @select="async (_, row) => router.push(`/admin/settings/emails/${row.original.id.toString()}`)"
  >
      <template #template_type-cell="{ row }">
        <UBadge variant="soft" :color="getColor(row.original.template_type)">{{row.original.template_type}}</UBadge>
      </template>
  </UTable>
</template>
