#![allow(non_camel_case_types, non_snake_case, dead_code)]

use std::ffi::{c_char, c_int, c_long, c_void};

pub type GLuint = u32;
pub type GLint = i32;
pub type GLenum = u32;
pub type GLsizei = i32;
pub type GLfloat = f32;

pub const GL_TRIANGLE_STRIP: GLenum = 0x0005;
pub const GL_ONE: GLenum = 1;
pub const GL_ZERO: GLenum = 0;
pub const GL_ONE_MINUS_SRC_ALPHA: GLenum = 0x0303;
pub const GL_DEPTH_TEST: GLenum = 0x0B71;
pub const GL_BLEND: GLenum = 0x0BE2;
pub const GL_SCISSOR_TEST: GLenum = 0x0C11;
pub const GL_UNPACK_ALIGNMENT: GLenum = 0x0CF5;
pub const GL_PACK_ALIGNMENT: GLenum = 0x0D05;
pub const GL_TEXTURE_2D: GLenum = 0x0DE1;
pub const GL_UNSIGNED_BYTE: GLenum = 0x1401;
pub const GL_FLOAT: GLenum = 0x1406;
pub const GL_RGBA: GLenum = 0x1908;
pub const GL_RENDERER: GLenum = 0x1F01;
pub const GL_VERSION: GLenum = 0x1F02;
pub const GL_LINEAR: GLint = 0x2601;
pub const GL_TEXTURE_MAG_FILTER: GLenum = 0x2800;
pub const GL_TEXTURE_MIN_FILTER: GLenum = 0x2801;
pub const GL_TEXTURE_WRAP_S: GLenum = 0x2802;
pub const GL_TEXTURE_WRAP_T: GLenum = 0x2803;
pub const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
pub const GL_CLAMP_TO_EDGE: GLint = 0x812F;
pub const GL_TEXTURE0: GLenum = 0x84C0;
pub const GL_ARRAY_BUFFER: GLenum = 0x8892;
pub const GL_STATIC_DRAW: GLenum = 0x88E4;
pub const GL_FRAGMENT_SHADER: GLenum = 0x8B30;
pub const GL_VERTEX_SHADER: GLenum = 0x8B31;
pub const GL_COMPILE_STATUS: GLenum = 0x8B81;
pub const GL_LINK_STATUS: GLenum = 0x8B82;
pub const GL_PROGRAM_BINARY_LENGTH: GLenum = 0x8741;
pub const GL_FRAMEBUFFER: GLenum = 0x8D40;
pub const GL_COLOR_ATTACHMENT0: GLenum = 0x8CE0;
pub const GL_FRAMEBUFFER_COMPLETE: GLenum = 0x8CD5;

