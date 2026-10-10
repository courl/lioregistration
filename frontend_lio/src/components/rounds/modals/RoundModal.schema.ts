import * as z from "zod"

import CalendarDate from "@/types/calendar-date.ts"

export const RoundSchema = z
  .object({
    name: z.string("Name is required").min(3, "Must be at least 3 characters"),
    starts_at: z.instanceof(CalendarDate, {
      message: "Start date is required",
    }),
    ends_at: z.instanceof(CalendarDate, { message: "End date is required" }),
  })
  .refine((data) => data.ends_at.getTime() >= data.starts_at.getTime(), {
    message: "End date cannot be earlier than start date.",
    path: ["ends_at"],
  })

type RoundData = z.output<typeof RoundSchema>

export interface RoundModalProps {
  title?: string
  round?: RoundData
}

export type RoundModalResult = RoundData | undefined
