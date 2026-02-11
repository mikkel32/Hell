use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Checks if the local system time is within a reasonable delta of atomic time.
/// Returns true if time is "Real", false if "Warped" or "Frozen".
/// Fails OPEN (returns true) if network is unreachable (to allow offline use).
pub fn reality_check() -> bool {
    let ntp_server = "pool.ntp.org:123";
    
    // 1. Setup UDP Socket (Timeout 2s to avoid hanging)
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return true, // Fail open
    };
    
    if socket.set_read_timeout(Some(Duration::from_secs(2))).is_err() {
        return true;
    }
    if socket.set_write_timeout(Some(Duration::from_secs(2))).is_err() {
        return true;
    }

    // 2. Construct NTP Packet (48 bytes)
    // LI=0, VN=3, Mode=3 (Client) -> 00 011 011 -> 0x1B
    let mut request = [0u8; 48];
    request[0] = 0x1B;

    // 3. Send
    if socket.send_to(&request, ntp_server).is_err() {
        return true; // Network likely down or blocked, assume innocent
    }

    // 4. Receive
    let mut response = [0u8; 48];
    if socket.recv_from(&mut response).is_err() {
        return true; // Timeout? Blocked? Fail open.
    }

    // 5. Parse Time (Transmit Timestamp at offset 40)
    // NTP time is seconds since 1900-01-01.
    // Bytes 40-43 are integer part (big endian).
    let ntp_seconds = ((response[40] as u64) << 24) |
                      ((response[41] as u64) << 16) |
                      ((response[42] as u64) << 8)  |
                      (response[43] as u64);

    // Convert to UNIX time (1970 epoch) -> Delta is 2,208,988,800 seconds
    const NTP_TO_UNIX_DELTA: u64 = 2_208_988_800;
    
    if ntp_seconds < NTP_TO_UNIX_DELTA {
        return true; // Invalid time?
    }
    
    let real_unix_time = ntp_seconds - NTP_TO_UNIX_DELTA;

    // 6. Compare with System Time
    let system_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => return false, // System time is before Epoch?! Definitely suspicious.
    };

    // 7. Verdict
    // If delta > 10 minutes (600s), something is wrong (Sandbox stuck in past/future).
    let delta = if real_unix_time > system_time {
        real_unix_time - system_time
    } else {
        system_time - real_unix_time
    };

    // We allow 10 minutes drift.
    if delta > 600 {
        return false; // WARPED REALITY DETECTED
    }

    true // Time flows normally here.
}
