import * as z from "zod"

import type { Participant, ParticipationState, Task } from "@/types/round.ts"

export interface ParticipationProps {
  participant: Participant
  tasks: Task[]
  title?: string
}

export interface ParticipationData {
  state: ParticipationState
  points: number[]
}

export type ParticipationResult = ParticipationData | undefined

export const calculateParticipationSchema = (tasks: Task[]) =>
  z.object({
    state: z.enum(["accepted", "invited", "declined", "noShow"]),
    points: z.array(z.number("Required")).superRefine((points, context) => {
      for (const [index, task] of tasks.entries()) {
        const value = points[index] ?? 0

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
