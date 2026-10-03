import type { Badge } from "@/types/badge.ts"
import type CalendarDate from "@/types/calendar-date.ts"

export type EmailTemplateId = Badge<number, "email_id">
export type EmailAttachmentId = Badge<number, "attachment_id">

export interface EmailTemplate {
  id: EmailTemplateId
  name: string
  template_type?: "registration" | "round" | "error"
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

export interface AttachmentPreview {
  name: string
  data_base64?: string
  error?: string
}

export interface EmailPreview {
  html?: string
  attachments: AttachmentPreview[]
  error?: string
}
