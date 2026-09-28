import * as z from "zod"

import type CalendarDate from "@/types/calendar-date.ts"

export const EditionSchema = z
  .object({
    name: z.string("Name is required").min(3, "Must be at least 3 characters"),
    year: z.number("Year is required").gt(0, "Year must be positive"),
    registration_opens_at: z.date("Registration open date is required"),
    registration_closes_at: z.date("Registration close date is required"),
  })
  .refine((data) => data.registration_opens_at <= data.registration_closes_at, {
    message: "Registration close date cannot be earlier than open date.",
    path: ["registration_closes_at"],
  })

export type EditionData = z.output<typeof EditionSchema>

export interface EditionModalProps {
  title?: string
  edition?: EditionData
}

type Override<T, R> = Omit<T, keyof R> & R
export type EditionModalResult =
  | Override<
      EditionData,
      {
        registration_opens_at: CalendarDate
        registration_closes_at: CalendarDate
      }
    >
  | undefined
