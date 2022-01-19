use std::time::SystemTime;

use num_derive::{FromPrimitive, ToPrimitive};

use crate::errors::Result;

/// InputEvent is a qsk-specific struct modeled in large part after evdev_rs::InputEvent.
/// Although evdev_rs::InputEvent actually supports a large range of Linux-specific input events,
/// we focus here on keyboard and synchronization events specifically since keyboard events are the
/// primary concern of qsk initially and synchronization needs to be represented. Abstracting away
/// from Linux-specific event handling in this way will enable us to support input event systems
/// for other OSes in the future.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InputEvent {
    pub time: SystemTime,
    pub code: EventCode,
    pub state: KeyState,
}

#[derive(FromPrimitive, ToPrimitive, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum KeyState {
    Up = 0,
    Down = 1,
    Held = 2,
    NotImplemented = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventCode {
    KeyCode(KeyCode),
    SynCode(SynCode),
}

/// Copied and pasted from evdev-rs 0.3.1 with s/KEY_/KC_/ to align more closely with QMK naming
/// key code naming conventions.
#[allow(non_camel_case_types)]
#[derive(FromPrimitive, ToPrimitive, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum KeyCode {
    KC_RESERVED = 0,
    KC_ESC = 1,
    KC_1 = 2,
    KC_2 = 3,
    KC_3 = 4,
    KC_4 = 5,
    KC_5 = 6,
    KC_6 = 7,
    KC_7 = 8,
    KC_8 = 9,
    KC_9 = 10,
    KC_0 = 11,
    KC_MINUS = 12,
    KC_EQUAL = 13,
    KC_BACKSPACE = 14,
    KC_TAB = 15,
    KC_Q = 16,
    KC_W = 17,
    KC_E = 18,
    KC_R = 19,
    KC_T = 20,
    KC_Y = 21,
    KC_U = 22,
    KC_I = 23,
    KC_O = 24,
    KC_P = 25,
    KC_LEFTBRACE = 26,
    KC_RIGHTBRACE = 27,
    KC_ENTER = 28,
    KC_LEFTCTRL = 29,
    KC_A = 30,
    KC_S = 31,
    KC_D = 32,
    KC_F = 33,
    KC_G = 34,
    KC_H = 35,
    KC_J = 36,
    KC_K = 37,
    KC_L = 38,
    KC_SEMICOLON = 39,
    KC_APOSTROPHE = 40,
    KC_GRAVE = 41,
    KC_LEFTSHIFT = 42,
    KC_BACKSLASH = 43,
    KC_Z = 44,
    KC_X = 45,
    KC_C = 46,
    KC_V = 47,
    KC_B = 48,
    KC_N = 49,
    KC_M = 50,
    KC_COMMA = 51,
    KC_DOT = 52,
    KC_SLASH = 53,
    KC_RIGHTSHIFT = 54,
    KC_KPASTERISK = 55,
    KC_LEFTALT = 56,
    KC_SPACE = 57,
    KC_CAPSLOCK = 58,
    KC_F1 = 59,
    KC_F2 = 60,
    KC_F3 = 61,
    KC_F4 = 62,
    KC_F5 = 63,
    KC_F6 = 64,
    KC_F7 = 65,
    KC_F8 = 66,
    KC_F9 = 67,
    KC_F10 = 68,
    KC_NUMLOCK = 69,
    KC_SCROLLLOCK = 70,
    KC_KP7 = 71,
    KC_KP8 = 72,
    KC_KP9 = 73,
    KC_KPMINUS = 74,
    KC_KP4 = 75,
    KC_KP5 = 76,
    KC_KP6 = 77,
    KC_KPPLUS = 78,
    KC_KP1 = 79,
    KC_KP2 = 80,
    KC_KP3 = 81,
    KC_KP0 = 82,
    KC_KPDOT = 83,
    KC_ZENKAKUHANKAKU = 85,
    KC_102ND = 86,
    KC_F11 = 87,
    KC_F12 = 88,
    KC_RO = 89,
    KC_KATAKANA = 90,
    KC_HIRAGANA = 91,
    KC_HENKAN = 92,
    KC_KATAKANAHIRAGANA = 93,
    KC_MUHENKAN = 94,
    KC_KPJPCOMMA = 95,
    KC_KPENTER = 96,
    KC_RIGHTCTRL = 97,
    KC_KPSLASH = 98,
    KC_SYSRQ = 99,
    KC_RIGHTALT = 100,
    KC_LINEFEED = 101,
    KC_HOME = 102,
    KC_UP = 103,
    KC_PAGEUP = 104,
    KC_LEFT = 105,
    KC_RIGHT = 106,
    KC_END = 107,
    KC_DOWN = 108,
    KC_PAGEDOWN = 109,
    KC_INSERT = 110,
    KC_DELETE = 111,
    KC_MACRO = 112,
    KC_MUTE = 113,
    KC_VOLUMEDOWN = 114,
    KC_VOLUMEUP = 115,
    KC_POWER = 116,
    KC_KPEQUAL = 117,
    KC_KPPLUSMINUS = 118,
    KC_PAUSE = 119,
    KC_SCALE = 120,
    KC_KPCOMMA = 121,
    KC_HANGEUL = 122,
    KC_HANJA = 123,
    KC_YEN = 124,
    KC_LEFTMETA = 125,
    KC_RIGHTMETA = 126,
    KC_COMPOSE = 127,
    KC_STOP = 128,
    KC_AGAIN = 129,
    KC_PROPS = 130,
    KC_UNDO = 131,
    KC_FRONT = 132,
    KC_COPY = 133,
    KC_OPEN = 134,
    KC_PASTE = 135,
    KC_FIND = 136,
    KC_CUT = 137,
    KC_HELP = 138,
    KC_MENU = 139,
    KC_CALC = 140,
    KC_SETUP = 141,
    KC_SLEEP = 142,
    KC_WAKEUP = 143,
    KC_FILE = 144,
    KC_SENDFILE = 145,
    KC_DELETEFILE = 146,
    KC_XFER = 147,
    KC_PROG1 = 148,
    KC_PROG2 = 149,
    KC_WWW = 150,
    KC_MSDOS = 151,
    KC_COFFEE = 152,
    KC_ROTATE_DISPLAY = 153,
    KC_CYCLEWINDOWS = 154,
    KC_MAIL = 155,
    KC_BOOKMARKS = 156,
    KC_COMPUTER = 157,
    KC_BACK = 158,
    KC_FORWARD = 159,
    KC_CLOSECD = 160,
    KC_EJECTCD = 161,
    KC_EJECTCLOSECD = 162,
    KC_NEXTSONG = 163,
    KC_PLAYPAUSE = 164,
    KC_PREVIOUSSONG = 165,
    KC_STOPCD = 166,
    KC_RECORD = 167,
    KC_REWIND = 168,
    KC_PHONE = 169,
    KC_ISO = 170,
    KC_CONFIG = 171,
    KC_HOMEPAGE = 172,
    KC_REFRESH = 173,
    KC_EXIT = 174,
    KC_MOVE = 175,
    KC_EDIT = 176,
    KC_SCROLLUP = 177,
    KC_SCROLLDOWN = 178,
    KC_KPLEFTPAREN = 179,
    KC_KPRIGHTPAREN = 180,
    KC_NEW = 181,
    KC_REDO = 182,
    KC_F13 = 183,
    KC_F14 = 184,
    KC_F15 = 185,
    KC_F16 = 186,
    KC_F17 = 187,
    KC_F18 = 188,
    KC_F19 = 189,
    KC_F20 = 190,
    KC_F21 = 191,
    KC_F22 = 192,
    KC_F23 = 193,
    KC_F24 = 194,
    KC_PLAYCD = 200,
    KC_PAUSECD = 201,
    KC_PROG3 = 202,
    KC_PROG4 = 203,
    KC_DASHBOARD = 204,
    KC_SUSPEND = 205,
    KC_CLOSE = 206,
    KC_PLAY = 207,
    KC_FASTFORWARD = 208,
    KC_BASSBOOST = 209,
    KC_PRINT = 210,
    KC_HP = 211,
    KC_CAMERA = 212,
    KC_SOUND = 213,
    KC_QUESTION = 214,
    KC_EMAIL = 215,
    KC_CHAT = 216,
    KC_SEARCH = 217,
    KC_CONNECT = 218,
    KC_FINANCE = 219,
    KC_SPORT = 220,
    KC_SHOP = 221,
    KC_ALTERASE = 222,
    KC_CANCEL = 223,
    KC_BRIGHTNESSDOWN = 224,
    KC_BRIGHTNESSUP = 225,
    KC_MEDIA = 226,
    KC_SWITCHVIDEOMODE = 227,
    KC_KBDILLUMTOGGLE = 228,
    KC_KBDILLUMDOWN = 229,
    KC_KBDILLUMUP = 230,
    KC_SEND = 231,
    KC_REPLY = 232,
    KC_FORWARDMAIL = 233,
    KC_SAVE = 234,
    KC_DOCUMENTS = 235,
    KC_BATTERY = 236,
    KC_BLUETOOTH = 237,
    KC_WLAN = 238,
    KC_UWB = 239,
    KC_UNKNOWN = 240,
    KC_VIDEO_NEXT = 241,
    KC_VIDEO_PREV = 242,
    KC_BRIGHTNESS_CYCLE = 243,
    KC_BRIGHTNESS_AUTO = 244,
    KC_DISPLAY_OFF = 245,
    KC_WWAN = 246,
    KC_RFKILL = 247,
    KC_MICMUTE = 248,
    KC_OK = 352,
    KC_SELECT = 353,
    KC_GOTO = 354,
    KC_CLEAR = 355,
    KC_POWER2 = 356,
    KC_OPTION = 357,
    KC_INFO = 358,
    KC_TIME = 359,
    KC_VENDOR = 360,
    KC_ARCHIVE = 361,
    KC_PROGRAM = 362,
    KC_CHANNEL = 363,
    KC_FAVORITES = 364,
    KC_EPG = 365,
    KC_PVR = 366,
    KC_MHP = 367,
    KC_LANGUAGE = 368,
    KC_TITLE = 369,
    KC_SUBTITLE = 370,
    KC_ANGLE = 371,
    KC_FULL_SCREEN = 372,
    KC_MODE = 373,
    KC_KEYBOARD = 374,
    KC_ASPECT_RATIO = 375,
    KC_PC = 376,
    KC_TV = 377,
    KC_TV2 = 378,
    KC_VCR = 379,
    KC_VCR2 = 380,
    KC_SAT = 381,
    KC_SAT2 = 382,
    KC_CD = 383,
    KC_TAPE = 384,
    KC_RADIO = 385,
    KC_TUNER = 386,
    KC_PLAYER = 387,
    KC_TEXT = 388,
    KC_DVD = 389,
    KC_AUX = 390,
    KC_MP3 = 391,
    KC_AUDIO = 392,
    KC_VIDEO = 393,
    KC_DIRECTORY = 394,
    KC_LIST = 395,
    KC_MEMO = 396,
    KC_CALENDAR = 397,
    KC_RED = 398,
    KC_GREEN = 399,
    KC_YELLOW = 400,
    KC_BLUE = 401,
    KC_CHANNELUP = 402,
    KC_CHANNELDOWN = 403,
    KC_FIRST = 404,
    KC_LAST = 405,
    KC_AB = 406,
    KC_NEXT = 407,
    KC_RESTART = 408,
    KC_SLOW = 409,
    KC_SHUFFLE = 410,
    KC_BREAK = 411,
    KC_PREVIOUS = 412,
    KC_DIGITS = 413,
    KC_TEEN = 414,
    KC_TWEN = 415,
    KC_VIDEOPHONE = 416,
    KC_GAMES = 417,
    KC_ZOOMIN = 418,
    KC_ZOOMOUT = 419,
    KC_ZOOMRESET = 420,
    KC_WORDPROCESSOR = 421,
    KC_EDITOR = 422,
    KC_SPREADSHEET = 423,
    KC_GRAPHICSEDITOR = 424,
    KC_PRESENTATION = 425,
    KC_DATABASE = 426,
    KC_NEWS = 427,
    KC_VOICEMAIL = 428,
    KC_ADDRESSBOOK = 429,
    KC_MESSENGER = 430,
    KC_DISPLAYTOGGLE = 431,
    KC_SPELLCHECK = 432,
    KC_LOGOFF = 433,
    KC_DOLLAR = 434,
    KC_EURO = 435,
    KC_FRAMEBACK = 436,
    KC_FRAMEFORWARD = 437,
    KC_CONTEXT_MENU = 438,
    KC_MEDIA_REPEAT = 439,
    KC_10CHANNELSUP = 440,
    KC_10CHANNELSDOWN = 441,
    KC_IMAGES = 442,
    KC_DEL_EOL = 448,
    KC_DEL_EOS = 449,
    KC_INS_LINE = 450,
    KC_DEL_LINE = 451,
    KC_FN = 464,
    KC_FN_ESC = 465,
    KC_FN_F1 = 466,
    KC_FN_F2 = 467,
    KC_FN_F3 = 468,
    KC_FN_F4 = 469,
    KC_FN_F5 = 470,
    KC_FN_F6 = 471,
    KC_FN_F7 = 472,
    KC_FN_F8 = 473,
    KC_FN_F9 = 474,
    KC_FN_F10 = 475,
    KC_FN_F11 = 476,
    KC_FN_F12 = 477,
    KC_FN_1 = 478,
    KC_FN_2 = 479,
    KC_FN_D = 480,
    KC_FN_E = 481,
    KC_FN_F = 482,
    KC_FN_S = 483,
    KC_FN_B = 484,
    KC_BRL_DOT1 = 497,
    KC_BRL_DOT2 = 498,
    KC_BRL_DOT3 = 499,
    KC_BRL_DOT4 = 500,
    KC_BRL_DOT5 = 501,
    KC_BRL_DOT6 = 502,
    KC_BRL_DOT7 = 503,
    KC_BRL_DOT8 = 504,
    KC_BRL_DOT9 = 505,
    KC_BRL_DOT10 = 506,
    KC_NUMERIC_0 = 512,
    KC_NUMERIC_1 = 513,
    KC_NUMERIC_2 = 514,
    KC_NUMERIC_3 = 515,
    KC_NUMERIC_4 = 516,
    KC_NUMERIC_5 = 517,
    KC_NUMERIC_6 = 518,
    KC_NUMERIC_7 = 519,
    KC_NUMERIC_8 = 520,
    KC_NUMERIC_9 = 521,
    KC_NUMERIC_STAR = 522,
    KC_NUMERIC_POUND = 523,
    KC_NUMERIC_A = 524,
    KC_NUMERIC_B = 525,
    KC_NUMERIC_C = 526,
    KC_NUMERIC_D = 527,
    KC_CAMERA_FOCUS = 528,
    KC_WPS_BUTTON = 529,
    KC_TOUCHPAD_TOGGLE = 530,
    KC_TOUCHPAD_ON = 531,
    KC_TOUCHPAD_OFF = 532,
    KC_CAMERA_ZOOMIN = 533,
    KC_CAMERA_ZOOMOUT = 534,
    KC_CAMERA_UP = 535,
    KC_CAMERA_DOWN = 536,
    KC_CAMERA_LEFT = 537,
    KC_CAMERA_RIGHT = 538,
    KC_ATTENDANT_ON = 539,
    KC_ATTENDANT_OFF = 540,
    KC_ATTENDANT_TOGGLE = 541,
    KC_LIGHTS_TOGGLE = 542,
    KC_ALS_TOGGLE = 560,
    KC_ROTATE_LOCK_TOGGLE = 561,
    KC_BUTTONCONFIG = 576,
    KC_TASKMANAGER = 577,
    KC_JOURNAL = 578,
    KC_CONTROLPANEL = 579,
    KC_APPSELECT = 580,
    KC_SCREENSAVER = 581,
    KC_VOICECOMMAND = 582,
    KC_ASSISTANT = 583,
    KC_BRIGHTNESS_MIN = 592,
    KC_BRIGHTNESS_MAX = 593,
    KC_KBDINPUTASSIST_PREV = 608,
    KC_KBDINPUTASSIST_NEXT = 609,
    KC_KBDINPUTASSIST_PREVGROUP = 610,
    KC_KBDINPUTASSIST_NEXTGROUP = 611,
    KC_KBDINPUTASSIST_ACCEPT = 612,
    KC_KBDINPUTASSIST_CANCEL = 613,
    KC_RIGHT_UP = 614,
    KC_RIGHT_DOWN = 615,
    KC_LEFT_UP = 616,
    KC_LEFT_DOWN = 617,
    KC_ROOT_MENU = 618,
    KC_MEDIA_TOP_MENU = 619,
    KC_NUMERIC_11 = 620,
    KC_NUMERIC_12 = 621,
    KC_AUDIO_DESC = 622,
    KC_3D_MODE = 623,
    KC_NEXT_FAVORITE = 624,
    KC_STOP_RECORD = 625,
    KC_PAUSE_RECORD = 626,
    KC_VOD = 627,
    KC_UNMUTE = 628,
    KC_FASTREVERSE = 629,
    KC_SLOWREVERSE = 630,
    KC_DATA = 631,
    KC_ONSCREEN_KEYBOARD = 632,
    KC_MAX = 767,
    NotImplemented = 768,
}

#[derive(FromPrimitive, ToPrimitive, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SynCode {
    Report = 0,
    Config = 1,
    MTReport = 2,
    Dropped = 3,
    Max = 15,
}

pub trait InputEventSource: Send {
    fn recv(&mut self) -> Result<InputEvent>;
}

pub trait InputEventSink: Send {
    fn send(&mut self, e: InputEvent) -> Result<()>;
}
