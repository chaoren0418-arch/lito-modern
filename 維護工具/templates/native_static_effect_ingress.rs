/// Local decode completion, sampled before event-channel backpressure.
/// This is not server time, kernel arrival time or the original PE's clock.
/// The private constructor is used by the native network worker only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IngressReceipt {
    connection: u64,
    counter: u64,
    decoded_at: Instant,
}
impl IngressReceipt {
    fn decoded(connection: u64, counter: u64) -> Self {
        Self { connection, counter, decoded_at: Instant::now() }
    }
    pub fn connection_epoch(self) -> u64 { self.connection }
    pub fn rx_counter(self) -> u64 { self.counter }
    pub fn decoded_at(self) -> Instant { self.decoded_at }
}
