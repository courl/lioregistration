import type { Badge } from "@/types/badge.ts"
import type CalendarDate from "@/types/calendar-date.ts"
import type { Credentials } from "@/types/credentials.ts"
import type { Edition } from "@/types/edition.ts"
import type { Student } from "@/types/student.ts"

export type ReviewStatus = "pending" | "validated" | "refused"

export const FIRST_CONTACT = [
  "Teachers",
  "Homepage",
  "Friends",
  "LinkedIn",
  "Poster",
  "Instagram",
  "Other",
] as const

export const PROGRAMMING_LANGUAGE = ["Python", "C/C++", "Java"] as const

type FirstContact = (typeof FIRST_CONTACT)[number]
type ProgrammingLanguage = (typeof PROGRAMMING_LANGUAGE)[number]

export type RegistrationId = Badge<number, "registration_id">

export interface Registration {
  id: RegistrationId
  given_name: string
  family_name: string
  email: string
  school: string
  grade: string
  registration_date: CalendarDate
  review_status: ReviewStatus
}

export interface NewRegistration extends Omit<
  Registration,
  "id" | "registration_date" | "review_status" | "school"
> {
  school_name: string
  birthdate: CalendarDate
  gender: "male" | "female" | "other"
  address_street: string
  address_postal_code: string
  address_locality: string
  phone: string
  programming_languages: ProgrammingLanguage[]
  first_contact_source: FirstContact
  first_contact_other?: string | null
}

export interface RegistrationDetails
  extends Omit<NewRegistration, "school_name">, Registration {
  edition: Edition
  credentials: Credentials | null
  student: Student | null
  /** Filled client-side from the candidates endpoint while unlinked */
  matches?: [number, Student][]
}
