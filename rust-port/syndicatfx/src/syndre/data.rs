// syndre/data.rs — Static data tables and globals from syndre.sx.
// Extends from the original sparse set to cover every EXPORT_SYMBOL reference
// found in the translated functions.

// ---- Music / mission state ------------------------------------------------
pub static mut DATA_5C354: i32 = 0;   // current music track request
pub static mut DATA_5C34C: i32 = 0;   // previous music state

// ---- Screen / update flags ------------------------------------------------
pub static mut DATA_5532C: u16 = 0;   // screen update flag
pub static mut DATA_5532E: u16 = 0;   // screen update flag 2
pub static mut DATA_60B4E: u8  = 0;   // command processing pause flag
pub static mut DATA_60B4F: u8  = 0;   // partial-redraw flag
pub static mut DATA_60B50: u8  = 0;   // scroll speed limiter

// ---- Game-loop state (byte_60AF* cluster) ---------------------------------
pub static mut DATA_60AF0: u32 = 0;
pub static mut DATA_60AF4: u8  = 0;
pub static mut DATA_60AF5: u8  = 0;
pub static mut DATA_60AF6: u8  = 0;
pub static mut DATA_60AF7: u8  = 0;
pub static mut DATA_60AF8: u8  = 0;
pub static mut DATA_60AF9: u8  = 0;
pub static mut DATA_60AFA: u8  = 0;
pub static mut DATA_60AFB: u8  = 0;
pub static mut DATA_60AFD: u8  = 0;
pub static mut DATA_60AFE: u32 = 0;
pub static mut A11:        u32 = 0;
pub static mut DATA_60B06: u32 = 0;
pub static mut DATA_60B32: u16 = 0;
pub static mut DATA_60AE8: u32 = 0;

// ---- Sound / music active flags -------------------------------------------
pub static mut SOUND_ACTIVE:  u8  = 1;
pub static mut MUSIC_ACTIVE:  u8  = 1;
pub static mut SCANNER_PULSE: u8  = 1;
pub static mut GAME_SPEED:    u32 = 7;
pub static mut RESEARCH:      u8  = 0;
pub static mut DATA_53EE8:    u8  = 0;  // last day-of-research

// ---- Per-player score / status arrays (data_60a7c, data_60674) -----------
pub static mut DATA_60A7C: [u32; 8]  = [0u32; 8];
pub static mut DATA_60674: [u8; 0x420] = [0u8; 0x420];

// ---- Map / tile data pointers --------------------------------------------
pub static mut DATA_55358: *mut u8   = std::ptr::null_mut(); // map block ptr
pub static mut DATA_5A510: [u8; 256] = [0u8; 256];          // collision type table
pub static mut DATA_5A8BC: *mut i16  = std::ptr::null_mut(); // drawlist scan start

// ---- Player network-slot data (data_605** group; stride=2) --------------
// Indexed by [slot], each slot stores u8/u16 fields packed at 2-byte stride
pub static mut DATA_605E0: [u8;  8] = [0u8;  8]; // player type
pub static mut DATA_605E1: [u8;  8] = [0u8;  8]; // player status
pub static mut DATA_605DE: [u16; 8] = [0u16; 8]; // packet counter
pub static mut DATA_605DA: [u16; 8] = [0u16; 8]; // seed copy
pub static mut DATA_605DC: [u16; 8] = [0u16; 8]; // person count
pub static mut DATA_605D6: [u16; 8] = [0u16; 8];
pub static mut DATA_605D8: [u16; 8] = [0u16; 8];
pub static mut PACKETS:    [u16; 8] = [0u16; 8]; // outbound packets

// ---- Player entity data (data_5e4** group; indexed by slot) --------------
pub static mut DATA_5E4A0: [u32; 8] = [0u32; 8]; // day counter
pub static mut DATA_5E4A4: [u16; 8] = [0u16; 8]; // days elapsed
pub static mut DATA_5E4A6: [u16; 8] = [0u16; 8]; // years elapsed
pub static mut DATA_5E4A8: [u16; 8] = [0u16; 8];
pub static mut DATA_5E4AA: [u8;  8] = [0u8;  8]; // player type (1=human, 2=cpu)
pub static mut DATA_5E4AB: [u8;  8] = [0u8;  8]; // colour
pub static mut DATA_5E4AC: [u8;  8] = [0u8;  8]; // slot index mirror
pub static mut DATA_5E4AD: [u8;  8] = [0u8;  8]; // flag
pub static mut DATA_5E4BF: [u8;  8] = [0u8;  8]; // flag2

