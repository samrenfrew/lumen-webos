use crate::ffi::*;
use crate::note;
use crate::shaders::{FRAGMENT_HEAD, VERTEX};
use std::cell::{Cell, RefCell};
use std::ffi::{c_char, c_void, CStr};
use std::ptr;

pub const DESIGN_W: f32 = 1920.0;
pub const DESIGN_H: f32 = 1080.0;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Tex {
    pub tex: GLuint,
    pub w: i32,
    pub h: i32,
}

pub struct Program {
    id: GLuint,
    known: RefCell<Vec<(*const c_char, GLint)>>,
}

pub struct Gfx {
    pub screen_w: i32,
    pub screen_h: i32,
    flip: Cell<f32>,
    drawing_target: GLuint,
}

impl Tex {
    pub fn bytes(&self) -> usize {
        (self.w.max(0) * self.h.max(0) * 4) as usize
    }
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

type ProgramBinary = unsafe extern "C" fn(GLuint, GLenum, *const c_void, GLsizei);
type GetProgramBinary = unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLenum, *mut c_void);

fn gl_proc<T>(names: &[&CStr]) -> Option<T> {
    names.iter().find_map(|name| {
        let found = unsafe { SDL_GL_GetProcAddress(name.as_ptr()) };
        (!found.is_null()).then(|| unsafe { std::mem::transmute_copy::<*mut c_void, T>(&found) })
    })
}

pub fn shader_key(renderer: &str, parts: &[&str]) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    renderer.hash(&mut hasher);
    parts.hash(&mut hasher);
    format!("shader-{:016x}.bin", hasher.finish())
}

fn from_kept(file: &str) -> Option<GLuint> {
    let load: ProgramBinary = gl_proc(&[c"glProgramBinary", c"glProgramBinaryOES"])?;
    let bytes = std::fs::read(crate::store::path(file)).ok().filter(|b| b.len() > 4)?;
    let format = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    unsafe {
        let id = glCreateProgram();
        load(id, format, bytes[4..].as_ptr().cast(), (bytes.len() - 4) as GLsizei);
        let mut ok = 0;
        glGetProgramiv(id, GL_LINK_STATUS, &mut ok);
        if ok == 0 {
            glDeleteProgram(id);
            return None;
        }
        Some(id)
    }
}

fn keep_binary(id: GLuint, file: &str) {
    let Some(read) = gl_proc::<GetProgramBinary>(&[c"glGetProgramBinary", c"glGetProgramBinaryOES"]) else { return };
    unsafe {
        let mut length = 0;
        glGetProgramiv(id, GL_PROGRAM_BINARY_LENGTH, &mut length);
        if length <= 0 {
            return;
        }
        let (mut written, mut format) = (0, 0);
        let mut bytes = vec![0u8; 4 + length as usize];
        read(id, length, &mut written, &mut format, bytes[4..].as_mut_ptr().cast());
        if written <= 0 {
            return;
        }
        bytes[..4].copy_from_slice(&format.to_le_bytes());
        bytes.truncate(4 + written as usize);
        crate::store::write(file, &bytes);
    }
}

impl Program {
    pub fn new(name: &str, fragment: &str) -> Program {
        let renderer = unsafe { glGetString(GL_RENDERER) };
        let renderer = if renderer.is_null() { String::new() } else { unsafe { CStr::from_ptr(renderer.cast()) }.to_string_lossy().into_owned() };
        let file = shader_key(&renderer, &[VERTEX, FRAGMENT_HEAD, fragment]);
        if let Some(id) = from_kept(&file) {
            return Program { id, known: RefCell::new(Vec::new()) };
        }
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
            } else {
                keep_binary(id, &file);
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
        let (mut quad_buffer, mut drawing_target) = (0, 0);
        unsafe {
            glGenFramebuffers(1, &mut drawing_target);
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
        Gfx { screen_w, screen_h, flip: Cell::new(-1.0), drawing_target }
    }

    pub fn upload_rgba(&self, pixels: &[u8], w: i32, h: i32) -> Tex {
        if w <= 0 || h <= 0 || pixels.len() < (w * h * 4) as usize {
            return Tex::default();
        }
        Tex { tex: new_texture(w, h, pixels.as_ptr().cast()), w, h }
    }

    pub fn free(&self, t: &mut Tex) {
        if t.tex != 0 {
            unsafe { glDeleteTextures(1, &t.tex) }
        }
        t.tex = 0;
    }

    pub fn free_all(&self, pictures: Vec<Tex>) {
        for mut picture in pictures {
            self.free(&mut picture);
        }
    }

    pub fn blank(&self, w: i32, h: i32) -> Tex {
        Tex { tex: new_texture(w, h, ptr::null()), w, h }
    }

    pub fn into_screen(&self) {
        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, 0);
            glViewport(0, 0, self.screen_w, self.screen_h);
        }
        self.flip.set(-1.0);
    }

    fn into_texture(&self, picture: &Tex, w: i32, h: i32) {
        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, self.drawing_target);
            glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, picture.tex, 0);
            if glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
                note!("picture {}x{}: cannot be drawn into", picture.w, picture.h);
            }
            glViewport(0, 0, w, h);
        }
        self.flip.set(1.0);
    }

    pub fn stretched_over(&self, picture: &Tex) {
        self.into_texture(picture, picture.w, picture.h);
    }

    pub fn at_design_scale_into(&self, picture: &Tex) {
        self.into_texture(picture, DESIGN_W as i32, DESIGN_H as i32);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kept_shader_is_found_again_only_for_the_same_source_on_the_same_chip() {
        let key = shader_key("PowerVR B-Series BXE-4-32", &["vertex", "fragment"]);
        assert_eq!(key, shader_key("PowerVR B-Series BXE-4-32", &["vertex", "fragment"]));
        assert_ne!(key, shader_key("PowerVR B-Series BXE-4-32", &["vertex", "fragment changed"]));
        assert_ne!(key, shader_key("llvmpipe", &["vertex", "fragment"]));
        assert!(key.starts_with("shader-") && key.ends_with(".bin"));
    }
}
