import * as z from "zod"

export interface NewCredentials {
  username: string
  password: string
}

/**
 * Parses one `username,password` pair per line. Only the first `,`, `;`, tab or space
 * separates, so passwords may contain any of them.
 */
export function parseCredentials(text: string): NewCredentials[] {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const index = line.search(/[,;\s]/)
      return index === -1
        ? { username: line, password: "" }
        : {
            username: line.slice(0, index),
            password: line.slice(index + 1).trim(),
          }
    })
}

export const CredentialsSchema = z.object({
  text: z
    .string("Credentials are required")
    .trim()
    .min(1, "Credentials are required")
    .superRefine((text, context) => {
      const seen = new Set<string>()
      for (const [index, { username, password }] of parseCredentials(
        text,
      ).entries()) {
        if (!username || !password) {
          context.addIssue({
            code: "custom",
            message: `Line ${(index + 1).toString()}: expected 'username,password'`,
          })
          return
        }
        if (seen.has(username)) {
          context.addIssue({
            code: "custom",
            message: `Line ${(index + 1).toString()}: duplicate username '${username}'`,
          })
          return
        }
        seen.add(username)
      }
    }),
})

export type CredentialsData = z.output<typeof CredentialsSchema>

export interface CredentialsModalProps {
  title?: string
}

export type CredentialsModalResult = NewCredentials[] | undefined