// Entity count / type per slot (player → agent mapping into level__People)
// These are fields within a per-slot struct of stride 1047 bytes; index = slot * 1047.
// 8 slots × 1047 = 8376 bytes needed; use 8192 + padding → 8448 total.
pub static mut DATA_5E551: [u8;  8448] = [0u8;  8448]; // start agent index for slot
pub static mut DATA_5E552: [i8;  8448] = [0i8;  8448]; // signed adjustment

// Agent name / team data (same stride; fields at varying offsets within per-slot struct)
pub static mut DATA_5E555: [u8;  8448] = [0u8;  8448];
pub static mut DATA_5E587: [u8;  8448] = [0u8;  8448];
pub static mut DATA_5E5B9: [u8;  8448] = [0u8;  8448]; // name id
pub static mut DATA_5E5BA: [u8;  8448] = [0u8;  8448]; // agent flags (u16 via raw ptr)
pub static mut DATA_5E5BC: [u8;  8448] = [0u8;  8448]; // gender (u16 via raw ptr)
pub static mut DATA_5E5C0: [u8;  8448] = [0u8;  8448]; // team assignment

// ---- Weapon ammo caps (data at 0x5a73a; indexed by weapon type, u16 each) -
pub static WEAPON_MAX_AMMO: [u16; 10] = [
    0x0000, // type 0: none
    0x0032, // type 1: pistol      50
    0x000c, // type 2: shotgun     12
    0x0002, // type 3: uzi          2
    0x000b, // type 4: minigun     11
    0x0031, // type 5: laser       49
    0x01f3, // type 6: gauss      499
    0x0004, // type 7: flamer       4
    0x03e7, // type 8: long-range 999
    0x001d, // type 9: scanner     29
];

// ---- Fatal weapon rating table (data at 0x5a686; byte per weapon type 0-9) -
// Returned by fatal_weapon(); nonzero = weapon is considered "lethal" for AI.
pub static DATA_5A686: [u8; 10] = [0x00, 0x00, 0x01, 0x09, 0x02, 0x03, 0x08, 0x01, 0x07, 0x05];

// ---- Weapon detection range table (data at 0x5a6c2; u16 per weapon type) ---
// Indexed as DATA_5A6C2[weapon_type]; values are fixed-point distances (>> 8).
pub static DATA_5A6C2: [u16; 20] = [
    0x0000, // type  0: none
    0x0100, // type  1: pistol        256
    0x0500, // type  2: shotgun      1280
    0x1400, // type  3: uzi          5120
    0x0400, // type  4: minigun      1024
    0x0700, // type  5: laser        1792
    0x0b00, // type  6: gauss        2816
    0x1000, // type  7: flamer       4096
    0x0200, // type  8: long-range    512
    0x1800, // type  9: scanner      6144
    0x1000, // type 10:              4096
    0x0100, // type 11:               256
    0x03e8, // type 12:              1000
    0x0100, // type 13:               256
    0x0100, // type 14:               256
    0x0100, // type 15:               256
    0x0100, // type 16:               256
    0x0300, // type 17:               768
    0x0300, // type 18:               768
    0x0300, // type 19:               768
];

// ---- Player credits (EXPORT_SYMBOL(players)) ----------------------------
pub static mut PLAYERS: [u32; 8] = [0u32; 8];
pub static mut SELECTED_TEAM: [u8; 8] = [0u8; 8];

// ---- Map camera / scroll state -------------------------------------------
pub static mut PLAYER_VIEW_MAP_X: i16  = 0;
pub static mut PLAYER_VIEW_MAP_Y: i16  = 0;
pub static mut DESTINATION_X:     i32  = 0;
pub static mut DESTINATION_Y:     i32  = 0;
pub static mut LEVEL_HI_BOUNDARYX: i16 = 0;
pub static mut LEVEL_HI_BOUNDARYY: i16 = 0;
pub static mut LEVEL_LO_BOUNDARYX: i16 = 0;
pub static mut LEVEL_LO_BOUNDARYY: i16 = 0;

// ---- Objective / mission state -------------------------------------------
pub static mut LEVEL_OBJECTIVES: [u32; 20] = [0u32; 20];
pub static mut DATA_9BE3E: [u16; 20]        = [0u16; 20]; // objective types
pub static mut DATA_9BE40: [u16; 20]        = [0u16; 20]; // objective targets

// ---- Network -------------------------------------------------------------------
pub static mut NETWORK_NUMBER_OF_SLOTS: i16 = 1;

