<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { onMounted, reactive, useTemplateRef, watch } from "vue"
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
  password: z.string("Password is required"),
})

export type Schema = z.output<typeof schema>

const state = reactive<Partial<Schema>>({})
const form = useTemplateRef("form")

async function onSubmit(event: FormSubmitEvent<Schema>) {
  try {
    await auth.login(event.data.email, event.data.password)
  } catch (error) {
    console.error(error)
    toast.add({
      title: "Login failed",
      description: "Invalid credentials",
      icon: "i-lucide-triangle-alert",
    })
    state.password = undefined
  }
}
</script>

<template>
  <div class="flex justify-center items-center w-full h-screen">
    <UCard title="Login" :ui="{ footer: 'justify-end flex' }">
      <UForm
        ref="form"
        :schema="schema"
        :state="state"
        class="space-y-4"
        @submit="onSubmit"
      >
        <UFormField label="Email" name="email">
          <UInput v-model="state.email" />
        </UFormField>
        <UFormField label="Password" name="password">
          <UInput v-model="state.password" type="password" />
        </UFormField>
      </UForm>
      <template #footer>
        <UButton
          label="Login"
          color="primary"
          variant="subtle"
          @click="form?.submit()"
        />
      </template>
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
