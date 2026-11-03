//! Common 4E8750 modifiers fed by S_SPEED and S_EMOTION receivers.
use anyhow::{ensure, Result};
pub fn factor(speed: u8, brave: u8, action: u8) -> Result<(u64, u64)> {
    let (n, d) = match speed {
        0 => (1, 1),
        1 => (3, 4),
        2 => (2, 1),
        _ => anyhow::bail!("unverified speed mode"),
    };
    let walk = matches!(action, 0 | 4 | 11 | 20 | 24 | 40 | 46 | 50 | 54 | 58 | 62);
    let (bn, bd) = match brave {
        0 | 6 => (1, 1),
        1 => (3, 4),
        3 => (7, 8),
        4 => {
            if walk {
                (3, 4)
            } else {
                (1, 1)
            }
        }
        5 => (3, 8),
        _ => anyhow::bail!("unverified brave mode"),
    };
    Ok((n * bn, d * bd))
}
pub fn advance(wall_ns: u64, speed: u8, brave: u8, action: u8) -> Result<u64> {
    let (n, d) = factor(speed, brave, action)?;
    Ok(u64::try_from(
        u128::from(wall_ns) * u128::from(d) / u128::from(n),
    )?)
}
pub fn duration(base_ns: u64, speed: u8, brave: u8, action: u8) -> Result<u64> {
    let (n, d) = factor(speed, brave, action)?;
    ensure!(base_ns > 0, "zero movement duration");
    Ok(u64::try_from(
        (u128::from(base_ns) * u128::from(n)).div_ceil(u128::from(d)),
    )?)
}
