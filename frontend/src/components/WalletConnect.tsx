import React, { useState } from "react";
import { web3Enable, web3Accounts } from "@polkadot/extension-dapp";
import { InjectedAccount } from "@polkadot/extension-inject/types";
import { Button } from "@/components/ui/button.tsx";

const WalletConnect: React.FC = () => {
    const [account, setAccount] = useState<InjectedAccount | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState<boolean>(false);

    const connectWallet = async () => {
        setLoading(true);
        setError(null);
        try {
            const extensions = await web3Enable("Eduverse");
            if (extensions.length === 0) {
                setError("No extension found");
                return;
            }

            // Get all accounts from the extension
            const allAccounts = await web3Accounts();
            if (allAccounts.length > 0) {
                setAccount(allAccounts[0]); // Set the first account as connected account
            } else {
                setError("No accounts found");
            }
        } catch (err) {
            console.error(err);
            setError("An error occurred while connecting the wallet");
        } finally {
            setLoading(false);
        }
    };

    return (
        <div>
            {account ? (
                <div>
                    <h2>Connected Account: {account.address}</h2>
                </div>
            ) : (
                <div>
                    <h2>Polkadot Wallet Login</h2>
                    <Button
                        variant="outline"
                        color="primary"
                        onClick={connectWallet}
                        disabled={loading}
                    >
                        {loading ? "Connecting..." : "Connect Wallet"}
                    </Button>
                    {error && <p className="accent-red-800">{error}</p>}
                </div>
            )}
        </div>
    );
};

export default WalletConnect;
