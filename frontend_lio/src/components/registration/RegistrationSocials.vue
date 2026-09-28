<script setup lang="ts">
const socials = [
  {
    label: "Website",
    icon: "i-lucide-globe",
    href: "https://portal.education.lu/infosolympiad/",
  },
  {
    label: "Instagram",
    icon: "i-lucide-instagram",
    href: "https://instagram.com/luxembourginformaticsolympiad/",
  },
  {
    label: "LinkedIn",
    icon: "i-lucide-linkedin",
    href: "https://linkedin.com/company/luxembourg-informatics-olympiad",
  },
]
</script>

<template>
  <div class="space-y-3">
    <p class="text-xl font-medium text-highlighted">Stay in the loop</p>
    <p class="text-sm text-muted">
      Follow along for updates, results, and behind-the-scenes.
    </p>
    <div class="flex justify-center gap-3">
      <UButton
        v-for="social in socials"
        :key="social.label"
        :icon="social.icon"
        :to="social.href"
        target="_blank"
        rel="noopener"
        color="neutral"
        variant="outline"
        size="lg"
        :aria-label="social.label"
      />
    </div>
  </div>
</template>
