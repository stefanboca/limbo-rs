use std::time::Duration;

use iced::{
    Border, Color,
    widget::{Row, container, mouse_area, text},
};

use crate::{
    components::section,
    desktop_environment::{MonitorInfo, WorkspaceId, WorkspaceInfo, focus_workspace},
};

mod state;
use state::WorkspaceState;

pub struct Workspaces {
    workspaces: Vec<WorkspaceState>,
    workspace_infos: Vec<WorkspaceInfo>,
    active_workspace_id: i32,
}

#[derive(Debug, Clone)]
pub enum WorkspacesMessage {
    Tick,
    DesktopEvent(MonitorInfo),
    FocusWorkspace(WorkspaceId),
}

impl Workspaces {
    pub fn new(workspaces: Vec<WorkspaceId>) -> Self {
        Self {
            workspaces: workspaces
                .iter()
                .map(|id| WorkspaceState::new(*id, false))
                .collect(),
            workspace_infos: Vec::new(),
            active_workspace_id: -1,
        }
    }

    pub fn update(&mut self, message: WorkspacesMessage) {
        match message {
            WorkspacesMessage::Tick => {
                for w in &mut self.workspaces {
                    w.update();
                }
            }
            WorkspacesMessage::DesktopEvent(monitor_info) => {
                self.workspace_infos = monitor_info.workspaces;
                self.active_workspace_id = monitor_info.active_workspace_id;
                for w in &mut self.workspaces {
                    w.set_active(w.id == self.active_workspace_id);
                }
            }
            WorkspacesMessage::FocusWorkspace(workspace_id) => focus_workspace(workspace_id),
        }
    }

    pub fn view(&self) -> iced::Element<'_, WorkspacesMessage> {
        let workspace_icons = self
            .workspaces
            .iter()
            .map(|w| {
                let info = self.workspace_infos.iter().find(|i| i.id == w.id);
                let has_windows = info.map(|w| w.has_windows).unwrap_or_default();
                let color = if has_windows || w.active {
                    Color::from_rgb8(137, 180, 250)
                } else {
                    Color::from_rgb8(88, 91, 112)
                };
                let width = (w.current_width_multiplier() * 5.).floor() as u16;

                mouse_area(
                    container(container(text("")).padding([5, width]).style(
                        move |_: &iced::Theme| {
                            container::Style {
                                background: Some(color.into()),
                                border: Border {
                                    radius: 20.0.into(), // High radius for pill shape
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        },
                    ))
                    .padding([8, 15 - width]),
                )
                .on_press(WorkspacesMessage::FocusWorkspace(w.id))
                .into()
            })
            .collect::<Vec<_>>();

        section(Row::from_vec(workspace_icons))
            .padding([0, 8])
            .into()
    }

    pub fn subscription(&self) -> Option<iced::Subscription<WorkspacesMessage>> {
        self.workspaces
            .iter()
            .any(|w| w.needs_update())
            .then(|| iced::time::every(Duration::from_millis(8)).map(|_| WorkspacesMessage::Tick))
    }
}
