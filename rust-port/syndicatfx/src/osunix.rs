// osunix.rs — mirrors osunix.c: Unix signal and path helpers.

#[cfg(unix)]
pub fn unix_restore_signal_handlers() {
    unsafe {
        libc::signal(libc::SIGINT,  libc::SIG_DFL);
        libc::signal(libc::SIGTERM, libc::SIG_DFL);
    }
}

pub fn sys_get_user_path(buf: &mut [u8]) -> bool {
    let path = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        format!("{}/syndicatfx", xdg)
    } else if let Ok(home) = std::env::var("HOME") {
        format!("{}/.config/syndicatfx", home)
    } else {
        return false;
    };
    let bytes = path.as_bytes();
    let len = bytes.len().min(buf.len() - 1);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf[len] = 0;
    true
}
