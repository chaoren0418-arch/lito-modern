//! Synthetic encrypted TCP fault matrix; this target requires teleport-wire-test.
use lito_modern::modern_lito::native_login::teleport_wire_tests as cases;

#[test]
fn coalesced_old_socket_burst_cannot_retarget_or_repeat_quit() {
    cases::coalesced_old_socket_burst_cannot_retarget_or_repeat_quit();
}

#[test]
fn retired_endpoint_duplicate_and_old_world_packets_do_not_change_quit_barrier() { cases::retired_endpoint_duplicate_and_old_world_packets_do_not_change_quit_barrier(); }

#[test]
fn malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner() { cases::malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner(); }

#[test]
fn new_destination_does_not_accept_previous_endpoint_as_a_new_handoff() { cases::new_destination_does_not_accept_previous_endpoint_as_a_new_handoff(); }

#[test]
fn cancel_discards_old_inbound_burst_before_fresh_login() { cases::cancel_discards_old_inbound_burst_before_fresh_login(); }

#[test]
fn invalid_transport_envelopes_during_teleport_retire_scope() { cases::invalid_transport_envelopes_during_teleport_retire_scope(); }

