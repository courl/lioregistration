import { onScopeDispose, type Ref, ref, watch } from "vue"

/** A copy of `source` that only follows it after `delay` ms without changes. */
export const useDebounced = <T>(source: Ref<T>, delay = 250): Ref<T> => {
  const debounced = ref(source.value) as Ref<T>
  let timer: ReturnType<typeof setTimeout> | undefined

  watch(source, (value) => {
    clearTimeout(timer)
    timer = setTimeout(() => {
      debounced.value = value
    }, delay)
  })
  onScopeDispose(() => {
    clearTimeout(timer)
  })

  return debounced
}
