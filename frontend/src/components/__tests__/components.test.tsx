import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { HealthFactorBar } from "@/components/HealthFactorBar";
import { WalletConnect } from "@/components/WalletConnect";
import { TransactionModal } from "@/components/TransactionModal";

vi.mock("@/hooks/useWallet", () => ({
  useWallet: () => ({
    address: "",
    isConnected: false,
    isConnecting: false,
    connect: vi.fn(),
    disconnect: vi.fn(),
    signTransaction: vi.fn(),
  }),
}));

describe("HealthFactorBar", () => {
  it("renders the numeric health factor", () => {
    render(<HealthFactorBar healthFactor={1500000000000000000n} />);
    expect(screen.getByRole("meter")).toHaveAttribute(
      "aria-valuenow",
      "1.5"
    );
  });

  it("returns a meter element with accessible label", () => {
    const { container } = render(
      <HealthFactorBar healthFactor={1200000000000000000n} />
    );
    expect(container.querySelector('[role="meter"]')).not.toBeNull();
  });
});

describe("TransactionModal", () => {
  it("does not render when closed", () => {
    const { container } = render(
      <TransactionModal
        open={false}
        title="Confirm"
        assetSymbol="XLM"
        amount="10"
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />
    );
    expect(container.querySelector('[role="dialog"]')).toBeNull();
  });

  it("renders asset and amount when open", () => {
    render(
      <TransactionModal
        open
        title="Confirm Deposit"
        assetSymbol="XLM"
        amount="100"
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />
    );
    expect(screen.getByRole("dialog")).toHaveAttribute(
      "aria-label",
      "Confirm Deposit"
    );
    expect(screen.getByText("XLM")).toBeInTheDocument();
  });

  it("calls onConfirm when Confirm is clicked", () => {
    const onConfirm = vi.fn();
    render(
      <TransactionModal
        open
        title="Confirm"
        assetSymbol="USDC"
        amount="5"
        onConfirm={onConfirm}
        onCancel={vi.fn()}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: "Confirm" }));
    expect(onConfirm).toHaveBeenCalled();
  });
});

describe("WalletConnect", () => {
  it("shows Connect Wallet button when disconnected", () => {
    render(<WalletConnect />);
    expect(
      screen.getByRole("button", { name: "Connect Stellar wallet" })
    ).toBeInTheDocument();
  });
});