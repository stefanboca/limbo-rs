use hyprland::{
    data::{Monitors, Workspaces},
    dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial},
    error::HyprError,
    event_listener::{Event as HyprEvent, EventStream},
    shared::HyprData,
};
use iced::futures::{StreamExt, stream::once};

use super::{Event, WorkspaceInfo};

pub struct HyprlandDesktop;
impl HyprlandDesktop {
    pub fn new() -> Self {
        HyprlandDesktop
    }

    pub fn focus_workspace(&mut self, id: i64) {
        Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
            id as i32,
        )))
        .unwrap();
    }

    pub fn subscription(&self) -> iced::Subscription<Event> {
        #[derive(Hash)]
        struct NiriEvents;

        iced::Subscription::run_with_id(
            NiriEvents,
            once(async { make_workspace_infos().await.map(Event::WorkspacesChanged) })
                .filter_map(|e| async { e })
                .chain(EventStream::new().filter_map(process_event)),
        )
    }
}

async fn process_event(event: Result<HyprEvent, HyprError>) -> Option<Event> {
    let Ok(event) = event else {
        return None;
    };
    use HyprEvent::*;
    match event {
        MonitorAdded(_) | MonitorRemoved(_) | WorkspaceChanged(_) | WorkspaceDeleted(_)
        | WorkspaceAdded(_) | WorkspaceMoved(_) | WindowOpened(_) | WindowClosed(_)
        | WindowMoved(_) => make_workspace_infos().await.map(Event::WorkspacesChanged),
        _ => None,
    }
}

async fn make_workspace_infos() -> Option<Vec<WorkspaceInfo>> {
    let monitors = Monitors::get_async()
        .await
        .ok()?
        .into_iter()
        .collect::<Vec<_>>();

    Some(
        Workspaces::get_async()
            .await
            .ok()?
            .into_iter()
            .map(|w| WorkspaceInfo {
                output: Some(w.monitor),
                id: w.id as i64,
                idx: w.id,
                is_active: monitors.iter().any(|m| m.active_workspace.id == w.id),
                has_windows: w.windows > 0,
            })
            .collect(),
    )
}
