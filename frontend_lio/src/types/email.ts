import type { Badge } from "@/types/badge.ts"
import type CalendarDate from "@/types/calendar-date.ts"

export type EmailTemplateId = Badge<number, "email_id">
type EmailAttachmentId = Badge<number, "attachment_id">

export interface EmailTemplate {
  id: EmailTemplateId
  name: string
  template_type?: "registration" | "round" | "error" | null
}

export interface EmailAttachment {
  id: EmailAttachmentId
  name: string
  template: string
}

export type CustomFieldKind = "text" | "multiline" | "date"

// A value the sender fills in for each send, inserted at `{{?key}}`.
export interface CustomField {
  key: string
  label: string
  kind: CustomFieldKind
  required: boolean
  default?: string | null
}

// Custom field values by key; dates as ISO `YYYY-MM-DD`.
export type CustomValues = Record<string, string>

export interface EmailTemplateDetails extends EmailTemplate {
  template: string
  custom_fields: CustomField[]
}

export type EmailTemplateUpdateRequest = Omit<EmailTemplateDetails, "id">

export interface EmailPreviewRequest {
  body: string
  global_context: {
    headline: string
    edition: string
    round?: string
    round_starts_at: CalendarDate
    round_ends_at: CalendarDate
    students?: [string, number][]
    custom?: CustomValues
  }
  instance_context: {
    given_name: string
    family_name: string
    school: string
    grade: string
    credentials_username?: string
    credentials_password?: string
  }
}

interface AttachmentPreview {
  name: string
  data_base64?: string | null
  error?: string | null
}

export interface EmailPreview {
  html?: string | null
  attachments: AttachmentPreview[]
  error?: string | null
}

export interface RoundEmailPreviewRequest {
  round_id: number
  student_id?: number
  headline: string
  body: string
  custom: CustomValues
}

export interface RoundEmailPreview extends EmailPreview {
  student_id: number
  recipients: { student_id: number; name: string }[]
}
