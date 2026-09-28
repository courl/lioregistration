import type {
  ParticipationProps,
  ParticipationResult,
} from "@/components/rounds/modals/RoundParticipationModal.schema.ts"

import RoundParticipationModal from "@/components/rounds/modals/RoundParticipationModal.vue"

export const useRoundParticipationModal = () => {
  const overlay = useOverlay()

  return async (options: ParticipationProps): Promise<ParticipationResult> => {
    const modal = overlay.create(RoundParticipationModal, {
      destroyOnClose: true,
      props: options as unknown as InstanceType<
        typeof RoundParticipationModal
      >["$props"],
    })

    return modal.open() as Promise<ParticipationResult>
  }
}
