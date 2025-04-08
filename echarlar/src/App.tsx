import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import "./App.css";
import Chat from "./components/chat";

function App() {
    const [connected, setConnected] = useState<boolean>(false);
    const [connecting, setConnecting] = useState<boolean>(false);
    const [message, setMessage] = useState<string>("");
    const [response, setResponse] = useState<string>("");
    const [history, setHistory] = useState<string[]>([]);

    const connectToServer = async (): Promise<void> => {
        if (connecting || connected) {
            return;
        }
        setConnecting(true);

        try {
            await invoke("connect_to_server");
            setConnected(true);
        } catch (err) {
            console.error("Connection error:", err);
        } finally {
            setConnecting(false);
        }
    };

    const sendMessage = async (): Promise<void> => {
        if (connecting) {
            return;
        }
        setConnecting(true);

        try {
            console.log('sendMessage...', message);
            const reply = await invoke<string>("send_message", {message});
            setHistory((prev) => [...prev, `You: ${message}`, `Server: ${reply}`]);
            setMessage("");
            setResponse(reply);
        } catch (err) {
            console.error("Send error:", err);
        } finally {
            setConnecting(false);
        }
    };

    const onMessageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setMessage(e.target.value);
    };

    useEffect(() => {
        connectToServer();
    }, []);

    return (
        <div className="container">
            <Chat
                history={history}
                message={message}
                onMessageChange={onMessageChange}
                onSendMessage={sendMessage}
                connected={connected}
            />
        </div>
    );
}

export default App;
