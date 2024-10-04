use iced::keyboard::key;
use iced::widget::{column, image, text, text_input, Column};
use iced::window::Id;
use iced::{
    keyboard, Alignment, Element, Event, Length, Renderer, Subscription, Task as Command, Theme,
};
use iced_sessionlock::actions::UnLockAction;
use iced_sessionlock::settings::Settings;
use iced_sessionlock::MultiApplication;
use pam::Client;
use uzers::{get_user_by_uid, get_current_uid};
use std::sync::LazyLock;
static INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

fn main() -> Result<(), iced_sessionlock::Error> {
    Lock::run(Settings::default())
}

struct Lock {
    steps: AuthSteps,
}

impl TryInto<UnLockAction> for Message {
    type Error = Self;
    fn try_into(self) -> Result<UnLockAction, Self::Error> {
        if let Self::Unlock = self {
            return Ok(UnLockAction);
        }
        Err(self)
    }
}

#[derive(Debug, Clone)]
enum Message {
    NextPressed,
    StepMessage(StepMessage),
    EnterEvent(Event),
    Unlock,
}
impl MultiApplication for Lock {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                steps: AuthSteps::new(),
            },
            Command::none(),
        )
    }
    fn namespace(&self) -> String {
        String::from("Waylock")
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            iced::event::listen().map(Message::EnterEvent),
        ])
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {

            Message::NextPressed => {
                self.steps.advance();
                text_input::focus(INPUT_ID.clone())
            }

            Message::EnterEvent(event) => match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Enter),
                    ..
                }) => {
                    let message = Message::NextPressed;
                    Command::perform(async { message }, |msg| msg)
                }
                _ => Command::none(),
            },

            Message::Unlock => Command::done(message),

            Message::StepMessage(step_msg) => self.steps.update(step_msg),
        }
    }

    fn view(&self, _window: Id) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        let Lock { steps, .. } = self;

        // TODO
        // Remove Next, Back and Unlock Button

        column![steps.view().map(Message::StepMessage)].into()
    }
}

struct AuthSteps {
    steps: Vec<AuthStep>,
    current: usize,
}


impl AuthSteps {
    fn new() -> AuthSteps {
        let user = get_user_by_uid(get_current_uid()).unwrap();
        let user_name = user.name().to_string_lossy().to_string().clone();
        Self {
            steps: vec![
                AuthStep::Welcome { user_name: user_name.clone() },
                AuthStep::Auth {
                    name: user_name.clone(),
                    password: String::new(),
                    auth_error: String::new(),
                },
            ],
            current: 0,
        }
    }

    fn update(&mut self, msg: StepMessage) -> Command<Message> {
        self.steps[self.current].update(msg)
    }

    fn view(&self) -> Element<StepMessage> {
        self.steps[self.current].view()
    }

    fn advance(&mut self) {
        if self.can_continue() {
            self.current += 1;
        }
    }

    fn can_continue(&self) -> bool {
        self.current + 1 < self.steps.len() && self.steps[self.current].can_continue()
    }
}

enum AuthStep {
    Welcome {
        user_name: String,
    },
    Auth {
        name: String,
        password: String,
        auth_error: String,
    },
}

#[derive(Clone, Debug)]
enum StepMessage {
    PasswordEntered(String),
    Submit,
    AuthError(String),
}

impl<'a> AuthStep {
    fn update(&mut self, msg: StepMessage) -> Command<Message> {
        match msg {
            StepMessage::AuthError(auth_error) => {
                if let AuthStep::Auth {
                    auth_error: error, ..
                } = self
                {
                    *error = auth_error;
                }
                Command::none()

            }

            StepMessage::PasswordEntered(password) => {
                if let AuthStep::Auth {
                    password: current_password,
                    ..
                } = self
                {
                    *current_password = password;
                }
                Command::none()
            }

            StepMessage::Submit => {
                if let AuthStep::Auth {
                    name,
                    password,
                    auth_error: _auth_error,
                } = self
                {
                    let name = name.clone();
                    let password = password.clone();
                    return Command::perform(
                        async move {
                            let mut client = Client::with_password("system-auth")
                                .expect("Failed to init PAM client.");
                            client.conversation_mut().set_credentials(&name, &password);
                            client.authenticate()
                        },
                        |result| match result {
                            Ok(_) => Message::Unlock,
                            Err(e) => Message::StepMessage(StepMessage::AuthError(
                                format!("{}", e),
                            )),
                        },
                    );
                }
                Command::none()
            }

        }
    }

    fn can_continue(&self) -> bool {
        match self {
            AuthStep::Welcome { .. } => true,
            AuthStep::Auth { .. } => true,
        }
    }

    fn view(&self) -> Element<StepMessage> {
        match self {
            AuthStep::Welcome { user_name } => Self::welcome(user_name),
            AuthStep::Auth {
                name: _,
                password,
                auth_error,
            } => Self::auth(password, auth_error),
        }
        .into()
    }

    fn welcome(user_name: &'a String) -> Column<'a, StepMessage> {
        column![text(user_name).size(30)]
            .padding(500)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn auth(password: &'a str, auth_error: &'a str) -> Column<'a, StepMessage> {
        // TODO
        // Improve styles
        column![
            // TODO
            // Add toggler icon for password
            image(format!("{}/assets/img.png", env!("CARGO_MANIFEST_DIR"))).width(250),
            text_input("Enter password", password)
                .on_input(StepMessage::PasswordEntered)
                .secure(true)
                .id(INPUT_ID.clone())
                .on_submit(StepMessage::Submit)
                .width(Length::Fixed(500f32))
                .size(30),
            text(auth_error),
        ]
        .padding(200)
        .spacing(10)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