// ---- AI state ----------------------------------------------------------------
pub static mut COMP_PLYR_PROCESS_TIMER:     u32 = 0;
pub static mut COMP_PLYR_LAST_PROCESS_TIME: u32 = 0;
pub static mut COMP_PLYR_PROCESS_INTERVAL:  u8  = 1;
pub static mut COMP_PLYR_TEAM_SIZE:         u8  = 0;
pub static mut COMPUTER_PLAYERS_COUNT:      u8  = 0;

// ---- Animation / wind state -------------------------------------------------
pub static mut DATA_9BC72: i16 = 0;
pub static mut DATA_9BC74: u16 = 0;
pub static mut DATA_9BC76: u8  = 0x14;
pub static mut DATA_9BC77: u8  = 0x10;
pub static mut DATA_9BC78: u8  = 0;
pub static mut DATA_9BC79: u8  = 1;
pub static mut DATA_9BC7A: u8  = 0x28;
pub static mut DATA_9BC7B: u8  = 0x0a;
pub static mut LEVEL_WORLDS: i16 = 0; // level__Worlds

// ---- Level boundary (level__LoBoundaryy / HiBoundaryy etc.) ----------------
pub static mut LEVEL_LOBOUNDARYY: i16 = 0x12;
pub static mut LEVEL_HIBOUNDARYX: i16 = 0xca_u16 as i16;
pub static mut LEVEL_HIBOUNDARYY: i16 = 0xda_u16 as i16;

// ---- Level palette index table (level_palettes at 0x54170) -----------------
// Indexed by current level number (0-based); values 1-5 select the palette.
pub static LEVEL_PALETTES: [u8; 52] = [
    1,2,3,4,5, 1,2,3,4,5, 1,2,3,4,5, 1,2,3,4,5,
    1,2,3,4,5, 1,2,3,4,5, 1,2,3,4,5, 1,2,3,4,5,
    1,2,3,4,5, 1,2,3,4,5, 0,0,
];

// ---- Misc level init counter (data_60ac8) ----------------------------------
pub static mut DATA_60AC8: u32 = 0;

// ---- Sprite table slots adjacent to h_sprites (data_5531c/55320/55334) ----
// These are pointer slots in the sprite table block; filled during init.
pub static mut DATA_5531C: *mut u8 = std::ptr::null_mut();
pub static mut DATA_55320: *mut u8 = std::ptr::null_mut();
pub static mut DATA_55334: *mut u8 = std::ptr::null_mut();

// ---- Level entity pointers --------------------------------------------------
// Subarray pointers into the level data block; set by init_level_data.
// Strides: People=0x34, Vehicles=?, Objects=?, Weapons=0x24, Effects=0x1e.
pub static mut LEVEL_THINGS_BASE: *mut u8 = std::ptr::null_mut();
pub static mut LEVEL_PEOPLE:      *mut u8 = std::ptr::null_mut();
pub static mut LEVEL_VEHICLES:    *mut u8 = std::ptr::null_mut();
pub static mut LEVEL_OBJECTS:     *mut u8 = std::ptr::null_mut();
pub static mut LEVEL_WEAPONS:     *mut u8 = std::ptr::null_mut();
pub static mut LEVEL_EFFECTS:     *mut u8 = std::ptr::null_mut();
pub static mut LEVEL_COMMANDS:    *mut u8 = std::ptr::null_mut();
pub static mut LEVEL_MAPWHO:      *mut u8 = std::ptr::null_mut();
pub static mut LEVEL_TIMER:       u16 = 0;
pub static mut LEVEL_PERSON_COUNT: u16 = 0; // level__PersonCount_UNSURE

// ---- Thing draw list --------------------------------------------------------
pub static mut THING_DRAWLIST: [u8; 16 * 1024] = [0u8; 16 * 1024];

// ---- UI / panel state -------------------------------------------------------
pub static mut DATA_5E122: u16 = 0;
pub static mut DATA_5E124: u16 = 0;
pub static mut DATA_5E128: i16 = 0; // z-displacement added to DATA_60B2C in hug moves
pub static mut DATA_5E12C: u16 = 0; // set to 1 by USE_WEAPON to suppress which_frame_person
pub static mut DATA_5E12E: i16 = 0; // perception range arg for agent_check_arc_for_enemy
pub static mut DATA_5E110: u16 = 0;
pub static mut DATA_5E112: u16 = 0;
pub static mut DATA_5E11E: u16 = 0;
pub static mut A2:         u16 = 0;

// ---- Horizontal distance table (getrdist) -----------------------------------
pub static mut GETRDIST_TABLE: [u16; 512] = [0u16; 512];

