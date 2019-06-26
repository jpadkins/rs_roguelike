/*
use std::collections::HashSet;
use std::path::Path;

use rand::prelude::*;

use specs::prelude::*;
#[macro_use]
extern crate specs_derive;

use sdl2::event::{Event, WindowEvent};
use sdl2::image::{InitFlag, LoadSurface};
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::video::Window;

use fps_counter::FPSCounter;

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Vel(f32);

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Pos(f32);

struct SysA;

impl<'a> System<'a> for SysA {
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.0 += vel.0;
            println!("{:?} pos, {:?} vel", pos, vel);
        }
    }
}

mod cp437;
use cp437::{Coords, Cp437};

const TILE_SIZE: (u32, u32) = (14, 16);
const CONSOLE_SIZE: (u32, u32) = (70, 30);
const WINDOW_SIZE: (u32, u32) = (1280, 720);

fn update_dstrect(dstrect: &mut Rect, (w, h): (u32, u32)) {
    let rat_w: f32 = w as f32 / WINDOW_SIZE.0 as f32;
    let rat_h: f32 = h as f32 / WINDOW_SIZE.1 as f32;
    if rat_w > rat_h {
        dstrect.w = (rat_h * WINDOW_SIZE.0 as f32) as i32;
        dstrect.h = h as i32;
        dstrect.x = ((w as i32 - dstrect.w) as f32 / 2f32) as i32;
        dstrect.y = 0;
    } else {
        dstrect.w = w as i32;
        dstrect.h = (rat_w * WINDOW_SIZE.1 as f32) as i32;
        dstrect.x = 0;
        dstrect.y = ((h as i32 - dstrect.h) as f32 / 2f32) as i32;
    }
}

fn randomize_tiles(
    canvas: &mut Canvas<Window>,
    frame_texture: &mut Texture,
    tiles_texture: &mut Texture,
) -> Result<(), String> {
    canvas
        .with_texture_canvas(frame_texture, |texture_canvas| {
            for x in 0..CONSOLE_SIZE.0 {
                for y in 0..CONSOLE_SIZE.1 {
                    let coords = Coords::from(Cp437::from(random::<u32>() % (Cp437::Count as u32)));
                    let srcrect = Rect::new(
                        (TILE_SIZE.0 as i32) * coords.row,
                        (TILE_SIZE.1 as i32) * coords.col,
                        TILE_SIZE.0,
                        TILE_SIZE.1,
                    );
                    let dstrect = Rect::new(
                        (x * TILE_SIZE.0) as i32,
                        (y * TILE_SIZE.1) as i32,
                        TILE_SIZE.0,
                        TILE_SIZE.1,
                    );
                    tiles_texture.set_color_mod(random::<u8>(), random::<u8>(), random::<u8>());
                    texture_canvas.set_draw_color(Color::RGBA(
                        random::<u8>() % 32u8,
                        random::<u8>() % 32u8,
                        random::<u8>() % 32u8,
                        255,
                    ));
                    texture_canvas
                        .fill_rect(Some(dstrect))
                        .expect("failed to draw rect");
                    texture_canvas
                        .copy(&tiles_texture, srcrect, dstrect)
                        .expect("failed to copy tile");
                }
            }
        })
        .map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));

    Ok(())
}

struct Tile {
    cp: Cp437,
    fg: Color,
    bg: Color,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            cp: Cp437::QuestionMark,
            fg: Color::RGBA(255, 0, 0, 255),
            bg: Color::RGBA(0, 0, 255, 255),
        }
    }
}

struct Console {
    width: u32,
    height: u32,
    tiles: Vec<Tile>,
}

impl Console {
    fn new(width: u32, height: u32) -> Self {
        let mut tiles = Vec::new();
        for _ in 0..(width * height) {
            tiles.push(Tile::default());
        }
        Self {
            width,
            height,
            tiles,
        }
    }
}

fn main() -> Result<(), String> {
    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new().with(SysA, "sys_a", &[]).build();
    dispatcher.setup(&mut world.res);
    world.create_entity().with(Vel(2.0)).with(Pos(0.0)).build();
    world.create_entity().with(Vel(4.0)).with(Pos(1.6)).build();
    world.create_entity().with(Vel(1.5)).with(Pos(5.4)).build();
    world.create_entity().with(Pos(2.0)).build();
    dispatcher.dispatch(&mut world.res);

    println!("{:?}", Coords::from(Cp437::from('G')));

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::JPG | InitFlag::PNG)?;
    let window = video_subsystem
        .window("rs_project", WINDOW_SIZE.0, WINDOW_SIZE.1)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .hidden()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .target_texture()
        .build()
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let tiles_surface = Surface::from_file(Path::new("res/cooz_14x16.png"))?;
    canvas.window_mut().set_icon(&tiles_surface);
    let mut tiles_texture = texture_creator
        .create_texture_from_surface(tiles_surface)
        .map_err(|e| e.to_string())?;

    let mut frame_texture = texture_creator
        .create_texture_target(
            PixelFormatEnum::RGBA8888,
            TILE_SIZE.0 * CONSOLE_SIZE.0,
            TILE_SIZE.1 * CONSOLE_SIZE.1,
        )
        .map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut dstrect = Rect::new(0, 0, 400, 300);
    let mut fps = FPSCounter::new();
    let mut dirty_window = false;

    canvas.window_mut().show();
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
    update_dstrect(&mut dstrect, canvas.window().size());
    randomize_tiles(&mut canvas, &mut frame_texture, &mut tiles_texture)?;

    'mainloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized { .. } | WindowEvent::SizeChanged { .. } => {
                        dirty_window = true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let keycodes: HashSet<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        if keycodes.contains(&Keycode::Escape) {
            break 'mainloop;
        }

        if keycodes.contains(&Keycode::Space) {
            randomize_tiles(&mut canvas, &mut frame_texture, &mut tiles_texture)?;
        }

        if dirty_window {
            update_dstrect(&mut dstrect, canvas.window().size());
            dirty_window = false;
        }

        canvas.clear();
        canvas.copy(&frame_texture, None, dstrect)?;
        canvas.present();

        fps.tick();
    }

    canvas.window_mut().hide();

    Ok(())
}
*/

use specs::prelude::*;

fn main() {
}