#[link(name = "GLESv2")]
extern "C" {
    pub fn glActiveTexture(texture: GLenum);
    pub fn glAttachShader(program: GLuint, shader: GLuint);
    pub fn glBindAttribLocation(program: GLuint, index: GLuint, name: *const c_char);
    pub fn glBindBuffer(target: GLenum, buffer: GLuint);
    pub fn glBindFramebuffer(target: GLenum, framebuffer: GLuint);
    pub fn glBindTexture(target: GLenum, texture: GLuint);
    pub fn glBlendFuncSeparate(src_rgb: GLenum, dst_rgb: GLenum, src_alpha: GLenum, dst_alpha: GLenum);
    pub fn glBufferData(target: GLenum, size: isize, data: *const c_void, usage: GLenum);
    pub fn glCheckFramebufferStatus(target: GLenum) -> GLenum;
    pub fn glClear(mask: u32);
    pub fn glScissor(x: GLint, y: GLint, w: GLsizei, h: GLsizei);
    pub fn glClearColor(r: GLfloat, g: GLfloat, b: GLfloat, a: GLfloat);
    pub fn glCompileShader(shader: GLuint);
    pub fn glCreateProgram() -> GLuint;
    pub fn glDeleteProgram(program: GLuint);
    pub fn glCreateShader(kind: GLenum) -> GLuint;
    pub fn glDeleteFramebuffers(n: GLsizei, framebuffers: *const GLuint);
    pub fn glDeleteTextures(n: GLsizei, textures: *const GLuint);
    pub fn glDisable(cap: GLenum);
    pub fn glDrawArrays(mode: GLenum, first: GLint, count: GLsizei);
    pub fn glEnable(cap: GLenum);
    pub fn glEnableVertexAttribArray(index: GLuint);
    pub fn glFinish();
    pub fn glFramebufferTexture2D(target: GLenum, attachment: GLenum, textarget: GLenum, texture: GLuint, level: GLint);
    pub fn glGenBuffers(n: GLsizei, buffers: *mut GLuint);
    pub fn glGenFramebuffers(n: GLsizei, framebuffers: *mut GLuint);
    pub fn glGenTextures(n: GLsizei, textures: *mut GLuint);
    pub fn glGetProgramInfoLog(program: GLuint, size: GLsizei, length: *mut GLsizei, log: *mut c_char);
    pub fn glGetProgramiv(program: GLuint, name: GLenum, params: *mut GLint);
    pub fn glGetShaderInfoLog(shader: GLuint, size: GLsizei, length: *mut GLsizei, log: *mut c_char);
    pub fn glGetShaderiv(shader: GLuint, name: GLenum, params: *mut GLint);
    pub fn glGetString(name: GLenum) -> *const c_char;
    pub fn glGetUniformLocation(program: GLuint, name: *const c_char) -> GLint;
    pub fn glLinkProgram(program: GLuint);
    pub fn glPixelStorei(name: GLenum, param: GLint);
    pub fn glReadPixels(x: GLint, y: GLint, w: GLsizei, h: GLsizei, format: GLenum, kind: GLenum, pixels: *mut c_void);
    pub fn glShaderSource(shader: GLuint, count: GLsizei, strings: *const *const c_char, lengths: *const GLint);
    pub fn glTexImage2D(target: GLenum, level: GLint, internal: GLint, w: GLsizei, h: GLsizei, border: GLint, format: GLenum, kind: GLenum, pixels: *const c_void);
    pub fn glTexParameteri(target: GLenum, name: GLenum, param: GLint);
    pub fn glUniform1f(at: GLint, x: GLfloat);
    pub fn glUniform1i(at: GLint, x: GLint);
    pub fn glUniform2f(at: GLint, x: GLfloat, y: GLfloat);
    pub fn glUniform3f(at: GLint, x: GLfloat, y: GLfloat, z: GLfloat);
    pub fn glUniform4f(at: GLint, x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat);
    pub fn glUseProgram(program: GLuint);
    pub fn glVertexAttribPointer(index: GLuint, size: GLint, kind: GLenum, normalized: u8, stride: GLsizei, pointer: *const c_void);
    pub fn glViewport(x: GLint, y: GLint, w: GLsizei, h: GLsizei);
}

pub const SDL_INIT_VIDEO: u32 = 0x20;
pub const SDL_INIT_EVENTS: u32 = 0x4000;
pub const SDL_WINDOW_FULLSCREEN: u32 = 0x1;
pub const SDL_WINDOW_OPENGL: u32 = 0x2;
pub const SDL_WINDOW_SHOWN: u32 = 0x4;
pub const SDL_GL_RED_SIZE: c_int = 0;
pub const SDL_GL_GREEN_SIZE: c_int = 1;
pub const SDL_GL_BLUE_SIZE: c_int = 2;
pub const SDL_GL_ALPHA_SIZE: c_int = 3;
pub const SDL_GL_DOUBLEBUFFER: c_int = 5;
pub const SDL_GL_DEPTH_SIZE: c_int = 6;
pub const SDL_GL_STENCIL_SIZE: c_int = 7;
pub const SDL_GL_CONTEXT_MAJOR_VERSION: c_int = 17;
pub const SDL_GL_CONTEXT_MINOR_VERSION: c_int = 18;
pub const SDL_GL_CONTEXT_PROFILE_MASK: c_int = 21;
pub const SDL_GL_CONTEXT_PROFILE_ES: c_int = 0x4;
pub const SDL_QUIT: u32 = 0x100;
pub const SDL_WINDOWEVENT: u32 = 0x200;
pub const SDL_KEYDOWN: u32 = 0x300;
pub const SDL_KEYUP: u32 = 0x301;
pub const SDL_MOUSEMOTION: u32 = 0x400;
pub const SDL_MOUSEBUTTONDOWN: u32 = 0x401;
pub const SDL_MOUSEBUTTONUP: u32 = 0x402;
pub const SDL_MOUSEWHEEL: u32 = 0x403;
pub const SDLK_0: i32 = 48;
pub const SDLK_1: i32 = 49;
pub const SDLK_9: i32 = 57;
pub const SDLK_KP_0: i32 = 0x4000_0062;
pub const SDLK_KP_1: i32 = 0x4000_0059;
pub const SDL_USEREVENT: u32 = 0x8000;
pub const SDL_WINDOWEVENT_FOCUS_GAINED: u8 = 12;
pub const SDL_WINDOWEVENT_FOCUS_LOST: u8 = 13;
pub const SDL_PIXELFORMAT_RGBA32: u32 = 0x16762004;
pub const SDL_BLENDMODE_NONE: c_int = 0;
pub const IMG_INIT_JPG: c_int = 1;
pub const IMG_INIT_PNG: c_int = 2;

