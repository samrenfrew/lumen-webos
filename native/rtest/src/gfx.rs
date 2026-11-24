use crate::ffi::*;
use crate::note;
use crate::shaders::{FRAGMENT_HEAD, VERTEX};
use std::cell::{Cell, RefCell};
use std::ffi::{c_char, c_void, CStr};
use std::ptr;

pub const DESIGN_W: f32 = 1920.0;
pub const DESIGN_H: f32 = 1080.0;

#[derive(Clone, Copy, Default)]
pub struct Tex {
    pub tex: GLuint,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Copy, Default)]
pub struct Target {
    pub fbo: GLuint,
    pub t: Tex,
}

pub struct Program {
    id: GLuint,
    known: RefCell<Vec<(*const c_char, GLint)>>,
}

pub struct Gfx {
    screen_w: i32,
    screen_h: i32,
    flip: Cell<f32>,
}

fn info_log(read: unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut c_char), object: GLuint) -> String {
    let mut why = [0 as c_char; 1024];
    unsafe {
        read(object, why.len() as GLsizei, ptr::null_mut(), why.as_mut_ptr());
        CStr::from_ptr(why.as_ptr()).to_string_lossy().into_owned()
    }
}

fn compile(kind: GLenum, name: &str, parts: &[&str]) -> GLuint {
    let strings: Vec<*const c_char> = parts.iter().map(|p| p.as_ptr().cast()).collect();
    let lengths: Vec<GLint> = parts.iter().map(|p| p.len() as GLint).collect();
    unsafe {
        let shader = glCreateShader(kind);
        glShaderSource(shader, parts.len() as GLsizei, strings.as_ptr(), lengths.as_ptr());
        glCompileShader(shader);
        let mut ok = 0;
        glGetShaderiv(shader, GL_COMPILE_STATUS, &mut ok);
        if ok == 0 {
            note!("shader {}: {}", name, info_log(glGetShaderInfoLog, shader));
        }
        shader
    }
}

fn new_texture(w: i32, h: i32, pixels: *const c_void) -> GLuint {
    let mut tex = 0;
    unsafe {
        glGenTextures(1, &mut tex);
        glBindTexture(GL_TEXTURE_2D, tex);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA as GLint, w, h, 0, GL_RGBA, GL_UNSIGNED_BYTE, pixels);
    }
    tex
}

impl Program {
    pub fn new(name: &str, fragment: &str) -> Program {
        unsafe {
            let id = glCreateProgram();
            glAttachShader(id, compile(GL_VERTEX_SHADER, name, &[VERTEX]));
            glAttachShader(id, compile(GL_FRAGMENT_SHADER, name, &[FRAGMENT_HEAD, fragment]));
            glBindAttribLocation(id, 0, c"a_pos".as_ptr());
            glLinkProgram(id);
            let mut ok = 0;
            glGetProgramiv(id, GL_LINK_STATUS, &mut ok);
            if ok == 0 {
                note!("program {}: {}", name, info_log(glGetProgramInfoLog, id));
            }
            Program { id, known: RefCell::new(Vec::new()) }
        }
    }

    fn loc(&self, name: &'static CStr) -> GLint {
        let mut known = self.known.borrow_mut();
        if let Some(&(_, at)) = known.iter().find(|(n, _)| *n == name.as_ptr()) {
            return at;
        }
        let at = unsafe { glGetUniformLocation(self.id, name.as_ptr()) };
        known.push((name.as_ptr(), at));
        at
    }

    pub fn activate(&self) {
        unsafe { glUseProgram(self.id) }
    }

    pub fn bind(&self, unit: u32, name: &'static CStr, tex: GLuint) {
        unsafe {
            glUseProgram(self.id);
            glActiveTexture(GL_TEXTURE0 + unit);
            glBindTexture(GL_TEXTURE_2D, tex);
            glUniform1i(self.loc(name), unit as GLint);
        }
    }

    pub fn set1(&self, name: &'static CStr, x: f32) {
        unsafe { glUniform1f(self.loc(name), x) }
    }

    pub fn set2(&self, name: &'static CStr, x: f32, y: f32) {
        unsafe { glUniform2f(self.loc(name), x, y) }
    }

    pub fn set3(&self, name: &'static CStr, v: [f32; 3]) {
        unsafe { glUniform3f(self.loc(name), v[0], v[1], v[2]) }
    }

    pub fn set4(&self, name: &'static CStr, x: f32, y: f32, z: f32, w: f32) {
        unsafe { glUniform4f(self.loc(name), x, y, z, w) }
    }
}

