//! Eventos acordam a reconciliação; a verificação periódica cobre eventos perdidos.
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

pub struct Changes {
    _watcher: RecommendedWatcher,
    receiver: Receiver<()>,
}
impl Changes {
    pub fn new(root: &Path) -> io::Result<Self> {
        let root = std::fs::canonicalize(root)?;
        let metadata = root.join(".crewsync");
        let (sender, receiver) = mpsc::sync_channel(1);
        let mut watcher = RecommendedWatcher::new(
            move |event: notify::Result<Event>| {
                let relevant = match event {
                    Ok(event) => {
                        !matches!(event.kind, EventKind::Access(_))
                            && event.paths.iter().any(|p| !p.starts_with(&metadata))
                    }
                    Err(_) => true, // overflow/erro exige reconciliação completa
                };
                if relevant {
                    let _ = sender.try_send(());
                }
            },
            Config::default().with_follow_symlinks(false),
        )
        .map_err(io::Error::other)?;
        watcher
            .watch(&root, RecursiveMode::Recursive)
            .map_err(io::Error::other)?;
        Ok(Self {
            _watcher: watcher,
            receiver,
        })
    }
    pub fn wait(&self, maximum: Duration) {
        if self.receiver.recv_timeout(maximum).is_ok() {
            // Janela finita: uma sequência contínua de eventos não adia o sync.
            std::thread::sleep(Duration::from_millis(200));
            while self.receiver.try_recv().is_ok() {}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_events_arrive_and_internal_metadata_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".crewsync")).unwrap();
        let changes = Changes::new(dir.path()).unwrap();
        std::fs::write(dir.path().join(".crewsync/state"), b"internal").unwrap();
        assert!(
            changes
                .receiver
                .recv_timeout(Duration::from_millis(100))
                .is_err()
        );
        std::fs::write(dir.path().join("file"), b"changed").unwrap();
        changes
            .receiver
            .recv_timeout(Duration::from_secs(3))
            .unwrap();
    }
}
