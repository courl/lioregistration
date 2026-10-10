import * as z from "zod"

export interface NewCredentials {
  username: string
  password: string
}

/** Parses one `username,password` pair per line (`,`, `;`, tab or spaces). */
export function parseCredentials(text: string): NewCredentials[] {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const [username = "", password = "", ...rest] = line.split(/[,;\t ]+/)
      return {
        username,
        password: rest.length > 0 ? "" : password,
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
