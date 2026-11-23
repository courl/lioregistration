import * as z from "zod"

import type { Participant, ParticipationState, Task } from "@/types/round.ts"

export interface ParticipationProps {
  participant: Participant
  tasks: Task[]
  title?: string
}

export interface ParticipationData {
  state: ParticipationState
  points: (number | null)[]
}

export type ParticipationResult = ParticipationData | undefined

export const calculateParticipationSchema = (
  tasks: Task[],
  recorded: (number | null)[],
) =>
  z.object({
    state: z.enum(["accepted", "invited", "declined", "noShow"]),
    points: z
      .array(z.number("Must be a number").nullable())
      .superRefine((points, context) => {
        for (const [index, task] of tasks.entries()) {
          const value = points[index] ?? 0

          // The API can change a score but not remove it
          if (points[index] === null && recorded[index] !== null) {
            context.addIssue({
              code: "custom",
              path: [index],
              message: "A recorded score cannot be cleared",
            })
          }

          if (value < 0) {
            context.addIssue({
              code: "custom",
              path: [index],
              message: "Cannot be negative",
            })
          }

          if (value > task.max_points) {
            context.addIssue({
              code: "custom",
              path: [index],
              message: `Maximum is ${task.max_points.toString()}`,
            })
          }
        }
      }),
  })
