use std::cell::RefCell;
use std::convert::TryFrom;

use sdl2::{
    render::{TextureCreator, Texture},
    pixels::PixelFormatEnum, rect::Rect
};

use crate::{Error, game::{WORLD_SIZE, GameContext}, texture::TEXTURE_WIDTH};

enum TextureTarget {
    Render,
    Minimap
}

pub struct RenderContext {
    pub desired_frame_time: std::time::Duration,
    pub dev_fixed_wall_height: bool,
    pub dev_fixed_wall_height_value: u32,
    pub wall_height: u32,
    pub floor_colour: u32,
    pub ceil_colour: u32,
    pub show_minimap: bool,
    pub minimap_scale: u32,
    minimap_scale_px: u32,
    pub colour_mapping: fn(i32) -> u32
}

impl RenderContext {
    pub fn new() -> Self {

        let colour_mapping: fn(i32) -> u32 = |colour| match colour {
            1 => 0xff0000ff,
            2 => 0x00ff00ff,
            3 => 0x0000ffff,
            4 => 0xffff00ff,
            5 => 0x00ffffff,
            6 => 0xff00ffff,
            _ => 0xffffffff
        };

        Self {
            desired_frame_time: std::time::Duration::from_millis(16),
            dev_fixed_wall_height: false,
            dev_fixed_wall_height_value: 20,
            wall_height: 1,
            floor_colour: 0x505050ff,
            ceil_colour: 0x828282ff,
            show_minimap: true,
            minimap_scale: 6,
            minimap_scale_px: 0,
            colour_mapping
        }
    }
}

pub struct Renderer {
    pub render_context: RenderContext,
    sdl_canvas: sdl2::render::WindowCanvas,
    creator: TextureCreator<sdl2::video::WindowContext>,
    image_textures: Vec<Vec<u32>>,
    render_texture: RefCell<Texture<'static>>,
    minimap_texture: RefCell<Texture<'static>>,
    // RefCell allows for multiple immutable borrows to be used internally
    render_data: Vec<u32>, minimap_data: Vec<u32>,
    width: u32, height: u32
}

