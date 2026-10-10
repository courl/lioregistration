import type { Task, TaskId } from "@/types/round.ts"

export interface RoundBulkScoreProps {
  tasks: Task[]
}

export interface RoundScoreData {
  credentials_username: string
  task_scores: [TaskId, number][]
}

export type RoundBulkScoreResult = RoundScoreData[] | undefined

export interface ParsedHeader {
  name: string
  isUserColumn: boolean
  taskId?: TaskId
}

export interface ParsedData {
  headers: ParsedHeader[]
  rows: Record<string, string>[]
}

export function parseInput(text: string, tasks: Task[]): ParsedData {
  const lines = text
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean)

  if (lines.length === 0) {
    return { headers: [], rows: [] }
  }

  const isCsv = lines[0].includes(",") || lines[0].includes(";")
  const separator = lines[0].includes(";") ? ";" : ","

  const raw: string[][] = isCsv
    ? lines.map((l) => l.split(separator).map((v) => v.trim()))
    : lines.map((l) => l.split(/\s+/))

  const headers = raw[0]

  const rows = raw
    .slice(1)
    .map((values) =>
      Object.fromEntries(headers.map((h, index) => [h, values[index] ?? ""])),
    )

  let foundUser = false
  const parsedHeaders = headers.map((header): ParsedHeader => {
    const values = rows.map((r) => r[header])

    const mostlyStrings =
      values.length > 0 && values.every((v) => Number.isNaN(Number(v)))

    const looksLikeUser = /user|participant|name/i.test(header)
    // Only the first matching column is the user column
    const isUserColumn = !foundUser && looksLikeUser && mostlyStrings
    foundUser ||= isUserColumn
    let taskId = undefined
    for (const task of tasks) {
      if (task.name.toLowerCase() === header.toLowerCase()) {
        taskId = task.id
      }
    }
    return {
      name: header,
      taskId,
      isUserColumn,
    }
  })

  return {
    headers: parsedHeaders,
    rows,
  }
}
