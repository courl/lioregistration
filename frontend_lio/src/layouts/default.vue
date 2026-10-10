<script setup lang="ts">
import type { DropdownMenuItem, NavigationMenuItem } from "@nuxt/ui"

import { computed } from "vue"
import { useRoute } from "vue-router"

import { useAuth } from "@/composables/api.ts"

const route = useRoute()
const auth = useAuth()

const userItems = computed<DropdownMenuItem[][]>(() => [
  [
    {
      label: auth.user?.email ?? "",
      icon: "i-lucide-user",
      disabled: true,
    },
  ],
  [
    {
      label: "Logout",
      icon: "i-lucide-log-out",
      onSelect: () => {
        void auth.logout()
      },
    },
  ],
])

const items = computed(() => {
  const items: NavigationMenuItem[] = [
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
  ]
  if (auth.isAdmin) {
    items.push({
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
    })
  }
  return items
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
      <UDropdownMenu :items="userItems">
        <UButton
          color="neutral"
          variant="ghost"
          icon="i-lucide-circle-user"
          :label="auth.user?.name"
          :aria-label="auth.user?.name ?? 'User menu'"
        >
          <template #trailing>
            <UBadge
              variant="subtle"
              size="sm"
              :color="auth.isAdmin ? 'primary' : 'neutral'"
              >{{ auth.user?.role }}</UBadge
            >
          </template>
        </UButton>
      </UDropdownMenu>
    </template>
  </UHeader>
  <UMain>
    <UContainer class="mt-6 mb-4">
      <RouterView />
    </UContainer>
  </UMain>
</template>
