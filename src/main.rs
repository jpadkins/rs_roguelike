use std::collections::HashSet;
use std::path::Path;
use std::time::{Duration, Instant};

use rand::prelude::*;

use specs::prelude::*;
use specs_derive::{Component, ConvertSaveload};

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

mod cp437;
use cp437::{Coords, Cp437};

const TILE_SIZE: (u32, u32) = (14, 16);
const CONSOLE_SIZE: (u32, u32) = (140, 60);
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

fn draw_tile(
    canvas: &mut Canvas<Window>,
    frame_texture: &mut Texture,
    tiles_texture: &mut Texture,
    tile: &Tile,
) -> Result<(), String> {
    canvas
        .with_texture_canvas(frame_texture, |texture_canvas| {
            let coords = Coords::from(tile.code_point);
            let srcrect = Rect::new(
                (TILE_SIZE.0 as i32) * coords.row,
                (TILE_SIZE.1 as i32) * coords.col,
                TILE_SIZE.0,
                TILE_SIZE.1,
            );
            let dstrect = Rect::new(
                (tile.row * TILE_SIZE.0) as i32,
                (tile.col * TILE_SIZE.1) as i32,
                TILE_SIZE.0,
                TILE_SIZE.1,
            );
            let Color { r, g, b, .. } = tile.foreground;
            tiles_texture.set_color_mod(r, g, b);
            texture_canvas.set_draw_color(tile.background);
            texture_canvas
                .fill_rect(Some(dstrect))
                .expect("failed to draw rect");
            texture_canvas
                .copy(&tiles_texture, srcrect, dstrect)
                .expect("failed to copy tile");
        })
        .map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));

    Ok(())
}

#[derive(Debug, Copy, Clone)]
enum Animation {
    Blink(f32),
    VerticalShift,
    HorizontalShift,
    ColorShift(f32, Color, Color),
}

#[derive(Debug, Clone)]
struct Tile {
    row: u32,
    col: u32,
    code_point: Cp437,
    foreground: Color,
    background: Color,
    dirty: bool,
    animations: Vec<Animation>,
}

impl Tile {
    pub fn dirty(&self) -> bool {
        self.dirty
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            row: 0,
            col: 0,
            code_point: Cp437::QuestionMark,
            foreground: Color::RGBA(255, 0, 0, 255),
            background: Color::RGBA(0, 0, 255, 255),
            dirty: true,
            animations: vec![],
        }
    }
}

#[derive(Debug, Default)]
struct Console {
    width: u32,
    height: u32,
    tiles: Vec<Tile>,
}

impl Console {
    pub fn new(width: u32, height: u32) -> Self {
        let mut tiles = Vec::new();
        for col in 0..height {
            for row in 0..width {
                tiles.push(Tile {
                    row,
                    col,
                    ..Default::default()
                })
            }
        }
        Self {
            width,
            height,
            tiles,
        }
    }
    /*
        // TODO: pass by reference?
        pub fn dirty_tiles(&self) -> impl Iterator<Item = Tile> {
            self.tiles.into_iter().filter(|t| t.dirty)
        }
    */
    pub fn tiles(&self) -> &Vec<Tile> {
        &self.tiles
    }

    pub fn tiles_mut(&mut self) -> &mut Vec<Tile> {
        &mut self.tiles
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn reset_tiles(&mut self) {
        for t in &mut self.tiles {
            t.dirty = false;
        }
    }

    pub fn tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        if x > self.width || y > self.height {
            return None;
        }
        let index = self.index(x, y);
        Some(&mut self.tiles[index])
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (x + (y * self.width)) as usize
    }
}

use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::{EventPump, Sdl};

/*
struct Sdl2System<'r> {
    sdl: Sdl,
    canvas: WindowCanvas,
    texture_creator: &'r TextureCreator<WindowContext>,
    tiles_texture: Texture<'r>,
    frame_texture: Texture<'r>,
    event_pump: EventPump,
    dstrect: Rect,
}

impl<'a, 'r> System<'a> for Sdl2System<'r> {
    type SystemData = (Read<'a, Console>,
                       Entities<'a>,
                       ReadStorage<'a, Pos>);

    fn run(&mut self, (con, ent, pos): Self::SystemData) {
        println!("{:?}", *con);

        println!("Listing Entities:");
        for (ent, pos) in (&ent, &pos).join() {
            println!("{:?}", ent);
        }
    }
}
*/

#[derive(Debug, Default)]
struct State {
    quit: bool,
    randomize: bool,
}

#[derive(Debug, Default)]
struct PressedKeycodes(HashSet<Keycode>);

struct SysA;

impl<'a> System<'a> for SysA {
    type SystemData = (Read<'a, PressedKeycodes>, Write<'a, State>);

    fn run(&mut self, data: Self::SystemData) {
        let (keycodes, mut state) = data;

        state.quit = keycodes.0.contains(&Keycode::Escape);
        state.randomize = keycodes.0.contains(&Keycode::Space);
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::JPG | InitFlag::PNG)?;
    let window = video_subsystem
        .window("rs_project", WINDOW_SIZE.0, WINDOW_SIZE.1)
        .position_centered()
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
    let mut dstrect = Rect::new(0, 0, 0, 0);
    let mut fps = FPSCounter::new();
    let mut dirty_window = false;

