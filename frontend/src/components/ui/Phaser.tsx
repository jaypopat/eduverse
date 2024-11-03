// src/PhaserGame.tsx
import React, { useEffect } from 'react';
import Phaser from 'phaser';

const PhaserGame: React.FC = () => {
    useEffect(() => {
        const config: Phaser.Types.Core.GameConfig = {
            type: Phaser.AUTO,
            width: 800,
            height: 600,
            physics: {
                default: 'arcade',
            },
            scene: {
                preload,
                create,
                update,
            },
        };

        const game = new Phaser.Game(config);

        function preload(this: Phaser.Scene) {
            this.load.image('sky', 'assets/sky.png'); // Load your assets here
            this.load.image('logo', 'assets/logo.png');
        }

        function create(this: Phaser.Scene) {
            this.add.image(400, 300, 'sky');
            const logo = this.physics.add.image(400, 150, 'logo');
            logo.setVelocity(100, 200);
            logo.setBounce(1, 1);
            logo.setCollideWorldBounds(true);
        }

        function update(this: Phaser.Scene) {
            // Game logic updates go here
        }

        return () => {
            // Clean up the game instance on component unmount
            game.destroy(true);
        };
    }, []);

    return <div id="phaser-game" />;

};

export default PhaserGame;
