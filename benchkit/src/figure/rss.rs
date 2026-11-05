//! Peak resident memory, where the platform reports it. On Linux it is
//! `VmHWM` of `/proc/self/status`, the process's high-water mark. Each
//! figure runs in a process of its own, and its mark is reset to its
//! resident set once its data is generated (by writing 5 to
//! `/proc/self/clear_refs`, Linux 4.0 and later). The peak then covers the
//! figure's stages with its data held. Where the reset is refused, it
//! covers the process's whole life. Elsewhere (Windows, macOS) there is no
//! peak, and the report shows "n/a".

use std::fs;

/// Resets the high-water mark to the current resident set. Returns false
/// where it cannot.
pub fn reset() -> bool {
    fs::write("/proc/self/clear_refs", "5").is_ok()
}

/// The high-water mark since the last [`reset`], in MiB, or `None` without
/// `/proc/self/status`.
pub fn peak_mb() -> Option<f64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    hwm_kb(&status).map(|kb| kb as f64 / 1024.0)
}

/// `VmHWM` of a `/proc/self/status`, kB.
fn hwm_kb(status: &str) -> Option<u64> {
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|rest| rest.split_whitespace().next()?.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::{hwm_kb, peak_mb, reset};

    #[test]
    fn the_peak_is_read_from_vmhwm_and_falls_after_a_reset() {
        let status = "Name:\tbench\nVmPeak:\t  9000 kB\nVmHWM:\t    2048 kB\nVmRSS:\t 1024 kB\n";
        assert_eq!(hwm_kb(status), Some(2048));
        assert_eq!(hwm_kb("Name:\tbench\n"), None);
        if !reset() {
            // No `/proc` (Windows, macOS), or a refused reset.
            return;
        }
        // 256 MiB touched, then freed: the mark holds it until a reset.
        let big = vec![1_u8; 256 << 20];
        assert!(big.iter().step_by(4096).all(|&b| b == 1));
        let high = peak_mb().unwrap();
        drop(big);
        assert!(reset());
        let low = peak_mb().unwrap();
        assert!(high - low > 200.0, "{high} → {low}");
    }
}