// ---- Map drawing state (draw_mapwho internals) ------------------------------
pub static mut DATA_60B10: u16 = 0; // screen x counter
pub static mut DATA_60B0A: u16 = 0; // tile column counter
pub static mut DATA_60B1A: u16 = 0; // sort position
pub static mut DATA_60B12: u16 = 0; // scan line counter
pub static mut DATA_60B14: u16 = 0; // entity count
pub static mut DATA_60B18: u16 = 0; // click mode
pub static mut DATA_60B1C: u16 = 0; // click tile x
pub static mut DATA_60B1E: u16 = 0; // click tile y
pub static mut DATA_60B20: u16 = 0; // click world x
pub static mut DATA_60B0E: u16 = 0; // cursor type

// click_map intermediate state
pub static mut DATA_60AAC: u32 = 0; // map offset from click
pub static mut DATA_60AE0: u32 = 0;

// ---- Entity tail pointers (last_vehicle/person/object) ----------------------
pub static mut LAST_VEHICLE: *mut u8 = std::ptr::null_mut(); // 0x60adc
pub static mut LAST_PERSON:  *mut u8 = std::ptr::null_mut(); // 0x60ae0
pub static mut LAST_OBJECT:  *mut u8 = std::ptr::null_mut(); // 0x60aec

// ---- Movement accumulator state (affect_by_wind / goto_angle) ---------------
pub static mut DATA_60B28: i16 = 0; // x accumulator
pub static mut DATA_60B2A: i16 = 0; // y accumulator
pub static mut DATA_60B2C: i16 = 0; // z accumulator
pub static mut DATA_60B30: i16 = 0; // ammo display cache (scanner/HUD)

// ---- Country index (used in draw_panel / level_init) ------------------------
pub static mut DATA_60B36: u16 = 0;

// ---- Mission load list count ------------------------------------------------
pub static mut DATA_55300: u16 = 0;

// ---- 8.8 fixed-point trig tables (256 entries each, index = angle/256 turn) -
// data_5ab60: sin(i * 2π/256) * 256  as i16
pub static DATA_5AB60: [i16; 256] = [
      0,   6,  12,  18,  25,  31,  37,  43,  49,  56,  62,  68,  74,  80,  86,  92,
     97, 103, 109, 115, 120, 126, 131, 136, 142, 147, 152, 157, 162, 167, 171, 176,
    181, 185, 189, 193, 197, 201, 205, 209, 212, 216, 219, 222, 225, 228, 231, 234,
    236, 238, 241, 243, 244, 246, 248, 249, 251, 252, 253, 254, 254, 255, 255, 255,
    256, 255, 255, 255, 254, 254, 253, 252, 251, 249, 248, 246, 244, 243, 241, 238,
    236, 234, 231, 228, 225, 222, 219, 216, 212, 209, 205, 201, 197, 193, 189, 185,
    181, 176, 171, 167, 162, 157, 152, 147, 142, 136, 131, 126, 120, 115, 109, 103,
     97,  92,  86,  80,  74,  68,  62,  56,  49,  43,  37,  31,  25,  18,  12,   6,
      0,  -6, -12, -18, -25, -31, -37, -43, -49, -56, -62, -68, -74, -80, -86, -92,
    -97,-103,-109,-115,-120,-126,-131,-136,-142,-147,-152,-157,-162,-167,-171,-176,
   -181,-185,-189,-193,-197,-201,-205,-209,-212,-216,-219,-222,-225,-228,-231,-234,
   -236,-238,-241,-243,-244,-246,-248,-249,-251,-252,-253,-254,-254,-255,-255,-255,
   -256,-255,-255,-255,-254,-254,-253,-252,-251,-249,-248,-246,-244,-243,-241,-238,
   -236,-234,-231,-228,-225,-222,-219,-216,-212,-209,-205,-201,-197,-193,-189,-185,
   -181,-176,-171,-167,-162,-157,-152,-147,-142,-136,-131,-126,-120,-115,-109,-103,
    -97, -92, -86, -80, -74, -68, -62, -56, -49, -43, -37, -31, -25, -18, -12,  -6,
];

