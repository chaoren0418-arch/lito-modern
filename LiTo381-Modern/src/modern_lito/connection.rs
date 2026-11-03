//! Non-UI connection lifecycle. At most two workers total, at most one active;
//! cancelling/reconnecting never joins a worker on the event-loop path.
//! TLS negotiation is NOT an account login. Gameplay/authentication remain gated.

use super::profile::{ModernProfile, Trust};
use super::protocol::{Ack, Hello};
use super::transport::{connect_and_negotiate, Cancellation, ModernTlsConnection, TransportLimits};
use std::fmt;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConnectionEpoch(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting(ConnectionEpoch),
    /// The server's certificate and protocol were validated, not the account.
    /// This records completion, not ongoing peer liveness: the persistent
    /// gameplay receiver/heartbeat is a separate, still unimplemented stage.
    Negotiated(ConnectionEpoch, Ack),
    Failed(ConnectionEpoch),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartError {
    RetiringWorkersBusy,
    EpochExhausted,
    WorkerSpawnFailed,
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "modern connection: {self:?}")
    }
}
impl std::error::Error for StartError {}

struct Attempt {
    epoch: ConnectionEpoch,
    cancel: Cancellation,
    receiver: Receiver<Result<ModernTlsConnection, String>>,
    worker: JoinHandle<()>,
}

/// Owns the transport and every thread. No static profile, socket, request ID,
/// certificate store, source ID, actor state or TW process is shared.
pub struct ConnectionController {
    next_epoch: u32,
    state: ConnectionState,
    active: Option<Attempt>,
    retiring: Vec<Attempt>,
    connection: Option<ModernTlsConnection>,
    context_key: Option<[u8; 32]>,
    error: Option<String>,
}

impl Default for ConnectionController {
    fn default() -> Self {
        Self {
            next_epoch: 1,
            state: ConnectionState::Disconnected,
            active: None,
            retiring: Vec::new(),
            connection: None,
            context_key: None,
            error: None,
        }
    }
}

impl ConnectionController {
    /// Called by the future game host, never by TW bootstrap/stage2.
    /// A busy result leaves the current connection/attempt unchanged.
    pub fn start(&mut self, profile: ModernProfile) -> Result<ConnectionEpoch, StartError> {
        self.reap_retiring();
        if self.retiring.len() + usize::from(self.active.is_some()) >= 2 {
            return Err(StartError::RetiringWorkersBusy);
        }
        let epoch = ConnectionEpoch(self.next_epoch);
        let next = self
            .next_epoch
            .checked_add(1)
            .ok_or(StartError::EpochExhausted)?;
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let context_key = profile.context_key();
        let (sender, receiver) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name(format!("lito-connect-{}", epoch.0))
            .spawn(move || {
                let root = match profile.trust() {
                    Trust::WebPki => None,
                    Trust::PrivateCa(der) => Some(der.as_slice()),
                };
                let limits = TransportLimits {
                    connect_timeout: profile.connect_timeout(),
                    total_timeout: profile.connect_timeout() + profile.exchange_timeout(),
                    ..TransportLimits::default()
                };
                let result = connect_and_negotiate(
                    profile.endpoint(),
                    profile.tls_server_name(),
                    root,
                    Hello::current(epoch.0),
                    limits,
                    worker_cancel,
                )
                .map_err(|error| error.to_string());
                // Capacity is exactly one and there is exactly one send. Dropping
                // the receiver discards/closes a late successful TLS connection.
                let _ = sender.send(result);
            })
            .map_err(|_| StartError::WorkerSpawnFailed)?;
        self.disconnect();
        self.next_epoch = next;
        self.context_key = Some(context_key);
        self.state = ConnectionState::Connecting(epoch);
        self.active = Some(Attempt {
            epoch,
            cancel,
            receiver,
            worker,
        });
        Ok(epoch)
    }

    /// Poll from a timer independently of focus/paint. Never blocks awaiting IO.
    pub fn poll(&mut self) {
        self.reap_retiring();
        let Some(attempt) = self.active.as_ref() else {
            return;
        };
        // The final result can arrive before thread exit. Avoid join until done.
        if !attempt.worker.is_finished() {
            return;
        }
        let attempt = self.active.take().unwrap();
        let result = attempt.receiver.try_recv();
        let joined = attempt.worker.join();
        if self.state != ConnectionState::Connecting(attempt.epoch) {
            // Only this exact attempt may publish. Any returned connection drops.
            return;
        }
        match (joined, result) {
            (Ok(()), Ok(Ok(connection))) => {
                if let Some(ack) = connection.ack() {
                    self.state = ConnectionState::Negotiated(attempt.epoch, ack);
                    self.connection = Some(connection);
                    self.error = None;
                } else {
                    self.fail(
                        attempt.epoch,
                        "connection closed before negotiation could be applied".to_owned(),
                    );
                }
            }
            (Ok(()), Ok(Err(error))) => self.fail(attempt.epoch, error),
            _ => self.fail(
                attempt.epoch,
                "connection worker ended without a result".to_owned(),
            ),
        }
    }

    fn fail(&mut self, epoch: ConnectionEpoch, error: String) {
        self.state = ConnectionState::Failed(epoch);
        self.error = Some(error);
        self.connection = None;
    }

    /// Immediate UI transition; cancelled IO retires in the background. No old
    /// source/profile/ACK is retained after disconnect, even while IO is ending.
    pub fn disconnect(&mut self) {
        if let Some(attempt) = self.active.take() {
            attempt.cancel.cancel();
            self.retiring.push(attempt);
        }
        self.connection = None;
        self.state = ConnectionState::Disconnected;
        self.context_key = None;
        self.error = None;
        self.reap_retiring();
    }

