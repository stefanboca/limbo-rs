use iced::{Alignment, Color, Element, Event, Length, Task, Theme, theme::Palette, widget::row};
use iced_layershell::{
    Application,
    reexport::{Anchor, KeyboardInteractivity},
    settings::{LayerShellSettings, Settings, StartMode},
    to_layer_message,
};

use crate::{
    components::{icon, section, side},
    desktop_environment::{Monitor, MonitorInfo, get_monitor_workspaces},
    sections::{
        clock::{Clock, ClockMessage},
        workspaces::{Workspaces, WorkspacesMessage},
    },
};

mod components;
mod desktop_environment;
mod icons;
mod sections;

#[tokio::main]
pub async fn main() {
    // Workaround for https://github.com/friedow/centerpiece/issues/237
    // WGPU picks the lower power GPU by default, which on some systems,
    // will pick an IGPU that doesn't exist leading to a black screen.
    if std::env::var("WGPU_POWER_PREF").is_err() {
        unsafe {
            std::env::set_var("WGPU_POWER_PREF", "high");
        }
    }

    let monitors = desktop_environment::listen_monitors();

    let mut tasks = vec![];
    while let Ok(monitor) = monitors.recv() {
        let task = std::thread::spawn(move || {
            let name = monitor.name();
            let r = Limbo::run(Settings {
                layer_settings: LayerShellSettings {
                    size: Some((0, 40)),
                    exclusive_zone: 40,
                    anchor: Anchor::Top | Anchor::Left | Anchor::Right,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    start_mode: StartMode::TargetScreen(monitor.name()),
                    ..Default::default()
                },
                flags: Flags { monitor },
                id: None,
                fonts: Vec::new(),
                default_font: iced::Font {
                    family: iced::font::Family::Name("DejaVu Sans Mono"),
                    weight: iced::font::Weight::Normal,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                },
                default_text_size: iced::Pixels(14.0),
                antialiasing: false,
                virtual_keyboard_support: None,
            });
            println!("exiting on {name}");
            r
        });
        tasks.push(task);
    }

    for task in tasks {
        task.join().unwrap().unwrap();
    }
}

#[derive(Debug)]
struct Flags {
    monitor: Monitor,
}

struct Limbo {
    monitor: Monitor,
    monitor_info: MonitorInfo,

    clock: Clock,
    workspaces: Workspaces,
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    DesktopEvent(MonitorInfo),
    IcedEvent(Event),
    Clock(ClockMessage),
    Workspaces(WorkspacesMessage),
}

impl Application for Limbo {
    type Message = Message;
    type Flags = Flags;
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(flags: Self::Flags) -> (Self, Task<Message>) {
        let workspaces = get_monitor_workspaces(flags.monitor.id());
        (
            Self {
                monitor: flags.monitor,
                monitor_info: Default::default(),
                clock: Clock::new(),
                workspaces: Workspaces::new(workspaces),
            },
            Task::none(),
        )
    }

    fn namespace(&self) -> String {
        "Limbo".to_string()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let mut subscriptions = vec![
            self.monitor.subscription().map(Message::DesktopEvent),
            self.clock.subscription().map(Message::Clock),
        ];
        if let Some(workspace_subscription) = self.workspaces.subscription() {
            subscriptions.push(workspace_subscription.map(Message::Workspaces));
        }
        iced::Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DesktopEvent(monitor_info) => {
                self.monitor_info = monitor_info.clone();
                self.workspaces
                    .update(WorkspacesMessage::DesktopEvent(monitor_info));
                Task::none()
            }
            Message::IcedEvent(event) => {
                println!("{event:?}");
                Task::none()
            }
            Message::Clock(msg) => {
                self.clock.update(msg);
                Task::none()
            }
            Message::Workspaces(msg) => {
                self.workspaces.update(msg);
                Task::none()
            }
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        row![
            // Left
            side(
                Alignment::Start,
                row![
                    section(icon("nix-snowflake-white", None)),
                    section(icon("nix-snowflake-white", None)),
                    section(icon("nix-snowflake-white", None)),
                    section(icon("nix-snowflake-white", None))
                ]
                .spacing(12)
            ),
            // Center
            side(
                Alignment::Center,
                row![self.workspaces.view().map(Message::Workspaces)].spacing(12)
            ),
            // Right
            side(
                Alignment::End,
                row![self.clock.view().map(Message::Clock)].spacing(12)
            ),
        ]
        .padding([4, 8])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::custom(
            "internal".to_string(),
            Palette {
                text: Color::WHITE,
                background: Color::from_rgb(
                    0x1e as f32 / 255.0,
                    0x1e as f32 / 255.0,
                    0x2e as f32 / 255.0,
                ),
                // Unused
                primary: Color::BLACK,
                success: Color::BLACK,
                danger: Color::BLACK,
            },
        )
    }

    fn style(&self, theme: &Self::Theme) -> iced_layershell::Appearance {
        use iced_layershell::Appearance;

        Appearance {
            background_color: if self.monitor_info.show_transparent {
                Color::TRANSPARENT
            } else {
                theme.palette().background
            },
            text_color: theme.palette().text,
        }
    }
}