impl Gfx {
    pub fn new(screen_w: i32, screen_h: i32) -> Gfx {
        const CORNERS: [GLfloat; 8] = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let mut quad_buffer = 0;
        unsafe {
            glGenBuffers(1, &mut quad_buffer);
            glBindBuffer(GL_ARRAY_BUFFER, quad_buffer);
            glBufferData(GL_ARRAY_BUFFER, size_of_val(&CORNERS) as isize, CORNERS.as_ptr().cast(), GL_STATIC_DRAW);
            glEnableVertexAttribArray(0);
            glVertexAttribPointer(0, 2, GL_FLOAT, 0, 0, ptr::null());
            glDisable(GL_DEPTH_TEST);
            glClearColor(0.0, 0.0, 0.0, 1.0);
            glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
            glPixelStorei(GL_PACK_ALIGNMENT, 1);
        }
        Gfx { screen_w, screen_h, flip: Cell::new(-1.0) }
    }

    pub fn upload(&self, surface: *mut SDL_Surface, premultiply: bool) -> Tex {
        unsafe {
            let rgba = SDL_ConvertSurfaceFormat(surface, SDL_PIXELFORMAT_RGBA32, 0);
            if rgba.is_null() {
                return Tex::default();
            }
            let (w, h, pitch) = ((*rgba).w as usize, (*rgba).h as usize, (*rgba).pitch as usize);
            let mut tight = vec![0u8; w * h * 4];
            for (y, row) in tight.chunks_exact_mut(w * 4).enumerate() {
                ptr::copy_nonoverlapping((*rgba).pixels.cast::<u8>().add(y * pitch), row.as_mut_ptr(), w * 4);
                if premultiply {
                    for px in row.chunks_exact_mut(4) {
                        let a = px[3] as u32;
                        px[0] = (px[0] as u32 * a / 255) as u8;
                        px[1] = (px[1] as u32 * a / 255) as u8;
                        px[2] = (px[2] as u32 * a / 255) as u8;
                    }
                }
            }
            let t = Tex { tex: new_texture(w as i32, h as i32, tight.as_ptr().cast()), w: w as i32, h: h as i32 };
            SDL_FreeSurface(rgba);
            t
        }
    }

    pub fn free(&self, t: &mut Tex) {
        if t.tex != 0 {
            unsafe { glDeleteTextures(1, &t.tex) }
        }
        t.tex = 0;
    }

    pub fn target(&self, w: i32, h: i32) -> Target {
        let mut target = Target { fbo: 0, t: Tex { tex: new_texture(w, h, ptr::null()), w, h } };
        unsafe {
            glGenFramebuffers(1, &mut target.fbo);
            glBindFramebuffer(GL_FRAMEBUFFER, target.fbo);
            glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, target.t.tex, 0);
            if glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
                note!("target {}x{}: not complete", w, h);
            }
        }
        target
    }

    pub fn shared_fbo(&self) -> GLuint {
        let mut fbo = 0;
        unsafe { glGenFramebuffers(1, &mut fbo) }
        fbo
    }

    pub fn target_on(&self, shared_fbo: GLuint, w: i32, h: i32) -> Target {
        let target = Target { fbo: shared_fbo, t: Tex { tex: new_texture(w, h, ptr::null()), w, h } };
        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, shared_fbo);
            glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, target.t.tex, 0);
        }
        target
    }

    pub fn target_free(&self, target: &mut Target) {
        if target.fbo != 0 {
            unsafe { glDeleteFramebuffers(1, &target.fbo) }
        }
        target.fbo = 0;
        self.free(&mut target.t);
    }

    pub fn into_screen(&self) {
        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, 0);
            glViewport(0, 0, self.screen_w, self.screen_h);
        }
        self.flip.set(-1.0);
    }

    pub fn into_target(&self, target: &Target) {
        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, target.fbo);
            glViewport(0, 0, target.t.w, target.t.h);
        }
        self.flip.set(1.0);
    }

    pub fn into_plate(&self, target: &Target) {
        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, target.fbo);
            glViewport(0, 0, DESIGN_W as i32, DESIGN_H as i32);
        }
        self.flip.set(1.0);
    }

    pub fn quad(&self, program: &Program, x: f32, y: f32, w: f32, h: f32) {
        program.activate();
        program.set4(c"u_quad", x, y, w, h);
        program.set1(c"u_flip", self.flip.get());
        unsafe { glDrawArrays(GL_TRIANGLE_STRIP, 0, 4) }
    }

    pub fn read_screen(&self) -> *mut SDL_Surface {
        let (w, h) = (self.screen_w as usize, self.screen_h as usize);
        unsafe {
            let shot = SDL_CreateRGBSurfaceWithFormat(0, self.screen_w, self.screen_h, 32, SDL_PIXELFORMAT_RGBA32);
            if shot.is_null() {
                return shot;
            }
            let mut pixels = vec![0u8; w * h * 4];
            glReadPixels(0, 0, self.screen_w, self.screen_h, GL_RGBA, GL_UNSIGNED_BYTE, pixels.as_mut_ptr().cast());
            let pitch = (*shot).pitch as usize;
            for y in 0..h {
                ptr::copy_nonoverlapping(pixels.as_ptr().add((h - 1 - y) * w * 4), (*shot).pixels.cast::<u8>().add(y * pitch), w * 4);
            }
            shot
        }
    }
}
