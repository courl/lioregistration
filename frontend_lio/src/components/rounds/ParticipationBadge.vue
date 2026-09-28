<script setup lang="ts">
import { computed } from "vue"

import type { ParticipationState } from "@/types/round.ts"

const props = defineProps<{ state: ParticipationState }>()

const color = computed(
  () =>
    ({
      accepted: "success" as const,
      invited: "primary" as const,
      declined: "warning" as const,
      noShow: "error" as const,
    })[props.state],
)
</script>

<template>
  <UBadge variant="soft" :color="color" class="capitalize">{{
    props.state
  }}</UBadge>
</template>
