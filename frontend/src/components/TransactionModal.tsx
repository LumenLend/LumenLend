"use client";

export function TransactionModal({
  open,
  title,
  assetSymbol,
  amount,
  estimatedGas,
  healthFactorAfter,
  onConfirm,
  onCancel,
  pending = false,
}: {
  open: boolean;
  title: string;
  assetSymbol: string;
  amount: string;
  estimatedGas?: string;
  healthFactorAfter?: string;
  onConfirm: () => void;
  onCancel: () => void;
  pending?: boolean;
}) {
  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      role="dialog"
      aria-modal="true"
      aria-label={title}
    >
      <div className="w-full max-w-md rounded-lg border border-surface-light bg-surface p-6">
        <h2 className="text-lg font-semibold">{title}</h2>

        <dl className="mt-4 space-y-3 text-sm">
          <div className="flex items-center justify-between">
            <dt className="text-gray-400">Asset</dt>
            <dd className="font-medium">{assetSymbol}</dd>
          </div>
          <div className="flex items-center justify-between">
            <dt className="text-gray-400">Amount</dt>
            <dd className="font-medium">{amount}</dd>
          </div>
          {estimatedGas && (
            <div className="flex items-center justify-between">
              <dt className="text-gray-400">Estimated Gas</dt>
              <dd className="font-medium">{estimatedGas}</dd>
            </div>
          )}
          {healthFactorAfter && (
            <div className="flex items-center justify-between">
              <dt className="text-gray-400">Health Factor After</dt>
              <dd className="font-medium">{healthFactorAfter}</dd>
            </div>
          )}
        </dl>

        <div className="mt-6 flex gap-3">
          <button
            type="button"
            onClick={onConfirm}
            disabled={pending}
            className="flex-1 rounded-md bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700 disabled:cursor-not-allowed disabled:opacity-60"
          >
            {pending ? "Confirming…" : "Confirm"}
          </button>
          <button
            type="button"
            onClick={onCancel}
            disabled={pending}
            className="flex-1 rounded-md border border-gray-600 px-4 py-2 text-sm font-medium text-gray-300 transition-colors hover:border-gray-400 hover:text-white disabled:cursor-not-allowed disabled:opacity-60"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}