// data_5ad60: cos(i * 2π/256) * 256  as i16  (= sin table rotated by 64)
pub static DATA_5AD60: [i16; 256] = [
    256, 255, 255, 255, 254, 254, 253, 252, 251, 249, 248, 246, 244, 243, 241, 238,
    236, 234, 231, 228, 225, 222, 219, 216, 212, 209, 205, 201, 197, 193, 189, 185,
    181, 176, 171, 167, 162, 157, 152, 147, 142, 136, 131, 126, 120, 115, 109, 103,
     97,  92,  86,  80,  74,  68,  62,  56,  49,  43,  37,  31,  25,  18,  12,   6,
      0,  -6, -12, -18, -25, -31, -37, -43, -49, -56, -62, -68, -74, -80, -86, -92,
    -97,-103,-109,-115,-120,-126,-131,-136,-142,-147,-152,-157,-162,-167,-171,-176,
   -181,-185,-189,-193,-197,-201,-205,-209,-212,-216,-219,-222,-225,-228,-231,-234,
   -236,-238,-241,-243,-244,-246,-248,-249,-251,-252,-253,-254,-254,-255,-255,-255,
   -256,-255,-255,-255,-254,-254,-253,-252,-251,-249,-248,-246,-244,-243,-241,-238,
   -236,-234,-231,-228,-225,-222,-219,-216,-212,-209,-205,-201,-197,-193,-189,-185,
   -181,-176,-171,-167,-162,-157,-152,-147,-142,-136,-131,-126,-120,-115,-109,-103,
    -97, -92, -86, -80, -74, -68, -62, -56, -49, -43, -37, -31, -25, -18, -12,  -6,
      0,   6,  12,  18,  25,  31,  37,  43,  49,  56,  62,  68,  74,  80,  86,  92,
     97, 103, 109, 115, 120, 126, 131, 136, 142, 147, 152, 157, 162, 167, 171, 176,
    181, 185, 189, 193, 197, 201, 205, 209, 212, 216, 219, 222, 225, 228, 231, 234,
    236, 238, 241, 243, 244, 246, 248, 249, 251, 252, 253, 254, 254, 255, 255, 255,
];

// ---- Arctan lookup table (data_5a95e) ---------------------------------------
// 259 i16 entries. Index = (min_abs * 256 / max_abs) as u8.
// Value = arctan(index/256) in 256-unit-circle units (0-32 = 0-45°).
// First 7 entries are zero (from .fill 0xe), data starts at index 7.
pub static DATA_5A95E: [i16; 259] = [
    // indices 0-6: zeros from .fill 0xe
    0, 0, 0, 0, 0, 0, 0,
    // indices 7-138 (lines 84224-84256)
     1, 1, 1, 1,  1, 1, 2, 2,  2, 2, 2, 2,  3, 3, 3, 3,
     3, 3, 3, 4,  4, 4, 4, 4,  4, 5, 5, 5,  5, 5, 5, 6,
     6, 6, 6, 6,  6, 6, 7, 7,  7, 7, 7, 7,  8, 8, 8, 8,
     8, 8, 8, 9,  9, 9, 9, 9,  9, 9,10,10, 10,10,10,10,
    11,11,11,11, 11,11,11,12, 12,12,12,12, 12,12,13,13,
    13,13,13,13, 13,14,14,14, 14,14,14,14, 15,15,15,15,
    15,15,15,15, 16,16,16,16, 16,16,16,17, 17,17,17,17,
    17,17,17,17, 18,18,18,18, 18,18,18,18, 19,19,19,19,
    // indices 139-258 (lines 84257-84286)
    20,20,20,20, 20,20,20,20, 20,20,20,21, 21,21,21,21,
    21,21,21,22, 22,22,22,22, 22,22,22,22, 23,23,23,23,
    23,23,23,23, 23,24,24,24, 24,24,24,24, 24,24,24,25,
    25,25,25,25, 25,25,25,25, 26,26,26,26, 26,26,26,26,
    26,27,27,27, 27,27,27,27, 27,27,27,27, 28,28,28,28,
    28,28,28,28, 28,28,29,29, 29,29,29,29, 29,29,29,29,
    29,29,30,30, 30,30,30,30, 30,30,30,30, 30,31,31,31,
    31,31,31,31, 31,31,31,31, 31,31,31,32,
];

// ---- Load file arrays -------------------------------------------------------
// These are pointers to TbLoadFiles arrays that live in the data segment;
// they get filled in when load_files_*.sx is translated.
pub static mut SOUND_BANK_FILES0:         *mut u8 = std::ptr::null_mut();
pub static mut UNK1_EMPTY_LOAD_FILES:     *mut u8 = std::ptr::null_mut();
pub static mut UNK984_LOAD_FILES:         *mut u8 = std::ptr::null_mut();
pub static mut MISSION_LOAD_FILES:        *mut u8 = std::ptr::null_mut();
