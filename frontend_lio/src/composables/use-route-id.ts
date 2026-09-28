import { computed, type ComputedRef } from "vue"
import { useRoute } from "vue-router"

export const useRouteId = <T extends number>(): ComputedRef<T | undefined> => {
  const route = useRoute()
  return computed(() => {
    const n = Number((route.params as Record<string, string | undefined>).id)
    return Number.isFinite(n) ? (n as T) : undefined
  })
}
