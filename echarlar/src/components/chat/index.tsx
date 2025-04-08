import React from "react";
import {Button} from "react-chat-elements";
import {Input} from "react-chat-elements";
import {MessageList} from "react-chat-elements"
import "react-chat-elements/dist/main.css"

interface ChatProps {
    history: string[];
    message: string;
    onMessageChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
    onSendMessage: () => void;
    connected: boolean;
}

const Chat: React.FC<ChatProps> = ({
                                       history,
                                       message,
                                       onMessageChange,
                                       onSendMessage,
                                       connected,
                                   }) => {
    return (<>
        <div style={{position: "relative", height: "500px"}}>
            <MessageList
                className='message-list'
                lockable={true}
                toBottomHeight={'100%'}
                dataSource={[
                    {
                        position: "left",
                        type: "text",
                        title: "Kursat",
                        text: "Give me a message list example !",
                        date: new Date(),
                    },
                    {
                        position: "right",
                        type: "text",
                        title: "Emre",
                        text: "That's all.",
                    },
                ]}
            />
            <Input
                placeholder="Type here..."
                multiline={true}
                value={message}
                onChange={onMessageChange}
            />
            <Button text={"Send"} onClick={onSendMessage} title="Send"
                    // disabled={!connected || message.trim() === ""}
            />
        </div>
    </>)
}

export default Chat;
