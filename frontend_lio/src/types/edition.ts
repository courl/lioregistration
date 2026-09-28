import type { Badge } from "@/types/badge.ts"
import type CalendarDate from "@/types/calendar-date.ts"

export type EditionId = Badge<number, "edition_id">

export interface ActiveEdition {
  name: string
  registration_opens_at: CalendarDate
  registration_closes_at: CalendarDate
}

export interface NewEdition extends ActiveEdition {
  year: number
}

export interface Edition extends NewEdition {
  id: EditionId
  active: boolean
}

export interface EditionStatistics {
  rounds: number
  students: number
  registrations: { day: CalendarDate; new: number; reviewed: number }[]
}
