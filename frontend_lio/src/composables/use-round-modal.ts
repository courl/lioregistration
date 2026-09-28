import type {
  RoundModalProps,
  RoundModalResult,
} from "@/components/rounds/modals/RoundModal.schema.ts"

import RoundModal from "@/components/rounds/modals/RoundModal.vue"

export const useRoundModal = () => {
  const overlay = useOverlay()

  return async (options: RoundModalProps): Promise<RoundModalResult> => {
    const modal = overlay.create(RoundModal, {
      destroyOnClose: true,
      props: options as InstanceType<typeof RoundModal>["$props"],
    })

    return modal.open() as Promise<RoundModalResult>
  }
}
