import type { Badge } from "@/types/badge.ts"
import type CalendarDate from "@/types/calendar-date.ts"
import type { Edition } from "@/types/edition.ts"
import type { Student } from "@/types/student.ts"

export type RoundId = Badge<number, "round_id">
export type TaskId = Badge<number, "task_id">

export interface NewRound {
  name: string
  starts_at: CalendarDate
  ends_at: CalendarDate
}

export type UpdateRound = Partial<NewRound> & { id: RoundId }

export interface Round extends NewRound {
  id: RoundId
  edition: Edition
  total_points: number
  task_count: number
  participant_count: number
}

export interface NewTask {
  name: string
  ordinal: number
  max_points: number
}

export type UpdateTask = Partial<NewTask> & { id: TaskId }

export interface Task extends NewTask {
  id: TaskId
}

export type ParticipationState = "accepted" | "invited" | "declined" | "noShow"

export interface UpdateParticipant {
  state?: ParticipationState
  points: [TaskId, number][]
}

export interface Participant {
  student: Student
  state: ParticipationState
  points: (number | null)[] // points per task (null: not scored yet), aligned to round.tasks order (by ordinal)
  total: number
}

export interface RoundDetails extends Round {
  tasks: Task[]
  participants: Participant[]
}
