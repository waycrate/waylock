use iced::window::Id;
use iced::{Command, Element, Renderer, Theme};
use iced::widget::{column, container, text_input, button};
use iced::widget::shader::wgpu::naga::MathFunction::Length;
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
    Unlock,
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

            Message::Unlock => {
                Command::single(UnLockAction.into())
            }
        }
    }

    fn view(&self, window: Id) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        column! [
            text_input("Enter name", &self.name).on_input(Message::NameEntered),
            text_input("Enter password", &self.password).on_input(Message::PasswordEntered),
            button("Unlock").on_press(Message::Unlock),
        ].into()
    }
}
fn main() -> Result<(), iced_sessionlock::Error> {

    AuthCredentials::run(Settings::default())
}
