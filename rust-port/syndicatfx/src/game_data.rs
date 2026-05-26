// game_data.rs — mirrors game_data.h structures and path helpers.

use std::env;

// CPObjective — mission objective node
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct CPObjective {
    pub child:        i16,
    pub parent:       u16,
    pub field_4:      i16,
    pub field_6:      u8,
    pub action_type:  u8,
    pub action:       u8,
    pub target_offs:  u16,
    pub field_b:      i16,
    pub field_d:      i16,
}

impl CPObjective {
    pub const fn zero() -> Self {
        CPObjective {
            child: 0, parent: 0, field_4: 0, field_6: 0,
            action_type: 0, action: 0, target_offs: 0,
            field_b: 0, field_d: 0,
        }
    }
}

// TbLoadFiles descriptor (also in bflibrary types but redeclared here for game use)
#[repr(C)]
pub struct TbLoadFiles {
    pub fname: *const libc::c_char,
    pub pp:    *mut *mut libc::c_void,
    pub fsize: *mut libc::c_long,
    pub flags: u16,
    pub pad:   u16,
}

// ---- Directory helpers (mirrors GetDirectoryUser / GetDirectoryHdd) --------

static mut USER_DIR: [u8; 512] = [0u8; 512];
static mut DATA_DIR: [u8; 512] = [0u8; 512];

pub fn GetDirectoryUser() -> *const libc::c_char {
    unsafe {
        if USER_DIR[0] == 0 {
            let path = get_user_dir();
            let bytes = path.as_bytes();
            let len = bytes.len().min(USER_DIR.len() - 1);
            USER_DIR[..len].copy_from_slice(&bytes[..len]);
            USER_DIR[len] = 0;
        }
        USER_DIR.as_ptr() as *const libc::c_char
    }
}

pub fn GetDirectoryHdd() -> *const libc::c_char {
    unsafe {
        if DATA_DIR[0] == 0 {
            let path = get_data_dir();
            let bytes = path.as_bytes();
            let len = bytes.len().min(DATA_DIR.len() - 1);
            DATA_DIR[..len].copy_from_slice(&bytes[..len]);
            DATA_DIR[len] = 0;
        }
        DATA_DIR.as_ptr() as *const libc::c_char
    }
}

fn get_user_dir() -> String {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return format!("{}/syndicatfx", xdg);
    }
    if let Ok(home) = env::var("HOME") {
        return format!("{}/.config/syndicatfx", home);
    }
    ".".to_string()
}

fn get_data_dir() -> String {
    // Default install path matches the autoconf GAME_DATA_PATH
    option_env!("GAME_DATA_PATH").unwrap_or("/usr/share/syndicatfx").to_string()
}

pub fn setup_file_names() {
    // Force initialisation of the directory paths on startup.
    GetDirectoryUser();
    GetDirectoryHdd();
}

pub fn free_map_level() {
    // Calls into ASM ASM_free_map_level() which will be in syndre.rs.
    unsafe { crate::syndre::ASM_free_map_level(); }
}
