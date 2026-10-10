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

export interface EmailTemplateDetails extends EmailTemplate {
  template: string
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
