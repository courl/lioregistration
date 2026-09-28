<script setup lang="ts">
import { CalendarDate as CDate } from "@internationalized/date";
import {reactive, ref, watch} from "vue";

import type {EmailPreview, EmailPreviewRequest, EmailTemplateId} from "@/types/email.ts";

import {useEmailTemplate} from "@/composables/use-email.ts";
import {useRouteId} from "@/composables/use-route-id.ts";
import CalendarDate from "@/types/calendar-date";


const id = useRouteId<EmailTemplateId>()
const { email, previewEmailTemplate, updateEmailTemplate } = useEmailTemplate(id)
const preview = ref<EmailPreview>({'error': 'Not computed yet'})
const tab = ref<string>('template')

const body = ref<string>("This is a **test**")
const global_context: EmailPreviewRequest["global_context"] = reactive({
  edition: "LIO 2026",
  headline: "LIO - NEWS",
  round_starts_at: new CalendarDate({year: 2025, month: 11, day: 25}),
  round_ends_at: new CalendarDate({year: 2026, month: 2, day: 15}),
  students: [["Alice Müller", 12], ["Peter Schmidt", 8], ["Thierry Hubert", 3.21]]
})

const instance_context: EmailPreviewRequest["instance_context"] = reactive({
  given_name: "Alice",
  family_name: "Becker",
  credentials_username: undefined,
  credentials_password: undefined,
  grade: "2B",
  school: "Lycée des Informaticians"
})

watch(email, async (newEmail) => {
  if(!newEmail) {
    preview.value = {'error': 'Not computed yet'}
    return
  }
  preview.value = await previewEmailTemplate({
    body: body.value,
    global_context,
    instance_context
  })
})

watch(() => email.value?.template_type, (newType) => {
  if (newType === 'round') {
    global_context.students = [["Alice Müller", 12], ["Peter Schmidt", 8], ["Thierry Hubert", 3.21]]
    instance_context.credentials_username = undefined
    instance_context.credentials_password = undefined
  } else {
    global_context.headline = "Registration"
    global_context.students = []
    instance_context.credentials_username = "lio1"
    // eslint-disable-next-line sonarjs/no-hardcoded-passwords
    instance_context.credentials_password = "password-1"
  }
})
</script>

<template>
  <PageHeader :title="email?.name || 'Unknown Template'">
    <template #header>
      <UButton icon="i-lucide-save" @click="async () => {if(email) await updateEmailTemplate(email)}">
        Save
      </UButton>
    </template>
  </PageHeader>
  <div v-if="email" class="grid grid-cols-1 gap-4 md:grid-cols-2">
    <div>
      <UTabs
      v-model="tab" :items="[{ icon: 'i-lucide-mail', label: 'Template', value: 'template' },
      { icon: 'i-lucide-sliders-horizontal', label: 'Preview', value: 'preview' }]"/>
      <template v-if="tab === 'template'">
        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 mb-2">
            <UFormField label="Template Name">
                <UInput v-model="email.name" class="w-full"/>
            </UFormField>
            <UFormField label="Template Type" class="w-full">
                <USelect v-model="email.template_type" class="w-full" :items="[['round', 'registration', 'error'], [{
                    label: 'Unset the template',
                    value: undefined
                  },]]"/>
            </UFormField>
        </div>
        <UFormField label="Template">
            <UTextarea v-model="email.template" class="w-full font-mono" autoresize/>
        </UFormField>
      </template>
      <template v-else-if="tab === 'preview'">
        <div class="space-y-4">
            <section class="space-y-2">
                <div>
                    <h2 class="text-base font-semibold text-highlighted">
                        Global Context
                    </h2>
                </div>
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                    <UFormField label="Edition">
                        <UInput v-model="global_context.edition" class="w-full"></UInput>
                    </UFormField>
                    <UFormField label="Headline" :hidden="email.template_type === 'registration'">
                        <UInput v-model="global_context.headline" class="w-full" ></UInput>
                    </UFormField>
                </div>
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                    <UFormField label="Round Start">
                        <UInputDate
                            :range="false"
                            class="w-full"
                            :model-value="new CDate(global_context.round_starts_at.year, global_context.round_starts_at.month, global_context.round_starts_at.day)"
                            @update:model-value="
                            (value) => {
                              global_context.round_starts_at = value ? new CalendarDate({date:value.toDate('UTC')}) : new CalendarDate({})
                            }
                            "
                        />
                    </UFormField>
                    <UFormField label="Round End">
                        <UInputDate
                            :range="false"
                            class="w-full"
                            :model-value="new CDate(global_context.round_ends_at.year, global_context.round_ends_at.month, global_context.round_ends_at.day)"
                            @update:model-value="
                            (value) => {
                              global_context.round_ends_at = value ? new CalendarDate({date:value.toDate('UTC')}) : new CalendarDate({})
                            }
                            "
                        />
                    </UFormField>
                </div>
                <UFormField label="Body" :hidden="email.template_type === 'registration'">
                    <UTextarea v-model="body" class="w-full" autoresize/>
                </UFormField>
            </section>

            <USeparator/>
            <section class="space-y-2">
                <div>
                    <h2 class="text-base font-semibold text-highlighted">
                        Instance
                    </h2>
                </div>
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                    <UFormField label="Given Name">
                        <UInput v-model="instance_context.given_name" class="w-full"></UInput>
                    </UFormField>
                    <UFormField label="Family Name">
                        <UInput v-model="instance_context.family_name" class="w-full"></UInput>
                    </UFormField>
                </div>
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2" :hidden="email.template_type === 'round'">
                    <UFormField label="Credentials Username">
                        <UInput v-model="instance_context.credentials_username" class="w-full"></UInput>
                    </UFormField>
                    <UFormField label="Credentials Password">
                        <UInput v-model="instance_context.credentials_password" class="w-full"></UInput>
                    </UFormField>
                </div>
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                    <UFormField label="School">
                        <UInput v-model="instance_context.school" class="w-full"></UInput>
                    </UFormField>
                    <UFormField label="Grade">
                        <UInput v-model="instance_context.grade" class="w-full"></UInput>
                    </UFormField>
                </div>
            </section>
        </div>
      </template>
    </div>
    <div>
      <UTabs
          :items="[{ icon: 'i-lucide-eye', label: 'Preview', value: 'preview' }]" model-value="preview" variant="link"></UTabs>
      <div class="rounded border border-muted h-full">
        <!-- eslint-disable-next-line vue/no-v-html -->
        <div v-if="preview.html" v-html="preview.html"/>
        <UError v-if="preview.error" :error="{statusMessage: 'Cannot load preview', message: preview.error}"/>
      </div>
    </div>
  </div>
  <UEmpty
      v-else
      icon="i-lucide-mail"
      title="Email template not found"
      description="It looks like this template does not exist"
      :actions="[
        {
          icon: 'i-lucide-circle-arrow-right',
          label: 'Templates',
        },
        {
          icon: 'i-lucide-refresh-cw',
          label: 'Refresh',
          color: 'neutral',
          variant: 'subtle',
        },
      ]"
  />
</template>
