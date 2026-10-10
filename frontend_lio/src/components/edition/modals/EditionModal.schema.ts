import * as z from "zod"

import CalendarDate from "@/types/calendar-date.ts"

export const EditionSchema = z
  .object({
    name: z.string("Name is required").min(3, "Must be at least 3 characters"),
    year: z.int("Year is required").gt(0, "Year must be positive"),
    registration_opens_at: z.instanceof(CalendarDate, {
      message: "Registration open date is required",
    }),
    registration_closes_at: z.instanceof(CalendarDate, {
      message: "Registration close date is required",
    }),
  })
  .refine(
    (data) =>
      data.registration_opens_at.getTime() <=
      data.registration_closes_at.getTime(),
    {
      message: "Registration close date cannot be earlier than open date.",
      path: ["registration_closes_at"],
    },
  )

export type EditionData = z.output<typeof EditionSchema>

export interface EditionModalProps {
  title?: string
  edition?: EditionData
}

export type EditionModalResult = EditionData | undefined
