
pub trait ChipDisplay {
    fn clear(&mut self);
    fn draw_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool;
}

pub struct MatrixDisplay {
    pub screen: [bool; 64*32]
}

impl MatrixDisplay {
    pub fn new() -> Self {
        Self { screen: [false; 64*32] }
    }
}

impl ChipDisplay for MatrixDisplay {
    fn clear(&mut self) {
        self.screen.fill(false)
    }

    fn draw_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        let mut flip = false;
        for sprite_row in sprite.chunks_exact(8) {
            let screen_row_start = x as usize + y as usize * 64;
            
            for i in 0..8 {
                let pixel = self.screen[screen_row_start + i] as u8;
                if pixel == 1 && sprite_row[i] == 1 {
                    flip = true
                }
                self.screen[screen_row_start + i] = (pixel ^ sprite_row[i]) != 0;
            }
        }
        flip
    }
}
