// src/components/WebSocketClient.tsx
import React, { useEffect, useRef, useState } from 'react';

const WebSocketClient: React.FC = () => {
    const [message, setMessage] = useState<string>('');
    const [log, setLog] = useState<string[]>([]);
    const socketRef = useRef<WebSocket | null>(null);

    useEffect(() => {
        // Create a WebSocket connection
        socketRef.current = new WebSocket('ws://127.0.0.1:8080');

        // Event handler for when the connection is opened
        socketRef.current.onopen = () => {
            console.log('WebSocket connected');
            setLog(prev => [...prev, 'WebSocket connected']);
        };

        // Event handler for receiving messages
        socketRef.current.onmessage = (event) => {
            console.log('Message from server:', event.data);
            setLog(prev => [...prev, `Message from server: ${event.data}`]);
        };

        // Event handler for errors
        socketRef.current.onerror = (error) => {
            console.error('WebSocket error:', error);
            setLog(prev => [...prev, `WebSocket error type: ${error.type}`]);
        };

        // Event handler for when the connection is closed
        socketRef.current.onclose = () => {
            console.log('WebSocket closed');
            setLog(prev => [...prev, 'WebSocket closed']);
        };

        // Cleanup function to close the socket when the component unmounts
        return () => {
            if (socketRef.current) {
                socketRef.current.close();
            }
        };
    }, []);

    const sendMessage = () => {
        if (socketRef.current && socketRef.current.readyState === WebSocket.OPEN) {
            socketRef.current.send(message);
            setLog(prev => [...prev, `Sent: ${message}`]);
            setMessage(''); // Clear input after sending
        } else {
            console.log('WebSocket is not open');
        }
    };

    return (
        <div>
            <h2>WebSocket Client</h2>
            <input
                type="text"
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                placeholder="Type a message"
            />
            <button onClick={sendMessage}>Send Message</button>
            <div>
                <h3>Log:</h3>
                <ul>
                    {log.map((msg, index) => (
                        <li key={index}>{msg}</li>
                    ))}
                </ul>
            </div>
        </div>
    );
};

export default WebSocketClient;
