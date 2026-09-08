import { Networks, Transaction } from "@stellar/stellar-sdk";
import { SorobanRpc } from "@stellar/stellar-sdk";
import { SOROBAN_RPC_URL, STELLAR_NETWORK } from "@/lib/constants";

export const NETWORK_PASSPHRASES: Record<string, string> = {
  testnet: Networks.TESTNET,
  mainnet: Networks.PUBLIC,
};

export function getNetworkPassphrase(
  network: string = STELLAR_NETWORK
): string {
  return NETWORK_PASSPHRASES[network] ?? Networks.TESTNET;
}

export function getRpcUrl(): string {
  return SOROBAN_RPC_URL;
}

export const rpcServer = new SorobanRpc.Server(getRpcUrl(), {
  allowHttp: getRpcUrl().startsWith("http://"),
});

export type SubmitTransactionResult = SorobanRpc.Api.GetTransactionResponse;

export async function submitTransaction(
  tx: Transaction
): Promise<SubmitTransactionResult> {
  const simulation = await rpcServer.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(simulation)) {
    throw new Error(simulation.error);
  }

  const prepared = await rpcServer.prepareTransaction(tx);

  const sendResponse = await rpcServer.sendTransaction(prepared);
  if (sendResponse.status === "PENDING") {
    return pollForResult(sendResponse.hash);
  }

  if (sendResponse.errorResult) {
    throw new Error(`Transaction failed: ${sendResponse.errorResult.toString()}`);
  }

  throw new Error(
    `Transaction send failed with status: ${sendResponse.status}`
  );
}

async function pollForResult(
  hash: string,
  attempts = 30
): Promise<SubmitTransactionResult> {
  for (let i = 0; i < attempts; i++) {
    const response = await rpcServer.getTransaction(hash);
    if (response.status === "SUCCESS" || response.status === "FAILED") {
      return response;
    }
    await new Promise((resolve) => setTimeout(resolve, 2_000));
  }
  throw new Error("Transaction timed out waiting for confirmation");
}
