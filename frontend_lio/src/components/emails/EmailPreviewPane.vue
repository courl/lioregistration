<script setup lang="ts">
import { computed, ref, watch } from "vue"

import type { EmailPreview } from "@/types/email.ts"

const props = defineProps<{
  preview: EmailPreview
}>()

const tab = ref<string>("preview")

const attachment = computed(() =>
  tab.value.startsWith("attachment-")
    ? props.preview.attachments.at(Number(tab.value.slice(11)))
    : undefined,
)

// A new preview may have fewer attachments: fall back to the email itself.
watch(
  () => props.preview,
  () => {
    if (tab.value !== "preview" && attachment.value === undefined) {
      tab.value = "preview"
    }
  },
)
</script>

<template>
  <div>
    <UTabs
      v-model="tab"
      :items="[
        { icon: 'i-lucide-eye', label: 'Preview', value: 'preview' },
        ...preview.attachments.map((x, idx) => ({
          icon: 'i-lucide-paperclip',
          label: x.name,
          value: `attachment-${idx}`,
        })),
      ]"
      variant="link"
    />
    <div class="rounded border border-muted h-full">
      <template v-if="tab === 'preview'">
        <div
          v-if="preview.subject"
          class="border-b border-muted px-3 py-2 text-sm"
        >
          <span class="text-muted">Subject:</span>
          <span class="font-medium">{{ preview.subject }}</span>
        </div>
        <!-- eslint-disable-next-line vue/no-v-html -->
        <div v-if="preview.html" v-html="preview.html" />
        <UError
          v-if="preview.error"
          :error="{
            statusMessage: 'Cannot load preview',
            message: preview.error,
          }"
          :clear="false"
        />
      </template>
      <template v-else-if="attachment">
        <PdfViewer
          v-if="attachment.data_base64"
          :src="attachment.data_base64"
        />
        <UError
          v-if="attachment.error"
          :error="{
            statusMessage: 'Cannot load preview',
            message: attachment.error,
          }"
          :clear="false"
        />
      </template>
    </div>
  </div>
</template>
