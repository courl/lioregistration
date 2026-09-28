import type { ConfirmDialogProps } from "@/components/ConfirmDialog.schema.ts"

import ConfirmDialog from "@/components/ConfirmDialog.vue"

export const useConfirmDialog = () => {
  const overlay = useOverlay()

  return async (options: ConfirmDialogProps): Promise<boolean | undefined> => {
    const modal = overlay.create(ConfirmDialog, {
      destroyOnClose: true,
      props: options as InstanceType<typeof ConfirmDialog>["$props"],
    })

    return modal.open() as Promise<boolean | undefined>
  }
}
