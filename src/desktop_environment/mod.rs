#[cfg(feature = "hyprland")]
mod hyprland_desktop;
#[cfg(feature = "niri")]
mod niri_desktop;

#[cfg(not(any(feature = "hyprland", feature = "niri")))]
compile_error!("At least one of \"hyprland\" or \"niri\" must be enabled.");

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub output: Option<String>,
    pub id: i64,
    pub idx: i32,
    pub is_active: bool,
    pub has_windows: bool,
}

#[derive(Debug, Clone)]
pub enum Event {
    WorkspacesChanged(Vec<WorkspaceInfo>),
}

pub enum Desktop {
    #[cfg(feature = "hyprland")]
    Hyprland(hyprland_desktop::HyprlandDesktop),
    #[cfg(feature = "niri")]
    Niri(niri_desktop::NiriDesktop),
}
impl Desktop {
    pub fn new() -> Self {
        #[cfg(feature = "hyprland")]
        {
            use hyprland::shared::HyprData;
            if hyprland::data::Version::get().is_ok() {
                return Self::Hyprland(hyprland_desktop::HyprlandDesktop::new());
            }
        }

        #[cfg(feature = "niri")]
        if let Ok(socket) = niri_ipc::socket::Socket::connect() {
            return Self::Niri(niri_desktop::NiriDesktop::new(socket));
        }

        panic!("no compatible desktop environment detected");
    }

    pub fn focus_workspace(&mut self, id: i64) {
        match self {
            Desktop::Hyprland(hyprland_desktop) => hyprland_desktop.focus_workspace(id),
            Desktop::Niri(niri_desktop) => niri_desktop.focus_workspace(id),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Event> {
        match self {
            Desktop::Hyprland(hyprland_desktop) => hyprland_desktop.subscription(),
            Desktop::Niri(niri_desktop) => niri_desktop.subscription(),
        }
    }
}
