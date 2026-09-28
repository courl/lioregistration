import * as z from "zod"

export const TaskSchema = z.object({
  name: z.string("Name is required").min(3, "Must be at least 3 characters"),
  ordinal: z.number("Ordinal is required").gte(0, "Must be bigger than 0"),
  max_points: z.number("Points are required").gte(0, "Must be bigger than 0"),
})

export type TaskData = z.output<typeof TaskSchema>

export interface TaskDataProps {
  title?: string
  task?: TaskData
}

export type TaskModalResult = TaskData | undefined
