<script setup lang="ts">
import type { NavigationMenuItem } from "@nuxt/ui"

import { computed } from "vue"
import { useRoute } from "vue-router"

const route = useRoute()
const items = computed(() => {
  return [
    {
      label: "Dashboard",
      icon: "i-lucide-house",
      to: "/admin",
      active: route.path === "/admin/" || route.path === "/admin",
    },
    {
      label: "Editions",
      icon: "i-lucide-book-open",
      to: "/admin/editions",
      active: route.path.startsWith("/admin/editions"),
    },
    {
      label: "Schools",
      icon: "i-lucide-school",
      to: "/admin/schools",
      active: route.path.startsWith("/admin/schools"),
    },
    {
      label: "Students",
      icon: "i-lucide-graduation-cap",
      to: "/admin/students",
      active: route.path.startsWith("/admin/students"),
    },
    {
      label: "Settings",
      icon: "i-lucide-settings",
      active: route.path.startsWith("/admin/settings"),
      children: [
        {
          label: "Emails",
          icon: "i-lucide-mail",
          to: "/admin/settings/emails",
          active: route.path.startsWith("/admin/settings/emails"),
        },
      ],
    },
  ] as NavigationMenuItem[]
})
</script>

<template>
  <UHeader to="/admin/" mode="slideover">
    <template #title>
      <span class="text-primary font-extrabold">LIO</span> Admin
    </template>
    <UNavigationMenu :items="items" />
    <template #body>
      <UNavigationMenu :items="items" orientation="vertical" />
    </template>
    <template #right>
      <UColorModeButton />
    </template>
  </UHeader>
  <UMain>
    <UContainer class="mt-6 mb-4">
      <RouterView />
    </UContainer>
  </UMain>
</template>
