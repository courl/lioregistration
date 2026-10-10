import type { Badge } from "@/types/badge.ts"
import type { RegistrationId } from "@/types/registration.ts"

export type CredentialId = Badge<number, "credential_id">

export interface Credentials {
  id: CredentialId
  username: string
  password: string
  registration: {
    id: RegistrationId
    given_name: string
    family_name: string
  } | null
}
