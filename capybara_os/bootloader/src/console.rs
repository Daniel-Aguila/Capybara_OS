use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::text::Text;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::Pixel;
use embedded_graphics::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::boot::ScopedProtocol;

const BYTES_PER_PIXEL: usize = 4;

//https://blog.malware.re/2023/11/12/rust-os-part3/index.html for reference
pub struct Console{
    frame_buffer_ptr: *mut u8,
    //width in pixels
    frame_buffer_width: usize,
    //number of rows on screen
    frame_buffer_height: usize,
    //stride width of a framebuffer rows
    frame_buffer_stride: usize,
    //Pixel Format
    pixel_format: PixelFormat,
}

pub enum ConsoleError {
    BoundsError,
}

impl Console{
    //take Graphics Output and instatiante a new Console
    //based on mode info and frame buffer address
    pub fn new_from_uefi_gfx(mut gfx: ScopedProtocol<GraphicsOutput>) -> Self {
        let mode_info = gfx.current_mode_info();
        let (width, height) = mode_info.resolution();
        Console {
            frame_buffer_ptr: gfx.frame_buffer().as_mut_ptr(),
            frame_buffer_width: width,
            frame_buffer_height: height,
            frame_buffer_stride: mode_info.stride(),
            pixel_format: mode_info.pixel_format(),
        }
    }
    
    //writes string constant to (x,y)
    pub fn write_str<'a>(&mut self, str_to_write: &'a str, x: i32, y: i32) -> Result<(), ConsoleError> {
        //style of the display text 
        let style = MonoTextStyle::new(&FONT_6X10, Rgb888::YELLOW);

        Text::new(str_to_write, Point::new(x,y), style).draw(self)?;
        Ok(())
    }
}

impl OriginDimensions for Console { 
    fn size(&self) -> Size {
        Size::new(self.frame_buffer_width as u32, self.frame_buffer_height as u32)
    }
}

impl DrawTarget for Console {
    //testing by setting Color to Rgb888
    type Color = Rgb888;
    type Error = ConsoleError;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>
    {
        for Pixel(Point { x: px, y: py }, color) in pixels.into_iter() {
            let x = px as usize;
            let y = py as usize;
            if (x < self.frame_buffer_width) && (y < self.frame_buffer_height) {
                //Calculating offfset into the framebuffer
                let offset = (y * (self.frame_buffer_stride * BYTES_PER_PIXEL)) + (x * BYTES_PER_PIXEL);
                let frame_buffer_size = self.frame_buffer_stride * self.frame_buffer_height * BYTES_PER_PIXEL;
                let frame_buffer = unsafe { core::slice::from_raw_parts_mut(self.frame_buffer_ptr, frame_buffer_size) };
                //green channel
                frame_buffer[offset + 1] = color.g();

                //Support for swapped-ordering when we are a BGR versus RGB console
                if self.pixel_format == PixelFormat::Bgr {
                    frame_buffer[offset] = color.b();
                    frame_buffer[offset+2] = color.r();
                } else {
                    frame_buffer[offset] = color.r();
                    frame_buffer[offset+2] = color.b();
                }
            } else {
                    //If it's an invalid bound out of the 3 offset options, return Error
                    return Err(ConsoleError::BoundsError)
                }
            }
        Ok(())
    }
}
