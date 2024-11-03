// src/App.tsx
import React from "react";
import WalletConnect from "@/components/WalletConnect.tsx";
// import PhaserGame from "@/components/ui/Phaser.tsx";
import WebSocketClient from "@/components/WebSocketClient.tsx"; // Import the new component

const App: React.FC = () => {
    return (
        <div>
            <h1>Eduverse</h1>
            <WalletConnect />
            {/*<PhaserGame />*/}
            <WebSocketClient />
        </div>
    );
};

export default App;
