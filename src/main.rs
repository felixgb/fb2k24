use sdl2;
use sdl2::event::Event;
use sdl2::mixer::{InitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS};
use sdl2::pixels::Color;
use sdl2::render::{TextureQuery, Texture, TextureCreator, Canvas};
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{WindowContext, Window};
use std::fs;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 480;

struct Columns<'a> {
    selected: u32,
    textures: Vec<(Texture<'a>, u32)>,
    line_height: u32,
    max_width: u32,
    total_height: u32,
}

impl<'a> Columns<'a> {

    fn init() -> Self {
        Columns {
            selected: 0,
            textures: Vec::new(),
            line_height: 0,
            max_width: 0,
            total_height: 0,
        }
    }

    fn visible_rows(&self) -> u32 {
        SCREEN_HEIGHT / self.line_height
    }

    fn top_visible(&self) -> usize {
        if self.selected > (self.visible_rows() / 2) {
            (self.selected - (self.visible_rows() / 2)) as usize
        } else {
            0
        }
    }

    fn selected_visible(&self) -> usize {
        self.selected as usize - self.top_visible()
    }

    fn add(&mut self, texture: Texture<'a>, width: u32, height: u32) {
        self.textures.push((texture, width));
        self.total_height += height;
        self.line_height = height;
        if self.max_width < width {
            self.max_width = width;
        }
    }

}

struct TextRenderContext<'a> {
    ttf_ctx: &'a Sdl2TtfContext,
    texture_creator: &'a TextureCreator<WindowContext>
}

impl<'a> TextRenderContext<'a> {

    pub fn text(&self, stuff: &str) -> Result<(Texture, (u32, u32)), String> {
        let font = self.ttf_ctx.load_font("TerminusTTF-4.49.3.ttf", 24)?;
        let surface = font
            .render(stuff)
            .solid(Color::RGBA(0, 255, 255, 255))
            .map_err(|e| e.to_string())?;

        let texture = self.texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        let TextureQuery { width, height, .. } = texture.query();
        Ok((texture, (width, height)))
    }

    pub fn make_text_block(&self, lines: &[String]) -> Columns {
        let mut c = Columns::init();
        for l in lines.iter() {
            let (texture, (width, height)) = self.text(l).unwrap();
            c.add(texture, width, height);
        }
        c
    }

}

fn draw_block<'a>(x_pos: u32, init_y_pos: u32, canvas: &mut Canvas<Window>, block: &Columns<'a>) {
    canvas.set_draw_color(Color::GREY);

    let to_render = &block.textures[block.top_visible()..];

    let rects = to_render.iter().scan(init_y_pos, |y_pos, (t, width)| {
        let r = Rect::new(x_pos as i32 + 2, *y_pos as i32, *width, block.line_height);
        *y_pos = *y_pos + block.line_height;
        Some((t, r))
    });

    for (i, (texture, rect)) in rects.enumerate() {
        if i == block.selected_visible() {
            let select_rect = Rect::new(rect.x - 2, rect.y, block.max_width + 4, rect.height());
            canvas.fill_rect(select_rect).unwrap();
        }
        canvas.copy(texture, None, Some(rect)).unwrap();
    }

    canvas.draw_rect(
        Rect::new(x_pos as i32, init_y_pos as i32, block.max_width + 4, block.total_height)
    ).unwrap();
}

fn init_music() -> Result<(), String> {
    let frequency = 44_100;
    let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1_024;

    sdl2::mixer::open_audio(frequency, format, channels, chunk_size)?;
    sdl2::mixer::init(InitFlag::MP3 | InitFlag::FLAC | InitFlag::MOD | InitFlag::OGG)?;
    Ok(())
}

fn go() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let _audio_subsys = sdl_context.audio()?;
    let ttf_ctx = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let music = sdl2::mixer::Music::from_file(
        "/home/fgb/music/A Journey through Italo Disco between '83 and '86 with DJ Subaru-Py0_uaQzflQ.mp3"
    )?;
    music.play(1)?;

    let window = video_subsys
        .window("fb2k24", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let text_thing = TextRenderContext { ttf_ctx: &ttf_ctx, texture_creator: &texture_creator };

    let (texture, (width, height)) = text_thing.text("woo")?;
    canvas.copy(&texture, None, Some(Rect::new(0, 0, width, height)))?;

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    let paths = fs::read_dir("/home/fgb/music/")
        .map_err(|e| e.to_string())?;

    let paths = paths.into_iter().map(
        |f| f.unwrap()
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
        ).collect::<Vec<String>>();

    let mut oks = text_thing.make_text_block(paths.as_slice());
    canvas.set_draw_color(Color::WHITE);

    'mainloop: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        draw_block(0, 0, &mut canvas, &oks);
        canvas.present();

        let event = sdl_context.event_pump()?.wait_event();
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::Q),
                ..
            }
            | Event::Quit { .. } => break 'mainloop,
            Event::KeyDown {
                keycode: Some(Keycode::K),
                ..
            } => oks.selected -= 1,
            Event::KeyDown {
                keycode: Some(Keycode::J),
                ..
            } => oks.selected += 1,
            _ => {}
        }
        // for event in sdl_context.event_pump()?.poll_iter() {
        //     match event {
        //         Event::KeyDown {
        //             keycode: Some(Keycode::Q),
        //             ..
        //         }
        //         | Event::Quit { .. } => break 'mainloop,
        //         Event::KeyDown {
        //             keycode: Some(Keycode::K),
        //             ..
        //         } => oks.selected -= 1,
        //         Event::KeyDown {
        //             keycode: Some(Keycode::J),
        //             ..
        //         } => oks.selected += 1,
        //         _ => {}
        //     }
        // }
        // thread::sleep(time::Duration::from_millis(10));
    }

    Ok(())
}

fn main() -> Result<(), String> {
    go()
}
