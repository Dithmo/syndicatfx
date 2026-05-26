// Mirror of bftypes.h — primitive type aliases used throughout the codebase.

pub type Ubyte  = u8;
pub type Sbyte  = i8;
pub type Ushort = u16;
pub type Uint   = u32;
pub type Ulong  = u32;
pub type S32    = i32;
pub type U32    = u32;
pub type S64    = i64;
pub type U64    = u64;

pub type TbBool       = u8;
pub type TbResult     = i32;
pub type TbMemSize    = usize;
pub type TbScreenCoord = i16;
pub type TbPixel      = u8;
pub type TbClockMSec  = i64;
pub type TbTimeSec    = i64;

pub const LB_SUCCESS: TbResult =  0;
pub const LB_FAIL:    TbResult = -1;
pub const LB_OK:      TbResult =  1;

pub const TRUE:  TbBool = 1;
pub const FALSE: TbBool = 0;

// Screen modes (match bfscreen.h ScreenMode enum + legacy mode numbers)
pub const LB_SCREEN_MODE_320_200_8: u16 = 5;
pub const LB_SCREEN_MODE_640_480_8: u16 = 13;
pub const LB_MAX_SCREEN_MODES_COUNT: usize = 24;

// Screen mode video flags
pub const LB_VF_WINDOWED: u16 = 0x01;

// Sprite structure — mirrors TbSprite
#[repr(C)]
pub struct TbSprite {
    pub s_width:  u16,
    pub s_height: u16,
    pub data:     *mut u8,
}

// Load-file descriptor — mirrors TbLoadFiles
#[repr(C)]
pub struct TbLoadFiles {
    pub fname:  *const libc::c_char,
    pub pp:     *mut *mut libc::c_void,
    pub fsize:  *mut libc::c_long,
    pub flags:  u16,
    pub pad:    u16,
}

// Screen mode info — mirrors TbScreenModeInfo
#[repr(C, align(4))]
#[derive(Copy, Clone)]
pub struct TbScreenModeInfo {
    pub width:       u16,  // offset=0
    pub height:      u16,  // offset=2
    pub bits_per_pixel: u16, // offset=4
    pub video_mode:  u16,  // offset=6 (LB_VF_*)
    pub name:        [u8; 24],
}

impl TbScreenModeInfo {
    pub const fn zero() -> Self {
        TbScreenModeInfo {
            width: 0, height: 0, bits_per_pixel: 0, video_mode: 0, name: [0u8; 24],
        }
    }
}

// Display struct — mirrors TbDisplayStruct (offsets preserved)
#[repr(C)]
pub struct TbDisplayStruct {
    pub physical_screen:       *mut u8,   // offset=0
    pub w_screen:              *mut u8,   // offset=4
    pub glass_map:             *mut u8,   // offset=8
    pub fade_table:            *mut u8,   // offset=12
    pub graphics_window_ptr:   *mut u8,   // offset=16
    pub _pad20:                u32,       // offset=20
    pub physical_screen_width: i32,       // offset=24
    pub physical_screen_height:i32,       // offset=28
    pub graphics_screen_width: i32,       // offset=32
    pub graphics_screen_height:i32,       // offset=36
    pub graphics_window_x:     i32,       // offset=40
    pub graphics_window_y:     i32,       // offset=44
    pub graphics_window_width: i32,       // offset=48
    pub graphics_window_height:i32,       // offset=52
    pub mouse_window_x:        i32,       // offset=56
    pub mouse_window_y:        i32,       // offset=60
    pub mouse_window_width:    i32,       // offset=64
    pub mouse_window_height:   i32,       // offset=68
    pub mouse_x:               i32,       // offset=72
    pub mouse_y:               i32,       // offset=76
    pub mmouse_x:              i32,       // offset=80
    pub mmouse_y:              i32,       // offset=84
    pub rmouse_x:              i32,       // offset=88
    pub rmouse_y:              i32,       // offset=92
    pub draw_flags:            u16,       // offset=96
    pub old_video_mode:        u16,       // offset=98
    pub screen_mode:           u16,       // offset=100
    pub vesa_is_set_up:        u8,        // offset=102
    pub left_button:           u8,        // offset=103
    pub right_button:          u8,        // offset=104
    pub middle_button:         u8,        // offset=105
    pub mleft_button:          u8,        // offset=106
    pub mright_button:         u8,        // offset=107
    pub mmiddle_button:        u8,        // offset=108
    pub rleft_button:          u8,        // offset=109
    pub rmiddle_button:        u8,        // offset=110
    pub rright_button:         u8,        // offset=111
    pub fade_step:             u8,        // offset=112
    pub draw_colour:           u8,        // offset=113
    pub palette:               *mut u8,   // offset=114
    pub wheel_position:        i16,
    pub wheel_move_up:         u16,
    pub wheel_move_down:       u16,
    pub mouse_move_ratio_x:    i16,
    pub mouse_move_ratio_y:    i16,
}

impl TbDisplayStruct {
    pub const fn zero() -> Self {
        TbDisplayStruct {
            physical_screen: std::ptr::null_mut(),
            w_screen: std::ptr::null_mut(),
            glass_map: std::ptr::null_mut(),
            fade_table: std::ptr::null_mut(),
            graphics_window_ptr: std::ptr::null_mut(),
            _pad20: 0,
            physical_screen_width: 0,
            physical_screen_height: 0,
            graphics_screen_width: 0,
            graphics_screen_height: 0,
            graphics_window_x: 0,
            graphics_window_y: 0,
            graphics_window_width: 0,
            graphics_window_height: 0,
            mouse_window_x: 0,
            mouse_window_y: 0,
            mouse_window_width: 0,
            mouse_window_height: 0,
            mouse_x: 0,
            mouse_y: 0,
            mmouse_x: 0,
            mmouse_y: 0,
            rmouse_x: 0,
            rmouse_y: 0,
            draw_flags: 0,
            old_video_mode: 0,
            screen_mode: 0,
            vesa_is_set_up: 0,
            left_button: 0,
            right_button: 0,
            middle_button: 0,
            mleft_button: 0,
            mright_button: 0,
            mmiddle_button: 0,
            rleft_button: 0,
            rmiddle_button: 0,
            rright_button: 0,
            fade_step: 0,
            draw_colour: 0,
            palette: std::ptr::null_mut(),
            wheel_position: 0,
            wheel_move_up: 0,
            wheel_move_down: 0,
            mouse_move_ratio_x: 0x100,
            mouse_move_ratio_y: 0x100,
        }
    }
}
