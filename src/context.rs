use crate::{gl, utils, framebuffer};

use std::{collections::HashMap, env, cell::RefCell};

use colors_transform::{Rgb, Color};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum PaletteType {
    Hair,
    Eyes,
    Skin,
    Highlight,
    Eyebags,
    PartyHat,
}

impl PaletteType {
    pub fn from_string(nm: &str) -> Option<Self> {
        match nm {
            "hair" => Some(Self::Hair),
            "eyes" => Some(Self::Eyes),
            "skin" => Some(Self::Skin),
            "highlight" => Some(Self::Highlight),
            "eyebags" => Some(Self::Eyebags),
            "hat" => Some(Self::PartyHat),
            _ => None,
        }
    }
    pub fn from_color((r, g, b): (u8, u8, u8)) -> Option<Self> {
        if r >= 186 && r <= 188 && g >= 176 && g <= 178 && b >= 189 && b <= 191 {
            Some(Self::Hair)
        } else if r >= 158 && r <= 162 && g >= 148 && g <= 152 && b >= 159 && b <= 162 {
            Some(Self::Highlight)
        } else if g > r && g > b {
            Some(Self::Eyes)
        } else if r >= 242 && r <= 246 && g >= 238 && g <= 242 && b >= 234 && b <= 238 {
            Some(Self::Skin)
        } else if r == 182 && g == 142 && b == 139 {
            Some(Self::Eyebags)
        } else if r == 255 && g == 0 && b == 0 {
            Some(Self::PartyHat)
        } else {
            None
        }
    }
}

pub struct VideoPlayer<'a> {
    pub fb: framebuffer::Framebuffer,
    pub mpv: &'a libmpv::Mpv,
    pub mpv_render_context: libmpv::render::RenderContext,
    pub mpv_event_context: libmpv::events::EventContext<'a>,
}

impl<'a> VideoPlayer<'a> {
    pub fn new(ctx: &Context) -> Self {
        let mpv = Box::leak(Box::new(libmpv::Mpv::new().unwrap()));
        mpv.set_property("mute", "yes").unwrap();
        mpv.set_property("keepaspect", "no").unwrap();
        mpv.set_property("loop", "inf").unwrap();
        mpv.set_property("video-timing-offset", 0).unwrap();
        mpv.set_property("ytdl-format", "bestvideo[height=?144][fps<=?30][vcodec!=?vp9]+bestaudio/best").unwrap();
        let mpv_render_context = libmpv::render::RenderContext::new(
            unsafe { mpv.ctx.as_mut() },
            vec![
                libmpv::render::RenderParam::ApiType(libmpv::render::RenderParamApiType::OpenGl),
                libmpv::render::RenderParam::InitParams(libmpv::render::OpenGLInitParams {
                    get_proc_address,
                    ctx: &ctx.video,
                }),
            ],
        ).unwrap();
        let mpv_event_context = mpv.create_event_context();
        let fb = framebuffer::Framebuffer::new(&ctx, false, (114, 64), (0, 0));

        Self {
            fb,
            mpv,
            mpv_render_context,
            mpv_event_context,
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        self.fb.bind(&ctx);
        self.mpv_render_context
            .render::<i32>(self.fb.fbo as _, self.fb.dims.w as _, self.fb.dims.h as _, true)
            .unwrap();
        // log::info!(
        //     "hair: {:?} {:?}",
        //     self.mpv.get_property::<String>("path"),
        //     self.mpv.get_property::<f64>("time-remaining"),
        // );
        self.fb.populate_pixels();
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> (u8, u8, u8) {
        let offsetx = (114 - 64)/2;
        // let offsetx = 0;
        let invx = (self.fb.dims.h as i32) - (x + 1);
        let idx = ((self.fb.dims.w as i32) * invx * 4 + (offsetx + y) * 4) as usize;
        let r = if let Some(r) = self.fb.pixels.get(idx) {
            *r
        } else { 0 };
        let g = if let Some(g) = self.fb.pixels.get(idx + 1) {
            *g
        } else { 0 };
        let b = if let Some(b) = self.fb.pixels.get(idx + 2) {
            *b
        } else { 0 };
        (r, g, b)
    }
}

#[allow(dead_code)]
pub struct Context {
    pub sdl2: sdl2::Sdl,
    pub video: sdl2::VideoSubsystem,
    pub image: sdl2::image::Sdl2ImageContext,
    pub gl_context: sdl2::video::GLContext,
    pub window: sdl2::video::Window,
    pub clearcolor: Rgb,
    pub bgcolor: (i32, i32, i32),
    pub dims: utils::Dimensions,
    pub attrib_expressions: HashMap<String, gl::types::GLuint>,
}

pub fn get_proc_address(
    video: &&sdl2::VideoSubsystem, name: &str,
) -> *mut std::ffi::c_void {
    video.gl_get_proc_address(name) as *mut std::ffi::c_void
}

impl Context {
    pub fn new() -> Self {
        let sdl2 = sdl2::init().unwrap();
        let video = sdl2.video().unwrap();
        let image = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();
        // sdl2.mouse().set_relative_mouse_mode(true);
        // sdl2::hint::set("SDL_HINT_MOUSE_RELATIVE_MODE_WARP", "1");

        // let gl_attr = video.gl_attr();
        // gl_attr.set_context_major_version(3);
        // gl_attr.set_context_minor_version(2);

        let window = video
            .window("colonq", 640 as _, 360 as _)
            .opengl()
        // .fullscreen_desktop()
            .build()
            .unwrap();
        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| video.gl_get_proc_address(s) as *const std::ffi::c_void);

        let (winw, winh) = window.size();

        let attrib_expressions: HashMap<String, gl::types::GLuint> = HashMap::from([
            ("happy".to_owned(), 5),
            ("angry".to_owned(), 6),
            ("sad".to_owned(), 7),
            ("relaxed".to_owned(), 8),
            ("surprised".to_owned(), 9),
            ("aa".to_owned(), 10),
            ("ih".to_owned(), 11),
            ("ou".to_owned(), 12),
            ("ee".to_owned(), 13),
            ("oh".to_owned(), 14),
            ("blink".to_owned(), 15),
        ]);

        let rgb = match env::var("COLONQ_BGCOLOR") {
            Ok(colorstr) => {
                Rgb::from_hex_str(&colorstr).unwrap()
            },
            Err(_) => {
                Rgb::from(0x15 as _, 0x05 as _, 0x0f as _)
            },
        };

        unsafe {
            // set unchanging options
            gl::Viewport(0, 0, winw as _, winh as _);
            gl::ClearColor(rgb.get_red() / 255.0, rgb.get_green() / 255.0, rgb.get_blue() / 255.0, 1.0);
            gl::ClearDepth(1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            // gl::Enable(gl::CULL_FACE); // NOTE: THIS BREAKS LIBMPV MAKE SURE TO DISABLE IT :3
            // gl::CullFace(gl::FRONT);
        }

        Self {
            sdl2,
            video,
            image,
            gl_context,
            window,
            clearcolor: rgb,

            bgcolor: (rgb.get_red() as _, rgb.get_green() as _, rgb.get_blue() as _),
            dims: utils::Dimensions {
                w: winw as _,
                h: winh as _,
            },
            attrib_expressions,
        }
    }
}
