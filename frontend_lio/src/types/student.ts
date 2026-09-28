import type { Badge } from "@/types/badge.ts"
import type { Edition } from "@/types/edition.ts"
import type { Registration } from "@/types/registration.ts"

export type StudentId = Badge<number, "student_id">

export interface Student {
  id: StudentId
  given_name: string
  family_name: string
  email: string
  birthdate: Date
  awards: {
    round: string
    award: "gold" | "silver" | "bronze" | "honorableMention"
    count: number
  }[]
}

export interface StudentDetails extends Student {
  participations: {
    edition: Edition
    registration: Registration
    rounds: {
      id: number
      name: string
      state: "accepted" | "invited" | "declined" | "noshow"
      scores: {
        task: string
        score: number
        max_points: number
      }[]
    }[]
  }[]
}