    canvas.window_mut().show();
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));

    let mut world = World::new();
    world.register::<Vel>();
    world.register::<Pos>();

    world.insert(Console::new(CONSOLE_SIZE.0, CONSOLE_SIZE.1));
    world.insert(State {
        quit: false,
        randomize: false,
    });
    world.insert(PressedKeycodes);

    println!("{:?}", Coords::from(Cp437::from('G')));

    let mut dispatcher = DispatcherBuilder::new().with(SysA, "sys_a", &[]).build();

    dispatcher.setup(&mut world);
    world.create_entity().with(Vel(2.0)).with(Pos(0.0)).build();
    world.create_entity().with(Vel(4.0)).with(Pos(1.6)).build();
    world.create_entity().with(Vel(1.5)).with(Pos(5.4)).build();
    world.create_entity().with(Pos(2.0)).build();
    dispatcher.dispatch(&mut world);

    update_dstrect(&mut dstrect, canvas.window().size());

    let mut last_fps_print = Instant::now();

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
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
        *world.fetch_mut::<PressedKeycodes>() = PressedKeycodes(keycodes);

        // Update user input
        dispatcher.dispatch(&mut world);
        world.maintain();

        let state = world.fetch::<State>();
        let mut console = world.fetch_mut::<Console>();

        use rayon::prelude::*;

        if state.randomize {
            console.tiles_mut().par_iter_mut().for_each(|tile| {
                if (random::<u32>() % 10) != 0 {
                    return;
                }
                tile.code_point = Cp437::from(random::<u32>() % (Cp437::Count as u32));
                tile.foreground = Color::RGBA(random::<u8>(), random::<u8>(), random::<u8>(), 255);
                tile.background = Color::RGBA(
                    random::<u8>() % 32u8,
                    random::<u8>() % 32u8,
                    random::<u8>() % 32u8,
                    255,
                );
                tile.dirty = true;
            });
        }

        if state.quit {
            break 'main;
        }

        for tile in console.tiles() {
            if tile.dirty() {
                draw_tile(&mut canvas, &mut frame_texture, &mut tiles_texture, &tile)?;
            }
        }

        console.reset_tiles();

        if dirty_window {
            update_dstrect(&mut dstrect, canvas.window().size());
            dirty_window = false;
        }

        canvas.clear();
        canvas.copy(&frame_texture, None, dstrect)?;
        canvas.present();

        if Instant::now() - last_fps_print > Duration::new(5, 0) {
            println!("fps: {}", fps.tick());
            last_fps_print = Instant::now();
        } else {
            fps.tick();
        }
    }

    canvas.window_mut().hide();

    Ok(())
}

/*
fn main() -> Result<(), String> {
    use hyphenation::{Language, Load, Standard};
    use slog::{debug, error, info, o, warn, Drain, Logger};
    use slog_async::Async;
    use slog_term::{CompactFormat, TermDecorator};
    use textwrap::Wrapper;

    let decorator = TermDecorator::new().build();
    let drain = CompactFormat::new(decorator).build().fuse();
    let drain = Async::new(drain).build().fuse();
    let log = Logger::root(drain, o!());

    let hyphenator = Standard::from_embedded(Language::EnglishUS).map_err(|e| e.to_string())?;
    let wrapper = Wrapper::with_splitter(20, hyphenator);

    debug!(log, "Logging ready!");
    info!(log, "Logging ready!");
    warn!(log, "Logging ready!");
    error!(log, "Logging ready!");

    println!(
        "{}",
        wrapper.fill("This is a fairly long line. I wonder how textwrap will handle it?")
    );

    Ok(())
}
*/

/*
#[derive(PartialEq)]
enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

#[derive(PartialEq)]
enum Input {
    Accept,
    Decline,
    Exit,
    Direction(Direction),
}

trait Scene {
    fn update(self: Box<Self>, input: Input) -> Box<Scene>;
}

struct SceneA {
    switch_input: Input,
}

impl Scene for SceneA {
    fn update(self: Box<Self>, input: Input) -> Box<Scene> {
        if input == self.switch_input {
            println!("SceneA: Switching to SceneB!");
            Box::new(SceneB {
                switch_input: Input::Direction(Direction::N),
            })
        } else {
            self
        }
    }
}

impl Drop for SceneA {
    fn drop(&mut self) {
        println!("dropping SceneA!");
    }
}

struct SceneB {
    switch_input: Input,
}

impl Scene for SceneB {
    fn update(self: Box<Self>, input: Input) -> Box<Scene> {
        if input == self.switch_input {
            println!("SceneB: Switching to SceneA!");
            Box::new(SceneA {
                switch_input: Input::Decline,
            })
        } else {
            self
        }
    }
}

impl Drop for SceneB {
    fn drop(&mut self) {
        println!("dropping SceneB!");
    }
}

fn main() -> Result<(), String> {
    let mut scene: Box<Scene> = Box::new(SceneA {
        switch_input: Input::Decline,
    });
    use std::io::Read;

    'main: loop {
        let c: char = std::io::stdin()
            .bytes()
            .next()
            .and_then(|r| r.ok())
            .map(|b| b as char)
            .ok_or("IO Err")?;

        match c {
            '\n' => {}
            '\u{1b}' => break 'main,
            'a' => {
                scene = scene.update(Input::Direction(Direction::N));
            }
            'b' => {
                scene = scene.update(Input::Decline);
            }
            'c' => {
                scene = scene.update(Input::Direction(Direction::S));
            }
            c => println!("{:?}", c),
        }
    }

    Ok(())
}
*/
