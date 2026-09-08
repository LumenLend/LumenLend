import {
  Account,
  Operation,
  Transaction,
  TransactionBuilder,
  TimeoutInfinite,
  nativeToScVal,
  scValToNative,
  xdr,
} from "@stellar/stellar-sdk";
import { SorobanRpc } from "@stellar/stellar-sdk";
import { rpcServer, getNetworkPassphrase } from "@/lib/stellar";

export interface ContractCallResult {
  isOk: boolean;
  value: unknown;
}

function buildInvokeTx(
  contractId: string,
  method: string,
  args: unknown[],
  source: string
): Transaction {
  const scAddr = nativeToScVal(contractId, { type: "address" }).address();

  const invokeArgs = new xdr.InvokeContractArgs({
    contractAddress: scAddr,
    functionName: method,
    args: args.map((arg) => nativeToScVal(arg)),
  });

  const hostFunction = xdr.HostFunction.hostFunctionTypeInvokeContract(
    invokeArgs
  );

  const operation = Operation.invokeHostFunction({
    func: hostFunction,
    auth: [],
  });

  return new TransactionBuilder(
    new Account(source, "0"),
    {
      fee: "100",
      networkPassphrase: getNetworkPassphrase(),
    }
  )
    .addOperation(operation)
    .setTimeout(TimeoutInfinite)
    .build();
}

export async function readContract({
  contractId,
  method,
  args = [],
  source = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
}: {
  contractId: string;
  method: string;
  args?: unknown[];
  source?: string;
}): Promise<ContractCallResult> {
  if (!contractId) {
    return { isOk: false, value: undefined };
  }

  try {
    const tx = buildInvokeTx(contractId, method, args, source);
    const simulation = await rpcServer.simulateTransaction(tx);

    if (SorobanRpc.Api.isSimulationError(simulation)) {
      return { isOk: false, value: undefined };
    }

    const retval = simulation.result?.retval;
    if (!retval) {
      return { isOk: false, value: undefined };
    }

    return { isOk: true, value: scValToNative(retval) };
  } catch {
    return { isOk: false, value: undefined };
  }
}