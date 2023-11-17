use std::{sync::mpsc::{Receiver, channel}, io::Write};
use termion::raw::IntoRawMode;

mod gl {
    #![allow(warnings)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod utils;
mod context;
mod vrm;
mod shader;
mod fig;
mod tracking;
mod framebuffer;
mod avatar;
mod mesh;
mod texture;
mod term;

fn render_loop(
    tracking_state: tracking::SharedTrackingState,
    control: fig::Control,
    command_receiver: Receiver<fig::ControlCommand>
) {
    let ctx = context::Context::new();
    let mut term0 = term::Term::new();
    let mut term1 = term::Term::new();
    let mut term = &mut term0;
    let mut lastterm = &mut term1;
    let mut term_counter = 0;
    let mut keyframe_counter = 0;
    let mut players = avatar::PalettePlayers::new(&ctx);
    let shader = shader::Shader::new(&ctx, "../assets/shader.vert", "../assets/shader.frag");
    let stdout = std::io::stdout().into_raw_mode().unwrap();
    let mut raw_stdout = stdout.into_raw_mode().unwrap();

    let mut rebroadcast_conn = std::net::TcpStream::connect("colonq.computer:31340").ok();
    if let Some(conn) = rebroadcast_conn.as_mut() {
        log::info!("Connected to rebroadcast server");
        conn.write("LCOLONQ!".as_bytes()).expect("failed to send broadcast header");
    } else {
        log::info!("Failed to connect to rebroadcast server");
    }

    let (mut websocket, _) = tungstenite::connect("wss://colonq.computer/bullfrog/api/channel/broadcast?token=foobar").expect("failed to open websocket");

    let mut avatar_old = avatar::Avatar::new(
        &ctx, "../assets/colonq_v1.vrm",
        |_| glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::new(1.0, 1.0, 1.0),
            glam::Quat::from_rotation_y(std::f32::consts::PI / 2.0),
            glam::Vec3::new(0.0, 0.02, 0.0)
        ),
    ); 
    let mut avatar_new = avatar::Avatar::new(
        &ctx, "../assets/lcolonq_flat.vrm",
        |sts: &fig::Control| glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::new(1.0, 1.0, 1.0),
            glam::Quat::from_rotation_y(
                std::f32::consts::PI / 2.0 +
                    if sts.is("forsen") {
                        -0.30
                    } else {
                        0.0
                    }
            ),
            glam::Vec3::new(-0.10, -0.07, 0.10),
        ),
    );
    // avatar_new.add_hat(avatar::hat::Hat::cone(&ctx));

    let mut fb = framebuffer::Framebuffer::new(&ctx, true, (64, 64), (0, 0));
    let pixels_len = (ctx.dims.w * ctx.dims.h * 4.0) as usize;
    let pixels = vec![0; pixels_len];
    let screen = framebuffer::Framebuffer {
        fbo: 0,
        tex: 0,
        dims: ctx.dims.clone(),
        offsets: (0, 0),
        pixels_len,
        pixels,
    };

    let fb_projection = glam::Mat4::perspective_lh(
        std::f32::consts::PI / 4.0,
        fb.dims.w / fb.dims.h,
        0.1,
        10.0,
    );

    let mut event_pump = ctx.sdl2.event_pump().unwrap();
    let mut framecount = 0;
    let mut float_time: f32 = 0.0;

    log::info!("Starting model renderer");
    print!("{}", termion::cursor::Hide);

    let dt: f32 = 1.0 / 60.0;
    let mut last = std::time::Instant::now();
    let mut acc: f32 = 0.0;
    'mainloop: loop {
        unsafe {
            let err = gl::GetError();
            if err != 0 {
                log::error!("OpenGL error: {:?}", err);
                break;
            }
        }

        let avatar = if control.is("old") {
            &mut avatar_old
        } else {
            &mut avatar_new
        };

        acc += last.elapsed().as_secs_f32();
        last = std::time::Instant::now();

        // update position, if enough time has accumulated since last update
        while acc >= dt {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. } => {
                        break 'mainloop;
                    },
                    _ => (),
                }
            }

            avatar.transform_bone("head", &glam::Mat4::from_rotation_translation(
                tracking_state.lock().unwrap().orientation.inverse(),
                glam::Vec3::ZERO,
            ));
            avatar.transform_bone("leftUpperArm", &glam::Mat4::from_rotation_translation(
                glam::Quat::from_rotation_z(-std::f32::consts::PI / 4.0),
                glam::Vec3::ZERO,
            ));
            avatar.transform_bone("rightUpperArm", &glam::Mat4::from_rotation_translation(
                glam::Quat::from_rotation_z(std::f32::consts::PI / 4.0),
                glam::Vec3::ZERO,
            ));

            // render framebuffer to terminal
            framecount = framecount + 1;
            float_time = float_time + 0.02;

            let forsen = control.is("forsen");
            let bgcolor = if forsen {
                (0x2c, 0x52, 0x39)
            } else {
                (ctx.bgcolor.0 as u8, ctx.bgcolor.1 as u8, ctx.bgcolor.2 as u8)
            };
            if framecount == 1000 {
                let mut f = std::fs::File::create("oneframe.txt").unwrap();
                // term.render_stream_nocolor_small(&mut f);
                // term.serialize(&mut f);
                f.write(&term.serialize_and_compress()).unwrap();
            }
            if framecount % 6 == 0 {
                fb.populate_pixels();
                fb.render_term(&ctx, framecount, &mut players, &control.0.lock().unwrap().palette, term, term_counter);
                term_counter += 1;
                term.render_stream(&mut raw_stdout, bgcolor);
                // term.render_stream_nocolor(&mut raw_stdout);
            }
            if framecount % 12 == 0 {
                if let Some(conn) = rebroadcast_conn.as_mut() {
                    term.render_stream_nocolor_small(conn);
                }
                let frame = if keyframe_counter == 0 {
                    term.serialize_and_compress()
                } else {
                    let diff = term::Diff::new(lastterm, term);
                    diff.serialize_and_compress()
                };
                core::mem::swap(&mut term, &mut lastterm);
                websocket.send(tungstenite::Message::Binary(frame))
                    .expect("failed to send");
                keyframe_counter = (keyframe_counter + 1) % 10;
            }

            acc -= dt
        }

        // compute camera position and view matrix
        let camera_pos_base = if control.is("forsen") {
            glam::Vec3::new(0.0, 1.35, -0.4)
        } else {
            glam::Vec3::new(0.0, 1.5, -0.4)
        };
        let camera_pos = if control.is("zoom_wave") {
            let offset = float_time.sin() * 0.1;
            camera_pos_base + glam::Vec3::new(0.0, 0.0, offset)
        } else {
            camera_pos_base
        };
        let up = if control.is("spin") {
            let angle = (std::f32::consts::PI / 2.0)
                + if control.is("spin_direction") {
                    -(float_time / 4.0)
                } else {
                    float_time / 4.0
                };
            glam::Vec3::new(angle.cos(), angle.sin(), 0.0)
        } else {
            glam::Vec3::new(0.0, 1.0, 0.0)
        };
        let view = glam::Mat4::look_at_lh(
            camera_pos,
            camera_pos + if control.is("forsen") {
                glam::Vec3::new(0.0, 0.2, 1.0)
            } else {
                glam::Vec3::new(0.0, 0.0, 1.0)
            },
            up,
        );

        // update video players
        while let Ok(comm) = command_receiver.try_recv() {
            match comm {
                fig::ControlCommand::PlayVideo { pty, url } => {
                    if let Some(player) = players.players.get(&pty) {
                        player.mpv.playlist_load_files(&[(&url, libmpv::FileState::Replace, None)]).unwrap();
                    }
                },
                fig::ControlCommand::ReloadPumpkin => {
                    // avatar.pumpkin.reload(&ctx);
                },
            }
        }
        players.update(&ctx, &control.0.lock().unwrap().palette);

        // render avatar framebuffer
        fb.bind(&ctx);
        unsafe {
            gl::ClearColor(ctx.bgcolor.0 as f32 / 255.0, ctx.bgcolor.1 as f32 / 255.0, ctx.bgcolor.2 as f32 / 255.0, 1.0);
        }
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); }
        shader.bind(&ctx);
        unsafe {
            gl::UniformMatrix4fv(shader.uniform_view, 1, false as u8, view.to_cols_array().as_ptr());
            gl::UniformMatrix4fv(shader.uniform_projection, 1, false as u8, fb_projection.to_cols_array().as_ptr());
            gl::Uniform3fv(shader.uniform_camera_pos, 1, camera_pos.to_array().as_ptr());
            for (enm, uniform) in &shader.uniform_expressions {
                gl::Uniform1f(*uniform, *tracking_state.lock().unwrap().expression_weights.get(enm).unwrap());
            }
        }
        avatar.render(&ctx, &shader, &view, &fb_projection, &control);
        screen.bind(&ctx);
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); }
        unsafe {
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fb.fbo);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, screen.fbo);
            gl::BlitFramebuffer(0, 0, fb.dims.w as _, fb.dims.h as _, 0, 0, 512 as _, 512 as _, gl::COLOR_BUFFER_BIT, gl::NEAREST);
        }

        // refresh display
        ctx.window.gl_swap_window();
    }
}

fn main() {
    simple_logging::log_to_file("colonq.log", log::LevelFilter::Debug).unwrap();
    log_panics::init();

    let tracking_state = tracking::TrackingState::new();
    let shared_tracking_state = std::sync::Arc::new(std::sync::Mutex::new(tracking_state));
    let shared_tracking_state_clone = shared_tracking_state.clone();

    let control = fig::Control::new();
    let (command_sender, command_receiver) = channel();

    let render_handle = std::thread::spawn({
        let control = control.clone();
        move || {
            render_loop(shared_tracking_state_clone, control, command_receiver);
        }
    });

    tracking::TrackingState::run(shared_tracking_state.clone());
    control.run(command_sender);

    render_handle.join().unwrap();
}
