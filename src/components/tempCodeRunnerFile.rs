use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    ToggleTheme,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    dark_mode: bool,
    username: String,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            username,
            dark_mode: false
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
            Msg::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let toggle_theme = ctx.link().callback(|_| Msg::ToggleTheme);

        let main_container_class = if self.dark_mode {
            "flex w-screen bg-gray-900 text-white"
        } else {
            "flex w-screen bg-white text-black"
        };

        let sidebar_class = if self.dark_mode {
            "flex-none w-56 h-screen bg-gray-800"
        } else {
            "flex-none w-56 h-screen bg-gray-100"
        };

        let user_card_class = if self.dark_mode {
            "flex m-3 bg-gray-700 rounded-lg p-2"
        } else {
            "flex m-3 bg-white rounded-lg p-2"
        };

        let input_bg_class = if self.dark_mode {
            "block w-full py-2 pl-4 mx-3 bg-gray-700 text-white rounded-full outline-none"
        } else {
            "block w-full py-2 pl-4 mx-3 bg-gray-100 text-black rounded-full outline-none"
        };

        let button_class = if self.dark_mode {
            "p-3 shadow-sm w-10 h-10 rounded-full flex justify-center items-center bg-grey-700 hover:bg-grey-800"
        } else {
            "p-3 shadow-sm w-10 h-10 rounded-full flex justify-center items-center bg-grey-500 hover:bg-grey-600"
        };

        html! {
            <div class={main_container_class}>
                // Sidebar
                <div class={sidebar_class}>
                    <div class="text-xl p-3 flex justify-between items-center">
                        {"Users"}
                        <button onclick={toggle_theme} class="text-sm px-2 py-1 border rounded hover:bg-gray-600">
                            { if self.dark_mode { "☀️" } else { "🌙" } }
                        </button>
                    </div>
                    {
                        self.users.iter().map(|u| {
                            html!{
                                <div class={user_card_class}>
                                    <div>
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-xs justify-between">
                                            <div>{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-gray-400">
                                            {"Hi there!"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>

                // Chat Area
                <div class="grow h-screen flex flex-col">
                    <div class="w-full h-14 border-b-2 border-gray-300">
                        <div class="text-xl p-3">{"💬 Chat!"}</div>
                    </div>

                    <div class="w-full grow overflow-auto border-b-2 border-gray-300">
                        {
                            self.messages.iter().map(|m| {
                                let is_me = m.from == self.username;
                                let user = self.users.iter().find(|u| u.name == m.from);
                                let avatar = user.map(|u| u.avatar.clone()).unwrap_or_default();

                                let alignment_class = if is_me { "justify-end" } else { "justify-start" };
                                let bubble_color_class = if is_me {
                                    if self.dark_mode {
                                        "bg-grey-600 text-white"
                                    } else {
                                        "bg-grey-100 text-black"
                                    }
                                } else if self.dark_mode {
                                    "bg-grey-800 text-white"
                                } else {
                                    "bg-grey-100 text-black"
                                };

                                let bubble_class = format!(
                                    "flex {} w-full px-4",
                                    alignment_class
                                );

                                html! {
                                    <div class={bubble_class}>
                                        <div class={format!("{} max-w-md rounded-lg p-3 flex items-end gap-2", bubble_color_class)}>
                                            { if !is_me {
                                                html! { <img class="w-6 h-6 rounded-full" src={avatar} alt="avatar" /> }
                                            } else {
                                                html! {}
                                            }}
                                            <div>
                                                <div class="text-xs font-semibold">{ &m.from }</div>
                                                {
                                                    if m.message.ends_with(".gif") {
                                                        html! { <img class="mt-2 rounded" src={m.message.clone()} /> }
                                                    } else {
                                                        html! { <div>{ m.message.clone() }</div> }
                                                    }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>

                    // Input area
                    <div class="w-full h-14 flex px-3 items-center">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Message" class={input_bg_class} name="message" required=true />
                        <button onclick={submit} class={button_class}>
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                                <path d="M0 0h24v24H0z" fill="none"></path>
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }

}