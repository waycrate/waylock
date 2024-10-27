use chrono::Local;
use iced::keyboard::key;
use iced::widget::{button, column, container, text, text_input, Image, Stack};
use iced::window::Id;
use iced::{
    keyboard, Alignment, Color, Element, Event, Length, Renderer, Subscription, Task as Command,
    Theme,
};
use iced_sessionlock::actions::UnLockAction;
use iced_sessionlock::settings::Settings;
use iced_sessionlock::MultiApplication;
use pam::Client;
use std::sync::LazyLock;
use uzers::{get_current_uid, get_user_by_uid};
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
    Step(StepMessage),
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
        Subscription::batch(vec![iced::event::listen().map(Message::EnterEvent)])
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

            Message::Step(step_msg) => self.steps.update(step_msg),
        }
    }

    fn view(&self, _window: Id) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        let Lock { steps, .. } = self;

        column![steps.view().map(Message::Step)].into()
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
                AuthStep::Welcome {
                    user_name: user_name.clone(),
                },
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
                            Err(e) => Message::Step(StepMessage::AuthError(format!("{}", e))),
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
    }

    fn welcome(user_name: &'a String) -> Element<'a, StepMessage> {
        let image = Image::new("assets/ferris.png")
            .width(Length::Fill)
            .height(Length::Fill)
            .opacity(10.0);

        let now = Local::now();
        let day = now.format("%A, %B %e").to_string();
        let time = now.format("%H:%M").to_string();
        let col = column![
            text(time).size(75),
            text(day).size(35),
            button(text(user_name).size(35))
                .width(Length::Fixed(400f32))
                .style(move |_theme, _status| {
                    button::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(
                            0.18, 0.18, 0.18,
                        ))),
                        text_color: Color::from_rgb(0.85, 0.85, 0.85),
                        border: iced::Border {
                            color: Color::from_rgb(0.3, 0.3, 0.3),
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        shadow: iced::Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                            offset: iced::Vector { x: 0.0, y: 2.0 },
                            blur_radius: 4.0,
                        },
                    }
                })
        ]
        .spacing(10)
        .padding(375)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

        let mut st = Stack::new();
        st = st.push(image);
        st = st.push(col);
        container(st).into()
    }

    fn auth(password: &'a str, auth_error: &'a str) -> Element<'a, StepMessage> {
        // TODO
        // Improve styles
        let now = Local::now();
        let day = now.format("%A, %B %e").to_string();
        let time = now.format("%H:%M").to_string();
        let col = column![
            // TODO
            // Add toggler icon for password
            text(time).size(75),
            text(day).size(35),
            text_input("Enter password", password)
                .on_input(StepMessage::PasswordEntered)
                .secure(true)
                .id(INPUT_ID.clone())
                .on_submit(StepMessage::Submit)
                .width(Length::Fixed(500f32))
                .size(30),
            text(auth_error),
        ]
        .padding(375)
        .spacing(10)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

        let image = Image::new("assets/ferris2.png")
            .width(Length::Fill)
            .height(Length::Fill);

        let mut st = Stack::new().width(Length::Fill).height(Length::Fill);
        st = st.push(image);
        st = st.push(col);

        container(st).into()
    }
}
