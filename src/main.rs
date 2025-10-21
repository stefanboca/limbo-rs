use std::collections::HashMap;

use iced::{
    Alignment, Element, Length, Task, Theme,
    daemon::{Appearance, DefaultStyle},
    event::wayland::{Event as WaylandEvent, LayerEvent, OutputEvent},
    platform_specific::shell::commands::{
        layer_surface::{destroy_layer_surface, get_layer_surface},
        subsurface::{Anchor, KeyboardInteractivity, Layer},
    },
    runtime::platform_specific::wayland::layer_surface::{
        IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
    },
    widget::{container, row},
    window,
};
use sctk::{output::OutputInfo, reexports::client::protocol::wl_output::WlOutput};

use crate::{
    components::{icon, section, side},
    desktop_environment::{Desktop, Event as DesktopEvent, WorkspaceInfo},
    sections::clock::{Clock, ClockMessage},
};

mod components;
mod desktop_environment;
mod icons;
mod sections;

#[tokio::main]
pub async fn main() -> iced::Result {
    // Workaround for https://github.com/friedow/centerpiece/issues/237
    // WGPU picks the lower power GPU by default, which on some systems,
    // will pick an IGPU that doesn't exist leading to a black screen.
    if std::env::var("WGPU_POWER_PREF").is_err() {
        unsafe {
            std::env::set_var("WGPU_POWER_PREF", "high");
        }
    }

    iced::daemon("limbo", Limbo::update, Limbo::view)
        .subscription(Limbo::subscription)
        .theme(Limbo::theme)
        .style(Limbo::style)
        .run_with(Limbo::new)
}

struct Bar {
    id: window::Id,
    wl_output: WlOutput,
}

struct Limbo {
    desktop: Desktop,
    workspace_infos: Option<Vec<WorkspaceInfo>>,

    output_infos: HashMap<WlOutput, OutputInfo>,
    bars: Vec<Bar>,

    clock: Clock,
}

#[derive(Debug, Clone)]
pub enum Message {
    WaylandEvent(WaylandEvent),
    DesktopEvent(DesktopEvent),
    Clock(ClockMessage),
}

impl Limbo {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                desktop: Desktop::new(),
                workspace_infos: None,
                output_infos: HashMap::new(),
                bars: Vec::new(),
                clock: Clock::new(),
            },
            Task::none(),
        )
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        let subscriptions = vec![
            iced::event::listen_with(|evt, _, _| {
                if let iced::Event::PlatformSpecific(iced::event::PlatformSpecific::Wayland(evt)) =
                    evt
                {
                    Some(Message::WaylandEvent(evt))
                } else {
                    None
                }
            }),
            self.desktop.subscription().map(Message::DesktopEvent),
            self.clock.subscription().map(Message::Clock),
        ];
        iced::Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DesktopEvent(DesktopEvent::WorkspacesChanged(workspace_infos)) => {
                self.workspace_infos = Some(workspace_infos);
                Task::none()
            }
            Message::WaylandEvent(evt) => match evt {
                WaylandEvent::Output(OutputEvent::Created(output_info), wl_output) => {
                    if let Some(output_info) = output_info {
                        self.output_infos.insert(wl_output.clone(), output_info);
                    }

                    self.spawn_bar(wl_output)
                }
                WaylandEvent::Output(OutputEvent::InfoUpdate(output_info), wl_output) => {
                    self.output_infos.insert(wl_output, output_info);
                    Task::none()
                }
                WaylandEvent::Output(OutputEvent::Removed, wl_output) => {
                    self.output_infos.remove(&wl_output);

                    let bars = self
                        .bars
                        .extract_if(.., |bar| bar.wl_output == wl_output)
                        .collect::<Vec<_>>();

                    Task::batch(bars.into_iter().map(|bar| destroy_layer_surface(bar.id)))
                }
                WaylandEvent::Layer(LayerEvent::Done, _wl_surface, id) => {
                    self.bars.retain(|bar| bar.id != id);
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Clock(msg) => {
                self.clock.update(msg);
                Task::none()
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if let Some(bar) = self.bars.iter().find(|bar| bar.id == window_id) {
            self.view_bar(bar)
        } else {
            unreachable!("All windows are bars");
        }
    }

    fn view_bar(&self, bar: &Bar) -> Element<'_, Message> {
        let content = row![
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
                row![self.clock.view().map(Message::Clock)].spacing(12)
            ),
            // Right
            side(
                Alignment::End,
                row![section(icon("nix-snowflake-white", None))].spacing(12)
            ),
        ]
        .padding([4, 8])
        .width(Length::Fill)
        .height(Length::Fill);

        let is_transparent = if let Some(workspace_infos) = &self.workspace_infos
            && let Some(output) = &self.output_infos.get(&bar.wl_output)
            && let Some(output_name) = &output.name
        {
            !workspace_infos
                .iter()
                .any(|w| w.is_active && w.has_windows && w.output.as_ref() == Some(output_name))
        } else {
            false
        };

        container(content)
            .style(move |theme| {
                let palette = theme.extended_palette();

                let background = if is_transparent {
                    None
                } else {
                    Some(palette.background.base.color.into())
                };

                iced::widget::container::Style {
                    icon_color: Some(palette.background.base.icon),
                    text_color: Some(palette.background.base.icon),
                    background,
                    ..Default::default()
                }
            })
            .into()
    }

    fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::CatppuccinMocha
    }

    fn style(&self, theme: &Theme) -> Appearance {
        Appearance {
            background_color: iced::Color::TRANSPARENT,
            ..DefaultStyle::default_style(theme)
        }
    }

    fn spawn_bar(&mut self, wl_output: WlOutput) -> Task<Message> {
        let id = window::Id::unique();

        self.bars.push(Bar {
            id,
            wl_output: wl_output.clone(),
        });

        get_layer_surface(SctkLayerSurfaceSettings {
            id,
            layer: Layer::Top,
            keyboard_interactivity: KeyboardInteractivity::None,
            pointer_interactivity: true,
            anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
            output: IcedOutput::Output(wl_output),
            namespace: "limbo:bar".to_string(),
            margin: IcedMargin {
                top: 0,
                right: 0,
                bottom: 0,
                left: 0,
            },
            size: Some((None, Some(40))),
            exclusive_zone: 40,
            size_limits: iced::Limits::NONE,
        })
    }
}
