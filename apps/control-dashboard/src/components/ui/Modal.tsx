import { Dialog, DialogBackdrop, DialogPanel, DialogTitle } from '@headlessui/react'
import type { ReactNode } from 'react'

interface ModalProps {
  isOpen: boolean
  onClose: () => void
  title: string
  children: ReactNode
  footer?: ReactNode
}

export function Modal({ isOpen, onClose, title, children, footer }: ModalProps) {
  return (
    <Dialog open={isOpen} onClose={onClose} className="relative z-50">
      <DialogBackdrop
        transition
        className="fixed inset-0 bg-black/50 transition-opacity duration-300 ease-out data-[closed]:opacity-0"
      />
      <div className="fixed inset-0 z-10 overflow-y-auto">
        <div className="flex min-h-full items-center justify-center p-4">
          <DialogPanel
            transition
            className="w-full max-w-md rounded-lg bg-white shadow-xl transition-all duration-300 ease-out data-[closed]:scale-95 data-[closed]:opacity-0"
          >
            <div className="border-b border-secondary-200 px-4 py-3">
              <DialogTitle className="text-base font-semibold text-secondary-900">{title}</DialogTitle>
            </div>
            <div className="p-4">{children}</div>
            {footer && <div className="border-t border-secondary-200 px-4 py-3">{footer}</div>}
          </DialogPanel>
        </div>
      </div>
    </Dialog>
  )
}
