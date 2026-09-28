import * as z from "zod"

import type CalendarDate from "@/types/calendar-date.ts"

export const RoundSchema = z
  .object({
    name: z.string("Name is required").min(3, "Must be at least 3 characters"),
    starts_at: z.date("Start date is required"),
    ends_at: z.date("End date is required"),
  })
  .refine((data) => data.ends_at > data.starts_at, {
    message: "End date cannot be earlier than start date.",
    path: ["ends_at"],
  })

export type RoundData = z.output<typeof RoundSchema>

type RoundDataCalendarDate = Override<
  RoundData,
  {
    starts_at: CalendarDate
    ends_at: CalendarDate
  }
>

export interface RoundModalProps {
  title?: string
  round?: RoundDataCalendarDate
}

type Override<T, R> = Omit<T, keyof R> & R
export type RoundModalResult = RoundDataCalendarDate | undefined