pub const SDLK_BACKSPACE: i32 = 8;
pub const SDLK_RETURN: i32 = 13;
pub const SDLK_ESCAPE: i32 = 27;
pub const SDLK_Q: i32 = 113;
pub const SDLK_RIGHT: i32 = 0x4000_004F;
pub const SDLK_LEFT: i32 = 0x4000_0050;
pub const SDLK_DOWN: i32 = 0x4000_0051;
pub const SDLK_UP: i32 = 0x4000_0052;
pub const SDLK_KP_ENTER: i32 = 0x4000_0058;
pub const SDLK_AC_BACK: i32 = 0x4000_010E;

#[repr(C)]
pub struct SDL_Window {
    opaque: [u8; 0],
}

#[repr(C)]
pub struct SDL_Surface {
    pub flags: u32,
    pub format: *mut c_void,
    pub w: c_int,
    pub h: c_int,
    pub pitch: c_int,
    pub pixels: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SDL_Rect {
    pub x: c_int,
    pub y: c_int,
    pub w: c_int,
    pub h: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SDL_Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
pub struct SDL_DisplayMode {
    pub format: u32,
    pub w: c_int,
    pub h: c_int,
    pub refresh_rate: c_int,
    pub driverdata: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SDL_Keysym {
    pub scancode: c_int,
    pub sym: i32,
    pub modifiers: u16,
    pub unused: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SDL_KeyboardEvent {
    pub kind: u32,
    pub timestamp: u32,
    pub window_id: u32,
    #[cfg(feature = "webos")]
    pub input_source: u32,
    pub state: u8,
    pub repeat: u8,
    pub padding2: u8,
    pub padding3: u8,
    pub keysym: SDL_Keysym,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SDL_WindowEvent {
    pub kind: u32,
    pub timestamp: u32,
    pub window_id: u32,
    pub event: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SDL_MouseMotionEvent {
    pub kind: u32,
    pub timestamp: u32,
    pub window_id: u32,
    pub which: u32,
    pub state: u32,
    pub x: i32,
    pub y: i32,
    pub xrel: i32,
    pub yrel: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SDL_MouseButtonEvent {
    pub kind: u32,
    pub timestamp: u32,
    pub window_id: u32,
    pub which: u32,
    pub button: u8,
    pub state: u8,
    pub clicks: u8,
    pub padding1: u8,
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SDL_MouseWheelEvent {
    pub kind: u32,
    pub timestamp: u32,
    pub window_id: u32,
    pub which: u32,
    pub x: i32,
    pub y: i32,
    pub direction: u32,
}

#[repr(C)]
pub union SDL_Event {
    pub kind: u32,
    pub key: SDL_KeyboardEvent,
    pub window: SDL_WindowEvent,
    pub motion: SDL_MouseMotionEvent,
    pub button: SDL_MouseButtonEvent,
    pub wheel: SDL_MouseWheelEvent,
    pub padding: [u64; 7],
}

#[link(name = "SDL2")]
extern "C" {
    pub fn SDL_GL_GetProcAddress(proc_name: *const c_char) -> *mut c_void;
    pub fn SDL_ConvertSurfaceFormat(surface: *mut SDL_Surface, format: u32, flags: u32) -> *mut SDL_Surface;
    pub fn SDL_CreateRGBSurfaceWithFormat(flags: u32, w: c_int, h: c_int, depth: c_int, format: u32) -> *mut SDL_Surface;
    pub fn SDL_CreateWindow(title: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, flags: u32) -> *mut SDL_Window;
    pub fn SDL_DestroyWindow(window: *mut SDL_Window);
    pub fn SDL_FreeSurface(surface: *mut SDL_Surface);
    pub fn SDL_GL_CreateContext(window: *mut SDL_Window) -> *mut c_void;
    pub fn SDL_GL_DeleteContext(context: *mut c_void);
    pub fn SDL_GL_GetDrawableSize(window: *mut SDL_Window, w: *mut c_int, h: *mut c_int);
    pub fn SDL_GL_SetAttribute(attr: c_int, value: c_int) -> c_int;
    pub fn SDL_GL_SetSwapInterval(interval: c_int) -> c_int;
    pub fn SDL_GL_SwapWindow(window: *mut SDL_Window);
    pub fn SDL_GetBasePath() -> *mut c_char;
    pub fn SDL_GetCurrentDisplayMode(display: c_int, mode: *mut SDL_DisplayMode) -> c_int;
    pub fn SDL_GetError() -> *const c_char;
    pub fn SDL_GetPerformanceCounter() -> u64;
    pub fn SDL_GetPerformanceFrequency() -> u64;
    pub fn SDL_GetTicks() -> u32;
    pub fn SDL_Init(flags: u32) -> c_int;
    pub fn SDL_PollEvent(event: *mut SDL_Event) -> c_int;
    pub fn SDL_PushEvent(event: *mut SDL_Event) -> c_int;
    pub fn SDL_Quit();
    pub fn SDL_RaiseWindow(window: *mut SDL_Window);
    pub fn SDL_ShowWindow(window: *mut SDL_Window);
    pub fn SDL_HideWindow(window: *mut SDL_Window);
    pub fn SDL_SetHint(name: *const c_char, value: *const c_char) -> c_int;
    pub fn SDL_SetSurfaceBlendMode(surface: *mut SDL_Surface, mode: c_int) -> c_int;
    pub fn SDL_UpperBlit(src: *mut SDL_Surface, src_rect: *const SDL_Rect, dst: *mut SDL_Surface, dst_rect: *mut SDL_Rect) -> c_int;
    pub fn SDL_UpperBlitScaled(src: *mut SDL_Surface, src_rect: *const SDL_Rect, dst: *mut SDL_Surface, dst_rect: *mut SDL_Rect) -> c_int;
    pub fn SDL_WaitEventTimeout(event: *mut SDL_Event, timeout: c_int) -> c_int;
    pub fn SDL_free(mem: *mut c_void);
    pub fn SDL_RWFromConstMem(mem: *const c_void, size: c_int) -> *mut c_void;
}

#[link(name = "SDL2_image")]
extern "C" {
    pub fn IMG_Init(flags: c_int) -> c_int;
    pub fn IMG_Load(path: *const c_char) -> *mut SDL_Surface;
    pub fn IMG_Load_RW(src: *mut c_void, free_src: c_int) -> *mut SDL_Surface;
    pub fn IMG_Quit();
    pub fn IMG_SavePNG(surface: *mut SDL_Surface, path: *const c_char) -> c_int;
}

extern "C" {
    pub fn getuid() -> u32;
    pub fn lumen_abi_checked() -> c_int;
    #[cfg(feature = "webos")]
    pub fn lumen_claim_window(window: *mut SDL_Window, overlay: c_int) -> c_int;
    #[cfg(feature = "webos")]
    pub fn lumen_export_video_window(window: *mut SDL_Window) -> *const c_char;
    #[cfg(feature = "webos")]
    pub fn lumen_place_video(source_w: c_int, source_h: c_int, x: c_int, y: c_int, w: c_int, h: c_int) -> c_int;
}

extern "C" {
    pub fn lumen_text_line(font: *const c_char, other_script_font: *const c_char, px: c_int, letter_spacing: f32, feature: *const c_char, utf8: *const c_char, rgba: *mut *mut u8, w: *mut c_int, h: *mut c_int) -> c_int;
    pub fn lumen_text_free(rgba: *mut u8);
    pub fn lumen_local_time(unix_seconds: i64, out: *mut c_int);
    pub fn lumen_http_get(url: *const c_char, data: *mut *mut u8, len: *mut usize) -> c_long;
    pub fn lumen_http_request(url: *const c_char, key: *const c_char, post_json: *const c_char, data: *mut *mut u8, len: *mut usize) -> c_long;
    pub fn lumen_http_free(data: *mut u8);
}

#[cfg(feature = "webos")]
extern "C" {
    pub fn lumen_shell_state(state: c_int) -> c_int;
    pub fn lumen_luna_open(name: *const c_char) -> *mut c_char;
    pub fn lumen_luna_send(uri: *const c_char, payload: *const c_char, subscribe: c_int, timeout_ms: c_int, call: extern "C" fn(*const c_char, *mut c_void), user: *mut c_void) -> *mut c_char;
    pub fn lumen_luna_wait();
    pub fn lumen_luna_wake();
    pub fn lumen_luna_free(text: *mut c_char);
}