    fn reap_retiring(&mut self) {
        let mut index = 0;
        while index < self.retiring.len() {
            if self.retiring[index].worker.is_finished() {
                let attempt = self.retiring.swap_remove(index);
                let _ = attempt.worker.join();
                // Dropping receiver also drops a queued (now stale) connection.
            } else {
                index += 1;
            }
        }
    }

    pub fn state(&self) -> ConnectionState {
        self.state
    }
    pub fn context_key(&self) -> Option<[u8; 32]> {
        self.context_key
    }
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    /// After disconnect, keep polling before destroying the final host. True
    /// means no connect worker is still using resources; this is not a paint flag.
    pub fn is_quiescent(&self) -> bool {
        self.active.is_none() && self.retiring.is_empty()
    }
}

impl Drop for ConnectionController {
    fn drop(&mut self) {
        self.disconnect();
        // Last-resort application teardown owns all workers; never detach them.
        // Normal UI shutdown polls is_quiescent first so it does not join here.
        for attempt in self.retiring.drain(..) {
            let _ = attempt.worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn finish(worker: &JoinHandle<()>) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while !worker.is_finished() {
            assert!(
                Instant::now() < deadline,
                "controlled worker failed to finish"
            );
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn profile() -> ModernProfile {
        ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
            "schema":1, "profile_id":"unit-test", "display_name":"Synthetic unit test",
            "shared_resource_root": if cfg!(windows) { "D:\\synthetic-test" } else { "/synthetic-test" },
            "endpoint":"127.0.0.1:25400", "tls_server_name":"localhost",
            "trust":{"kind":"web_pki"}, "connect_timeout_ms":100, "exchange_timeout_ms":100
        })).unwrap()).unwrap()
    }

    fn controlled_attempt(epoch: u32) -> (Attempt, mpsc::SyncSender<()>) {
        let (release, wait_for_release) = mpsc::sync_channel(1);
        let (sender, receiver) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            // A failed assertion must not deadlock the controller's Drop.
            let _ = wait_for_release.recv_timeout(Duration::from_secs(2));
            let _ = sender.send(Err("old profile reply must be discarded".to_owned()));
        });
        (
            Attempt {
                epoch: ConnectionEpoch(epoch),
                cancel: Cancellation::default(),
                receiver,
                worker,
            },
            release,
        )
    }

    #[test]
    fn cancellation_clears_profile_and_ignores_late_worker_failure() {
        let (attempt, barrier) = controlled_attempt(7);
        let mut controller = ConnectionController::default();
        controller.state = ConnectionState::Connecting(ConnectionEpoch(7));
        controller.active = Some(attempt);
        controller.context_key = Some([9; 32]);
        controller.disconnect();
        assert_eq!(controller.state(), ConnectionState::Disconnected);
        assert!(controller.context_key().is_none());
        assert!(controller.error().is_none());
        assert!(!controller.is_quiescent());
        // Release the fake IO only after the nonblocking disconnect returned.
        barrier.send(()).unwrap();
        finish(&controller.retiring[0].worker);
        controller.poll();
        assert!(controller.is_quiescent());
        assert_eq!(controller.state(), ConnectionState::Disconnected);
        assert!(controller.error().is_none());
    }

    #[test]
    fn completed_active_error_has_its_own_epoch_and_never_becomes_authenticated() {
        let (attempt, barrier) = controlled_attempt(11);
        let mut controller = ConnectionController::default();
        controller.state = ConnectionState::Connecting(ConnectionEpoch(11));
        controller.active = Some(attempt);
        barrier.send(()).unwrap();
        finish(&controller.active.as_ref().unwrap().worker);
        controller.poll();
        assert_eq!(
            controller.state(),
            ConnectionState::Failed(ConnectionEpoch(11))
        );
        assert!(controller.is_quiescent());
        assert!(controller.connection.is_none());
    }

    #[test]
    fn two_retiring_attempts_cannot_spawn_a_third_or_replace_current_state() {
        let (first, first_barrier) = controlled_attempt(1);
        let (second, second_barrier) = controlled_attempt(2);
        let mut controller = ConnectionController::default();
        controller.retiring = vec![first, second];
        controller.next_epoch = 3;
        // Collect before assertions so the synthetic workers are always released.
        let outcome = controller.start(profile());
        first_barrier.send(()).unwrap();
        second_barrier.send(()).unwrap();
        assert_eq!(outcome, Err(StartError::RetiringWorkersBusy));
        assert_eq!(controller.next_epoch, 3);
        assert_eq!(controller.state(), ConnectionState::Disconnected);
    }

    #[test]
    fn epoch_never_wraps_and_reuses_an_old_request_id() {
        let mut controller = ConnectionController::default();
        controller.next_epoch = u32::MAX;
        assert_eq!(controller.start(profile()), Err(StartError::EpochExhausted));
        assert!(controller.is_quiescent());
        assert_eq!(controller.state(), ConnectionState::Disconnected);
    }

    #[test]
    fn one_active_and_one_retiring_preserve_active_state_when_busy() {
        let (retiring, release_retiring) = controlled_attempt(4);
        let (active, release_active) = controlled_attempt(5);
        let mut controller = ConnectionController::default();
        controller.retiring.push(retiring);
        controller.active = Some(active);
        controller.state = ConnectionState::Connecting(ConnectionEpoch(5));
        controller.context_key = Some([17; 32]);
        controller.next_epoch = 6;
        let result = controller.start(profile());
        release_active.send(()).unwrap();
        release_retiring.send(()).unwrap();
        assert_eq!(result, Err(StartError::RetiringWorkersBusy));
        assert_eq!(
            controller.state(),
            ConnectionState::Connecting(ConnectionEpoch(5))
        );
        assert_eq!(controller.context_key(), Some([17; 32]));
        assert_eq!(controller.next_epoch, 6);
    }
}
