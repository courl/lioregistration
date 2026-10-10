<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { FetchError } from "ofetch"
import { onMounted, reactive, watch } from "vue"
import { useRouter } from "vue-router"
import * as z from "zod"

import { useAuth } from "@/composables/api.ts"

const auth = useAuth()

const router = useRouter()
const toast = useToast()

onMounted(async () => {
  await auth.fetchMe()
})

watch(
  () => auth.isAuthenticated,
  async (isAuthenticated) => {
    if (isAuthenticated) {
      await router.push("/admin")
    }
  },
  { immediate: true },
)

const schema = z.object({
  email: z.email("Email is required"),
  password: z.string("Password is required").min(1, "Password is required"),
})

export type Schema = z.output<typeof schema>

const state = reactive<Partial<Schema>>({})

async function onSubmit(event: FormSubmitEvent<Schema>) {
  try {
    await auth.login(event.data.email, event.data.password)
  } catch (error) {
    // Other failures (rate limit, server errors) are toasted by the API client
    if (error instanceof FetchError && error.status === 401) {
      toast.add({
        title: "Login failed",
        description: "Invalid credentials",
        icon: "i-lucide-triangle-alert",
        color: "error",
      })
    }
    state.password = undefined
  }
}
</script>

<template>
  <div class="flex justify-center items-center w-full h-screen">
    <UCard title="Login" class="w-full max-w-sm">
      <UForm
        :schema="schema"
        :state="state"
        class="space-y-4"
        @submit="onSubmit"
      >
        <UFormField label="Email" name="email">
          <UInput
            v-model="state.email"
            type="email"
            autocomplete="username"
            class="w-full"
          />
        </UFormField>
        <UFormField label="Password" name="password">
          <UInput
            v-model="state.password"
            type="password"
            autocomplete="current-password"
            class="w-full"
          />
        </UFormField>
        <div class="flex justify-end">
          <UButton
            type="submit"
            label="Login"
            color="primary"
            variant="subtle"
            :loading="auth.pending"
          />
        </div>
      </UForm>
    </UCard>
  </div>
</template>

<route>
{
  meta: {
    layout: 'plain'
  }
}
</route>
