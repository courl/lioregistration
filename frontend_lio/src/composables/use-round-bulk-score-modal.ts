import type {
  RoundBulkScoreProps,
  RoundBulkScoreResult,
} from "@/components/rounds/modals/RoundBulkScoreModal.ts"

import RoundBulkScoreModal from "@/components/rounds/modals/RoundBulkScoreModal.vue"

export const useRoundBulkScoreModal = () => {
  const overlay = useOverlay()

  return async (props: RoundBulkScoreProps): Promise<RoundBulkScoreResult> => {
    const modal = overlay.create(RoundBulkScoreModal, {
      destroyOnClose: true,
      props: props as unknown as InstanceType<
        typeof RoundBulkScoreModal
      >["$props"],
    })

    return modal.open() as Promise<RoundBulkScoreResult>
  }
}