impl Renderer {
    pub fn new(sdl_context: &sdl2::Sdl, title: &'static str, width: u32, height: u32) -> Result<Self, Error> {
        let video_ss = sdl_context.video()?;
        let window = video_ss.window(title, width, height)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        let sdl_canvas = window.into_canvas().build()?;
        let creator = sdl_canvas.texture_creator();
        let render_context = RenderContext::new();

        let minimap_size = u32::try_from(render_context.minimap_scale as usize * WORLD_SIZE)?;

        let render_texture = creator.create_texture_target(
            PixelFormatEnum::RGBA8888, width, height)?;
        let minimap_texture = creator.create_texture_target(
                PixelFormatEnum::RGBA8888, minimap_size, minimap_size)?;

        // Reinterpret the bits of `render_texture` and
        // `minimap_texture` as a `Texture<'static>`
        let (render_texture, minimap_texture) = unsafe {
            (std::mem::transmute::<_,Texture<'static>>(render_texture),
             std::mem::transmute::<_,Texture<'static>>(minimap_texture))
        };

        let image_textures = crate::texture::load_textures();

        Ok(Self {
            render_context,
            sdl_canvas, creator,
            image_textures,
            render_texture: RefCell::new(render_texture),
            minimap_texture: RefCell::new(minimap_texture),
            width, height,
            render_data: vec![0; (width*height) as usize],
            minimap_data: vec![0; (minimap_size) as usize]
        })
    }

    pub fn draw(&mut self, game_context: &GameContext) -> Result<(), Error> {
        self.render_context.minimap_scale_px = u32::try_from(self.render_context.minimap_scale as usize * WORLD_SIZE)?;
        self.clear(&TextureTarget::Minimap);
        self.draw_background();
        self.draw_world(game_context)?;
        if self.render_context.show_minimap {
            self.draw_minimap_cells(game_context);
            self.draw_player_on_minimap(game_context);
        }
        self.swap()?;
        Ok(())
    }

    fn swap(&mut self) -> Result<(), Error> {
        let mut render_texture = self.render_texture.borrow_mut();

        render_texture.update(None, self.data_raw(TextureTarget::Render),
            (self.width*4) as usize)?;

        self.sdl_canvas.copy(&render_texture,None,None)?;

        if self.render_context.show_minimap {
            let mut minimap_texture = self.minimap_texture.borrow_mut();

            minimap_texture.update(None, self.data_raw(TextureTarget::Minimap),
                (self.render_context.minimap_scale_px*4) as usize)?;

            let minimap_pos_x = (self.width-self.render_context.minimap_scale_px-10) as i32;
            let minimap_pos_y = (self.height-self.render_context.minimap_scale_px-10) as i32;
            self.sdl_canvas.copy(&minimap_texture, None,
                Rect::new(minimap_pos_x, minimap_pos_y,
                    self.render_context.minimap_scale_px, self.render_context.minimap_scale_px)
            )?;
        }

        self.sdl_canvas.present();

        Ok(())
    }

    fn data_raw(&self, target: TextureTarget) -> &[u8] {
        let data = match target {
            TextureTarget::Render => &self.render_data,
            TextureTarget::Minimap => &self.minimap_data
        };

        unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len()*4
            )
        }
    }

    fn set_pixel(&mut self, target: &TextureTarget, x: u32, y: u32, colour: u32) {

        let (max_x, max_y) = match target {
            TextureTarget::Render => (self.width - 1, self.height - 1),
            TextureTarget::Minimap => (self.render_context.minimap_scale_px - 1, self.render_context.minimap_scale_px - 1)
        };

        if x  > max_x || y > max_y {
            //println!("Warning: attempted to write to pixel {}, {}, which is outside of the texture", x, y);
            return
        }

        match target {
            TextureTarget::Render => self.render_data[(y * self.width + x) as usize] = colour,
            TextureTarget::Minimap => self.minimap_data[(y * self.render_context.minimap_scale_px + x) as usize] = colour
        }
    }

    fn draw_line(&mut self, target: &TextureTarget, x0: i32, y0: i32, x1: i32, y1: i32, thickness: i32, colour: u32) {

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x0;
        let mut y = y0;

        let half_thickness = thickness / 2;

        // Plot the initial point
        for i in 0..thickness {
            for j in 0..thickness {
                self.set_pixel(target, (x + i - half_thickness) as u32, (y + j - half_thickness) as u32, colour);
            }
        }

        // Bresenham's line algorithm
        while x != x1 || y != y1 {
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }

            // Draw the line with the specified thickness
            for i in 0..thickness {
                for j in 0..thickness {
                    self.set_pixel(target, (x + i - half_thickness) as u32, (y + j - half_thickness) as u32, colour);
                }
            }
        }
    }

    fn draw_filled_circle(&mut self, target: &TextureTarget, center_x: u32, center_y: u32, radius: u32, colour: u32) {
        // Helper function to set a pixel if it is within the bounds of the window
        let mut set_pixel_if_in_bounds = |x: i32, y: i32| {
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                self.set_pixel(target, x as u32, y as u32, colour);
            }
        };

        // Midpoint circle algorithm to draw the filled circle
        let mut x = radius as i32;
        let mut y = 0;
        let mut radius_error = 1 - x;

        while x >= y {
            for i in -x..=x {
                set_pixel_if_in_bounds(center_x as i32 + i, center_y as i32 + y);
                set_pixel_if_in_bounds(center_x as i32 + i, center_y as i32 - y);
            }
            for i in -y..=y {
                set_pixel_if_in_bounds(center_x as i32 + i, center_y as i32 + x);
                set_pixel_if_in_bounds(center_x as i32 + i, center_y as i32 - x);
            }

            y += 1;

            if radius_error < 0 {
                radius_error += 2 * y + 1;
            } else {
                x -= 1;
                radius_error += 2 * (y - x + 1);
            }
        }
    }

    fn draw_background(&mut self) {
        for x in 0..self.width {
            for y in 0..(self.height/2) {
                self.set_pixel(&TextureTarget::Render, x, y, self.render_context.ceil_colour)
            }
        }

        for x in 0..self.width {
            for y in (self.height/2)..self.height {
                self.set_pixel(&TextureTarget::Render, x, y, self.render_context.floor_colour)
            }
        }
    }

    fn draw_world(&mut self, context: &GameContext) -> Result<(), String> {

        let minimap_cell_size = self.render_context.minimap_scale_px as usize / WORLD_SIZE;

        let pos_x = context.player.position.x;
        let pos_y = context.player.position.y;

        let minimap_scaled_pos_x = (pos_y*minimap_cell_size as f32).max(0.0) as u32;
        let minimap_scaled_pos_y = (pos_x*minimap_cell_size as f32).max(0.0) as u32;

        for column_index in 0..self.width {

            let camera_x: f32 = ((2 * column_index) as f32 / self.width as f32) - 1.0;

            let ray_dir_x = context.player.camera_direction.x + context.player.camera_plane.x * camera_x as f32;
            let ray_dir_y = context.player.camera_direction.y + context.player.camera_plane.y * camera_x as f32;

            let mut map_x = pos_x as i32;
            let mut map_y = pos_y as i32;

            let delta_dist_x = if ray_dir_x == 0.0 {f32::MAX} else {(1.0/ray_dir_x).abs()};
            let delta_dist_y = if ray_dir_y == 0.0 {f32::MAX} else {(1.0/ray_dir_y).abs()};

            let step_x: i32;
            let step_y: i32;

            let mut side_dist_x: f32;
            let mut side_dist_y: f32;

            if ray_dir_x < 0.0 {
                step_x = -1;
                side_dist_x = (pos_x - map_x as f32) * delta_dist_x
            } else {
                step_x = 1;
                side_dist_x = ((map_x + 1) as f32 - pos_x) * delta_dist_x;
            }
            
            if ray_dir_y < 0.0 {
                step_y = -1;
                side_dist_y = (pos_y - map_y as f32) * delta_dist_y;
            } else {
                step_y = 1;
                side_dist_y = ((map_y + 1) as f32 - pos_y) * delta_dist_y;
            }

            let mut hit = false;
            let mut side_facing_x_axis = false;

            while !hit {
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    side_facing_x_axis = true;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    side_facing_x_axis = false;
                }

                if map_x < 0 || map_y < 0 {
                    todo!("Implement proper handling for rays which are out of bounds");
                }

                if context.world.get(map_x as usize).unwrap().get(map_y as usize).unwrap() > &0 {
                    hit = true;
                }
            }

            let perpendicular_wall_dist: f32;

            if side_facing_x_axis {
                perpendicular_wall_dist = side_dist_x - delta_dist_x;
            } else {
                perpendicular_wall_dist = side_dist_y - delta_dist_y;
            }
            
            let ray_intersection_x: f32;
            let ray_intersection_y: f32;

            // Calculate the exact x, y coordinates the ray intersects with
            if side_facing_x_axis && step_x == -1 {
                ray_intersection_x = (map_x+1) as f32;
                ray_intersection_y = pos_y + perpendicular_wall_dist * ray_dir_y;
            }
            else if side_facing_x_axis && step_x == 1 {
                ray_intersection_x = map_x as f32;
                ray_intersection_y = pos_y + perpendicular_wall_dist * ray_dir_y;
            }
            else if !side_facing_x_axis && step_y == -1 {
                ray_intersection_x = pos_x + perpendicular_wall_dist * ray_dir_x;
                ray_intersection_y = (map_y+1) as f32;
            }
            else if !side_facing_x_axis && step_y == 1 {
                ray_intersection_x = pos_x + perpendicular_wall_dist * ray_dir_x;
                ray_intersection_y = map_y as f32;
            }
            // It shouldn't be possible for all of the above to return false,
            // but just in case that happens, just set the intersection coordinates to the origin of the ray
            else {
                ray_intersection_x = pos_x;
                ray_intersection_y = pos_y;
            }

            if column_index % 5 == 0  && self.render_context.show_minimap {

                self.draw_line(
                    &TextureTarget::Minimap,

                    (ray_intersection_y * minimap_cell_size as f32) as i32,
                    (ray_intersection_x * minimap_cell_size as f32) as i32,

                    minimap_scaled_pos_x as i32,
                    minimap_scaled_pos_y as i32,

                    2,
                    0xFFFFFFFF
                );

            }
            
            let line_height: u32;

            if self.render_context.dev_fixed_wall_height {
                line_height = self.render_context.dev_fixed_wall_height_value;
            } else {
                line_height = ((self.height * self.render_context.wall_height) as f32 / perpendicular_wall_dist) as u32;
            }  

            let draw_start = std::cmp::max(
                (-(line_height as i32) / 2) + ((self.height / 2) as i32),
                0
            ) as u32;
            let draw_end = std::cmp::min(
                ((line_height as i32) / 2) + ((self.height / 2) as i32),
                (self.height - 1) as i32
            ) as u32;

            let step = TEXTURE_WIDTH as f32 / line_height as f32;
            let texture_index = context.world[map_x as usize][map_y as usize] as usize - 1;
            let wall_x;

            if side_facing_x_axis {
                wall_x = ray_intersection_y - ray_intersection_y.floor()
            } else {
                wall_x = ray_intersection_x - ray_intersection_x.floor()
            }

            let texture_x = TEXTURE_WIDTH as i32 - (wall_x * TEXTURE_WIDTH as f32) as i32 - 1;

            let mut texture_y = (((draw_start as i32 - (self.height / 2) as i32) + (line_height / 2) as i32) as f32 * step).max(0.0);

            for y in draw_start..draw_end {

                let mut colour = self.image_textures[texture_index][TEXTURE_WIDTH as usize * texture_y as usize + texture_x as usize];

                texture_y += step;

                if !side_facing_x_axis {
                    // Extract the r,g,b,a components and
                    // right shift the r,g,b values to half them
                    let (r, g, b, a) = (
                        colour >> 24 + 1,
                        (colour >> 16 & 0xFF) >> 1,
                        (colour >> 8 & 0xFF) >> 1,
                        colour & 0xFF
                    );
                    
                    // Shift the values into their correct place
                    // And combine them
                    colour = r << 24 | g << 16 | b << 8 | a;
                }

                self.set_pixel(&TextureTarget::Render, column_index, y, colour)

            }

            /*
            let mut colour = (self.render_context.colour_mapping)(context.world[map_x as usize][map_y as usize]);

            // Half the brightness of the pixels if the side
            // of the wall they occupy is facing the x axis
            if !side_facing_x_axis {
                // Extract the r,g,b,a components and
                // right shift the r,g,b values to half them
                let (r, g, b, a) = (
                    colour >> 24 + 1,
                    (colour >> 16 & 0xFF) >> 1,
                    (colour >> 8 & 0xFF) >> 1,
                    colour & 0xFF
                );
                
                // Shift the values into their correct place
                // And combine them
                colour = r << 24 | g << 16 | b << 8 | a;
            }

            for y in draw_start..draw_end {
                self.set_pixel(&TextureTarget::Render, column_index, y, colour)
            }
            */
            
        }

        Ok(())
    }

    fn draw_minimap_cells(&mut self, game_context: &GameContext) {
        
        let minimap_cell_size = self.render_context.minimap_scale_px as usize / WORLD_SIZE;

        for y in 0..WORLD_SIZE {
            for (x, cell) in game_context.world[y].iter().enumerate() {

                let cell_colour = (self.render_context.colour_mapping)(*cell);

                let scaled_x = x * minimap_cell_size;
                let scaled_y = y * minimap_cell_size;

                if *cell != 0 {
                    for sub_x in scaled_x..scaled_x+minimap_cell_size {
                        for sub_y in scaled_y..scaled_y+minimap_cell_size {
                            self.set_pixel(
                                &TextureTarget::Minimap,
                                sub_x as u32, sub_y as u32,
                                cell_colour
                            )
                        }
                    }
                }
            
                
            }
        }

        /*
        // Example of how to write a texture as if it was a texture object
        self.sdl_canvas.with_texture_canvas(&mut self.minimap_texture.borrow_mut(), |texture_canvas| {

            let pixel_format = PixelFormat::try_from(PixelFormatEnum::RGBA8888);

            if pixel_format.is_ok() {

                let pixel_format = &pixel_format.unwrap();

                for y in 0..game_context.world.len() {
                    for (x, cell) in game_context.world[y].iter().enumerate() {
                        let cell_colour = (self.render_context.colour_mapping)(*cell);
                        texture_canvas.set_draw_color(Color::from_u32(pixel_format, cell_colour));
                        let _ = texture_canvas.fill_rect(Rect::new(
                            (x*minimap_cell_size_x) as i32, (y*minimap_cell_size_y) as i32,
                            minimap_cell_size_x as u32, minimap_cell_size_y as u32
                        ));
                    }
                }

            }
        
        })?;
        */
    }

    fn draw_player_on_minimap(&mut self, game_context: &GameContext) {

        let minimap_cell_size = self.render_context.minimap_scale_px as usize / WORLD_SIZE;

        let minimap_scaled_pos_x = (game_context.player.position.y*minimap_cell_size as f32).max(0.0) as u32;
        let minimap_scaled_pos_y = (game_context.player.position.x*minimap_cell_size as f32).max(0.0) as u32;

        let minimap_scaled_dir_x = minimap_scaled_pos_x as i32 + (game_context.player.camera_direction.y*minimap_cell_size as f32 * 3.0) as i32;
        let minimap_scaled_dir_y = minimap_scaled_pos_y as i32 + (game_context.player.camera_direction.x*minimap_cell_size as f32 * 3.0) as i32;

        // Draw line representing the direction (look) vector
        self.draw_line(
            &TextureTarget::Minimap,
            minimap_scaled_pos_x as i32, minimap_scaled_pos_y as i32,
            minimap_scaled_dir_x, minimap_scaled_dir_y,
            2, 0x00FF00FF
        );

        // Draw small circle on map representing player
        self.draw_filled_circle(
            &TextureTarget::Minimap,
            minimap_scaled_pos_x, minimap_scaled_pos_y,
            3, 0xFF0000FF
        );

    }

    fn clear(&mut self, target: &TextureTarget) {
        match target {
            TextureTarget::Minimap
            => {self.minimap_data = vec![0; (self.render_context.minimap_scale_px.pow(2)) as usize]}
            TextureTarget::Render
            => {self.render_data = vec![0; (self.width*self.height) as usize]}
        }
    }

}