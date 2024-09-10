use iced::keyboard::key;
use iced::widget::{column, text_input};
use iced::window::Id;
use iced::{keyboard, Command, Element, Event, Renderer, Subscription, Theme};
use pam::Client;

struct AuthCredentials {
    name: String,
    password: String,
}

use iced_sessionlock::actions::UnLockAction;
use iced_sessionlock::settings::Settings;
use iced_sessionlock::MultiApplication;

#[derive(Clone, Debug)]
enum Message {
    NameEntered(String),
    PasswordEntered(String),
    KeyboardEvent(Event),
}
impl MultiApplication for AuthCredentials {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                name: String::new(),
                password: String::new(),
            },
            Command::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("Iced waylock")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        return match message {
            Message::NameEntered(name) => {
                self.name = name;
                Command::none()
            }
            Message::PasswordEntered(password) => {
                self.password = password;
                Command::none()
            }

            Message::KeyboardEvent(event) => match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Enter),
                    ..
                }) => {
                    let mut client =
                        Client::with_password("system-auth").expect("Failed to init PAM client.");
                    client
                        .conversation_mut()
                        .set_credentials(&self.name, &self.password);

                    if client.authenticate().is_ok() {
                        return Command::single(UnLockAction.into());
                    }
                    Command::none()
                }
                _ => Command::none(),
            },
        };
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced::event::listen().map(Message::KeyboardEvent)
    }
    fn view(&self, _window: Id) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        column![
            text_input("Enter name", &self.name).on_input(Message::NameEntered),
            text_input("Enter password", &self.password).on_input(Message::PasswordEntered),
        ]
        .into()
    }
}
fn main() -> Result<(), iced_sessionlock::Error> {
    AuthCredentials::run(Settings::default())
}
