<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui"

import { computed, reactive, ref } from "vue"
import * as z from "zod"

import type { ActiveEdition } from "@/types/edition.ts"

import { useRegister } from "@/composables/use-register.ts"
import CalendarDate from "@/types/calendar-date.ts"
import { FIRST_CONTACT, PROGRAMMING_LANGUAGE } from "@/types/registration.ts"

defineProps<{ edition: ActiveEdition }>()
const emit = defineEmits<{
  registered: [data: RegisterData]
}>()
const schoolQuery = ref("")
const toast = useToast()
const submitLoading = ref<boolean>(false)
const { schools, register, loading } = useRegister(schoolQuery)

const createdSchools = ref<string[]>([])

const schoolsToDisplay = computed(() => [createdSchools.value, schools.value])

const RegisterSchema = z.object({
  given_name: z.string("Given name is required"),
  family_name: z.string("Family name is required"),
  birthdate: z.date("Birthdate is required"),
  gender: z.enum(["male", "female", "other"]),
  address_street: z.string("Street is required"),
  address_postal_code: z.string("Postal code is required"),
  address_locality: z.string("Locality is required"),
  email: z.email("Email is required"),
  phone: z.string("Phone number is required"),
  school_name: z.string("School is required"),
  grade: z.string("Grade is required"),
  programming_languages: z.enum(PROGRAMMING_LANGUAGE).array(),
  first_contact_source: z.enum(FIRST_CONTACT),
  first_contact_other: z.string().optional(),
})

type RegisterData = z.output<typeof RegisterSchema>
const state = reactive<Partial<RegisterData>>({})

async function onSubmit(event: FormSubmitEvent<RegisterData>) {
  submitLoading.value = true
  try {
    await register({
      ...event.data,
      birthdate: new CalendarDate({ date: event.data.birthdate }),
    })
    emit("registered", event.data)
  } catch (error) {
    console.error(error)
    toast.add({
      title: "Uh oh! Something went wrong.",
      description: "Registration failed",
      icon: "i-lucide-triangle-alert",
    })
  } finally {
    submitLoading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen bg-default">
    <div class="max-w-2xl mx-auto px-4 py-12">
      <h1 class="text-3xl font-bold mb-2">{{ edition.name }} Registration</h1>
      <p class="text-muted mb-8">
        Fill in the form below to register. You will receive your contest
        credentials by email.
      </p>
      <UForm
        :schema="RegisterSchema"
        :state="state"
        :disabled="submitLoading"
        class="space-y-10"
        @submit="onSubmit"
      >
        <section class="space-y-4">
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              Personal details
            </h2>
            <p class="text-sm text-muted">Tell us who you are.</p>
          </div>

          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <UFormField label="Given name" name="given_name" required>
              <UInput
                v-model="state.given_name"
                placeholder="Alex"
                class="w-full"
              />
            </UFormField>
            <UFormField label="Family name" name="family_name" required>
              <UInput
                v-model="state.family_name"
                placeholder="Dupont"
                class="w-full"
              />
            </UFormField>
          </div>

          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <UFormField label="Birthdate" name="birthdate" required>
              <UInputDate
                :range="false"
                class="w-full"
                icon="i-lucide-cake"
                @update:model-value="
                  (value) => {
                    state.birthdate = value ? value.toDate('UTC') : undefined
                  }
                "
              />
            </UFormField>
            <UFormField label="Gender" name="gender" required>
              <USelect
                v-model="state.gender"
                :items="[
                  { label: 'Male', value: 'male' },
                  { label: 'Female', value: 'female' },
                  { label: 'Other', value: 'other' },
                ]"
                placeholder="Select gender"
                class="w-full"
              />
            </UFormField>
          </div>
        </section>

        <USeparator />

        <section class="space-y-4">
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              Contact & address
            </h2>
            <p class="text-sm text-muted">Where and how we can reach you.</p>
          </div>

          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <UFormField label="Email" name="email" required>
              <UInput
                v-model="state.email"
                type="email"
                placeholder="alex@example.com"
                class="w-full"
                icon="i-lucide-mail"
              />
            </UFormField>
            <UFormField label="Phone" name="phone" required>
              <UInput
                v-model="state.phone"
                type="tel"
                placeholder="+352 621 123 456"
                class="w-full"
                icon="i-lucide-phone"
              />
            </UFormField>
          </div>

          <UFormField label="Street" name="address_street" required>
            <UInput
              v-model="state.address_street"
              placeholder="12 Rue de la Gare"
              class="w-full"
            />
          </UFormField>

          <div class="grid grid-cols-1 gap-4 sm:grid-cols-[140px_1fr]">
            <UFormField label="Postal code" name="address_postal_code" required>
              <UInput
                v-model="state.address_postal_code"
                placeholder="L-1234"
                class="w-full"
              />
            </UFormField>
            <UFormField label="Locality" name="address_locality" required>
              <UInput
                v-model="state.address_locality"
                placeholder="Luxembourg"
                class="w-full"
              />
            </UFormField>
          </div>
        </section>

        <USeparator />

        <section class="space-y-4">
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              School & background
            </h2>
            <p class="text-sm text-muted">Where you come from.</p>
          </div>

          <div class="grid grid-cols-1 gap-4 sm:grid-cols-[1fr_140px]">
            <UFormField label="School" name="school_name" required>
              <UInputMenu
                v-model="state.school_name"
                v-model:search-term="schoolQuery"
                :items="schoolsToDisplay"
                :loading="loading"
                create-item
                placeholder="Search for your school"
                class="w-full"
                icon="i-lucide-school"
                @create="
                  (x) => {
                    state.school_name = x
                    createdSchools = [...createdSchools, x]
                  }
                "
              />
            </UFormField>
            <UFormField label="Grade" name="grade" required>
              <UInput
                v-model="state.grade"
                placeholder="e.g. 4e, 1BAC"
                class="w-full"
              />
            </UFormField>
          </div>

          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <UFormField
              label="Programming languages"
              name="programming_languages"
            >
              <USelect
                v-model="state.programming_languages"
                :items="
                  PROGRAMMING_LANGUAGE.map((x) => ({ label: x, value: x }))
                "
                multiple
                placeholder="Select languages"
                class="w-full"
                icon="i-lucide-code"
              />
            </UFormField>

            <UFormField
              label="How did you hear about us?"
              name="first_contact_source"
              required
            >
              <USelect
                v-model="state.first_contact_source"
                :items="FIRST_CONTACT.map((x) => ({ label: x, value: x }))"
                placeholder="Select an option"
                class="w-full"
                icon="i-lucide-megaphone"
              />
            </UFormField>
          </div>
          <UFormField
            v-show="state.first_contact_source === 'Other'"
            name="first_contact_other"
          >
            <UTextarea
              v-model="state.first_contact_other"
              placeholder="Tell us more on how you heard about us"
              class="w-full"
            />
          </UFormField>
        </section>

        <UButton
          type="submit"
          size="lg"
          block
          :loading="loading || submitLoading"
        >
          Submit registration
        </UButton>
      </UForm>
    </div>
  </div>
</template>
