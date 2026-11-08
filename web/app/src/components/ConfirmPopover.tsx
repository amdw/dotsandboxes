// A small popover asking the user to confirm an action, closed by Cancel, Escape or
// clicking anywhere outside it and the control which opened it (its parent element)
import { useLayoutEffect, useRef } from "preact/hooks";

interface ConfirmPopoverProps {
  question: string;
  confirmLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmPopover({ question, confirmLabel, onConfirm, onCancel }: ConfirmPopoverProps) {
  const root = useRef<HTMLDivElement>(null);
  const confirm = useRef<HTMLButtonElement>(null);

  // A layout effect, so focus moves and the listeners are in place as soon as it appears
  useLayoutEffect(() => {
    confirm.current?.focus();
    const onPointerDown = (e: PointerEvent) => {
      // Clicks on the control which opened the popover (its container) are left to that control
      const container = root.current?.parentElement;
      if (container && !container.contains(e.target as Node)) onCancel();
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCancel();
    };
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [onCancel]);

  return (
    <div ref={root} class="popover" role="dialog" aria-label={question}>
      <span>{question}</span>
      <button ref={confirm} type="button" class="primary small" onClick={onConfirm}>
        {confirmLabel}
      </button>
      <button type="button" class="quiet small" onClick={onCancel}>
        Cancel
      </button>
    </div>
  );
}
