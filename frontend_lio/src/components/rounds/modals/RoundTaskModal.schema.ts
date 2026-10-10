import * as z from "zod"

export const TaskSchema = z.object({
  name: z.string("Name is required").min(3, "Must be at least 3 characters"),
  ordinal: z.int("Ordinal is required").gte(0, "Cannot be negative"),
  max_points: z.int("Points are required").gte(0, "Cannot be negative"),
})

export type TaskData = z.output<typeof TaskSchema>

export interface TaskDataProps {
  title?: string
  task?: TaskData
}

export type TaskModalResult = TaskData | undefined
