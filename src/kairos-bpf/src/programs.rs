//! eBPF programs for kairos-bpf
pub mod anomaly;
pub mod execsnoop;
pub mod filemon;
pub mod oomkill;
pub mod schedlatency;
pub mod tcptop;

use crate::error::Result;

/// Load all eBPF programs
pub fn load_all() -> Result<()> {
    execsnoop::load()?;
    tcptop::load()?;
    filemon::load()?;
    anomaly::load()?;
    schedlatency::load()?;
    oomkill::load()?;
    Ok(())
}

/// Unload all eBPF programs
pub fn unload_all() -> Result<()> {
    execsnoop::unload()?;
    tcptop::unload()?;
    filemon::unload()?;
    anomaly::unload()?;
    schedlatency::unload()?;
    oomkill::unload()?;
    Ok(())
}
