import {
  computed,
  type ComputedRef,
  inject,
  type InjectionKey,
  type MaybeRefOrGetter,
  provide,
  toValue,
} from "vue"

import type { Edition } from "@/types/edition.ts"

import { useAuth } from "@/composables/api.ts"

const canEditKey: InjectionKey<ComputedRef<boolean>> = Symbol("canEdit")

/**
 * Called by pages showing one edition's data: only admins may edit it, and only
 * while it is the active edition (past editions are read-only, as for observers).
 */
export const provideCanEdit = (
  edition: MaybeRefOrGetter<Edition | undefined>,
): ComputedRef<boolean> => {
  const auth = useAuth()
  const canEdit = computed(
    () => auth.isAdmin && toValue(edition)?.active === true,
  )
  provide(canEditKey, canEdit)
  return canEdit
}

/** Whether the surrounding edition's data may be edited, falls back to the role outside one. */
export const useCanEdit = (): ComputedRef<boolean> => {
  const auth = useAuth()
  return inject(
    canEditKey,
    computed(() => auth.isAdmin),
    false,
  )
}
