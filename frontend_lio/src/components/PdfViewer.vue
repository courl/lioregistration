<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue"

const props = withDefaults(
  defineProps<{
    src: string
    height?: string
  }>(),
  { height: "100vh" },
)

const objectUrl = ref<string | null>(null)
const error = ref<string | null>(null)

function base64ToBlob(input: string, mime = "application/pdf"): Blob {
  const b64 = input.replace(/^data:[^;]+;base64,/, "").replaceAll(/\s/g, "")
  const bytes = Uint8Array.fromBase64(b64)
  return new Blob([bytes], { type: mime })
}

function revoke() {
  if (objectUrl.value !== null) URL.revokeObjectURL(objectUrl.value)
  objectUrl.value = null
}

watch(
  () => props.src,
  (source) => {
    revoke()
    error.value = null
    if (!source) return
    try {
      objectUrl.value = URL.createObjectURL(base64ToBlob(source))
    } catch (error_) {
      error.value = "Invalid PDF data"
      console.error(error_)
    }
  },
  { immediate: true },
)

onBeforeUnmount(revoke)

const viewerSource = computed(() =>
  objectUrl.value === null
    ? null
    : `${objectUrl.value}#toolbar=1&navpanes=0&view=FitH`,
)
</script>

<template>
  <UError
    v-if="error"
    :error="{ statusMessage: 'Cannot load preview', message: error }"
  />
  <template v-else-if="viewerSource">
    <iframe
      :src="viewerSource"
      :style="{ width: '100%', height, border: 0 }"
      title="PDF document"
    />
  </template>
</template>
