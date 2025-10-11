use std::collections::HashSet;

use iced::futures::{
    StreamExt,
    stream::{once, unfold},
};
use niri_ipc::{
    Action, Request, Response,
    socket::Socket,
    state::{EventStreamState, EventStreamStatePart},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixSocket, UnixStream},
};

use super::{Event, WorkspaceInfo};

pub struct NiriDesktop {
    socket: Socket,
}
impl NiriDesktop {
    pub fn new(socket: Socket) -> Self {
        Self { socket }
    }

    pub fn focus_workspace(&mut self, id: i64) {
        let response = self
            .socket
            .send(Request::Action(Action::FocusWorkspace {
                reference: niri_ipc::WorkspaceReferenceArg::Id(id as u64),
            }))
            .unwrap()
            .unwrap();
        assert!(matches!(response, Response::Handled));
    }

    pub fn subscription(&self) -> iced::Subscription<Event> {
        #[derive(Hash)]
        struct NiriEvents;

        iced::Subscription::run_with_id(
            NiriEvents,
            once(new_event_stream()).flat_map(|socket| {
                unfold(
                    (socket, String::new(), EventStreamState::default()),
                    |(mut socket, mut buf, mut state)| async {
                        let event = read_event(&mut buf, &mut socket).await?;
                        state.apply(event);

                        Some((
                            Event::WorkspacesChanged(make_workspace_infos(&state)),
                            (socket, buf, state),
                        ))
                    },
                )
            }),
        )
    }
}

async fn new_event_stream() -> BufReader<UnixStream> {
    let socket = UnixSocket::new_stream()
        .unwrap()
        .connect(std::env::var(niri_ipc::socket::SOCKET_PATH_ENV).unwrap())
        .await
        .unwrap();
    let mut socket = BufReader::new(socket);

    let mut buf = serde_json::to_string(&Request::EventStream).unwrap();
    buf.push('\n');
    socket.write_all(buf.as_bytes()).await.unwrap();
    buf.clear();

    socket.read_line(&mut buf).await.unwrap();
    let reply: niri_ipc::Reply = serde_json::from_str(&buf).unwrap();
    assert!(matches!(reply, Ok(Response::Handled)));

    socket
}

async fn read_event(
    buf: &mut String,
    socket: &mut BufReader<UnixStream>,
) -> Option<niri_ipc::Event> {
    buf.clear();
    socket.read_line(buf).await.ok()?;
    serde_json::from_str(buf).ok()
}

fn make_workspace_infos(state: &EventStreamState) -> Vec<WorkspaceInfo> {
    let nonempty_workspace_ids = state
        .windows
        .windows
        .values()
        .filter_map(|w| w.workspace_id)
        .collect::<HashSet<_>>();

    state
        .workspaces
        .workspaces
        .values()
        .map(|w| WorkspaceInfo {
            output: w.output.clone(),
            id: w.id as i64,
            idx: w.idx as i32,
            is_active: w.is_active,
            has_windows: nonempty_workspace_ids.contains(&w.id),
        })
        .collect()
}
