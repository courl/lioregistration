import type {
  CredentialsModalProps,
  CredentialsModalResult,
} from "@/components/credentials/modals/CredentialsModal.schema.ts"

import CredentialsModal from "@/components/credentials/modals/CredentialsModal.vue"

export const useCredentialsModal = () => {
  const overlay = useOverlay()

  return async (
    options: CredentialsModalProps,
  ): Promise<CredentialsModalResult> => {
    const modal = overlay.create(CredentialsModal, {
      destroyOnClose: true,
      props: options as InstanceType<typeof CredentialsModal>["$props"],
    })

    return modal.open() as Promise<CredentialsModalResult>
  }
}
