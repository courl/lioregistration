import type {
  TaskDataProps,
  TaskModalResult,
} from "@/components/rounds/modals/RoundTaskModal.schema.ts"

import RoundTaskModal from "@/components/rounds/modals/RoundTaskModal.vue"

export const useRoundTaskModal = () => {
  const overlay = useOverlay()

  return async (options: TaskDataProps): Promise<TaskModalResult> => {
    const modal = overlay.create(RoundTaskModal, {
      destroyOnClose: true,
      props: options as InstanceType<typeof RoundTaskModal>["$props"],
    })

    return modal.open() as Promise<TaskModalResult>
  }
}
