"use client";

import { useCallback, useState } from "react";
import { StellarWalletsKit, allowAllModules } from "@creit.tech/stellar-wallets-kit";
import { WalletNetwork } from "@creit.tech/stellar-wallets-kit";
import { STELLAR_NETWORK } from "@/lib/constants";

const network =
  STELLAR_NETWORK === "mainnet"
    ? WalletNetwork.PUBLIC
    : WalletNetwork.TESTNET;

const kit = new StellarWalletsKit({
  network,
  selectedWalletId: "freighter",
  modules: allowAllModules(),
});

export function useWallet() {
  const [address, setAddress] = useState<string>("");
  const [isConnected, setIsConnected] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);

  const connect = useCallback(async () => {
    setIsConnecting(true);
    try {
      await kit.openModal({
        onWalletSelected: (wallet) => {
          kit.setWallet(wallet.id);
        },
      });
      const publicKey = await kit.getPublicKey();
      setAddress(publicKey);
      setIsConnected(true);
    } catch (err) {
      console.error("Failed to connect wallet", err);
      setIsConnected(false);
    } finally {
      setIsConnecting(false);
    }
  }, []);

  const disconnect = useCallback(() => {
    setAddress("");
    setIsConnected(false);
  }, []);

  const signTransaction = useCallback(
    async (xdr: string, publicKeys: string[]): Promise<string> => {
      const signingResult = await kit.signTx({
        xdr,
        publicKeys,
        network,
      });
      return signingResult.result;
    },
    []
  );

  return {
    address,
    isConnected,
    isConnecting,
    connect,
    disconnect,
    signTransaction,
  };
}