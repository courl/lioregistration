import type { Badge } from "@/types/badge.ts"

export type SchoolId = Badge<number, "school_id">

export interface School {
  id: SchoolId
  name: string
}
