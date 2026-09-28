import type {
  EditionModalProps,
  EditionModalResult,
} from "@/components/edition/modals/EditionModal.schema.ts"

import EditionModal from "@/components/edition/modals/EditionModal.vue"

export const useEditionModal = () => {
  const overlay = useOverlay()

  return async (options: EditionModalProps): Promise<EditionModalResult> => {
    const modal = overlay.create(EditionModal, {
      destroyOnClose: true,
      props: options as InstanceType<typeof EditionModal>["$props"],
    })

    return modal.open() as Promise<EditionModalResult>
  }
}
