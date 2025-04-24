use image::EncodableLayout;
use slint::Color;
use slint::ComponentHandle;
use slint::PlatformError;
use slint::SharedPixelBuffer;

use image;
use imageproc::geometric_transformations::Interpolation;
use imageproc::geometric_transformations::rotate_about_center;
use imageproc::map;

use std::cmp::min;

// use crate::ui::AppLogic;
use crate::ui::MainWindow;

const WHEEL_IMAGE_WIDTH: u32 = 900;
const WHEEL_IMAGE_HEIGHT: u32 = 900;

pub struct App {
    pub window: MainWindow,
    // red_image: SharedPixelBuffer<slint::Rgba8Pixel>,
    // green_image: SharedPixelBuffer<slint::Rgba8Pixel>,
    // blue_image: SharedPixelBuffer<slint::Rgba8Pixel>,
    blended_image: SharedPixelBuffer<slint::Rgba8Pixel>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            window: MainWindow::new().unwrap(),
            // red_image: SharedPixelBuffer::new(WHEEL_IMAGE_WIDTH, WHEEL_IMAGE_HEIGHT),
            // green_image: SharedPixelBuffer::new(WHEEL_IMAGE_WIDTH, WHEEL_IMAGE_HEIGHT),
            // blue_image: SharedPixelBuffer::new(WHEEL_IMAGE_WIDTH, WHEEL_IMAGE_HEIGHT),
            blended_image: SharedPixelBuffer::new(WHEEL_IMAGE_WIDTH, WHEEL_IMAGE_HEIGHT),
        };

        app.create_color_images();
        app.update_color_image();

        app
    }

    fn map_image_color<I>(
        image: &I,
        c: slint::RgbaColor<u8>,
    ) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>>
    where
        I: image::GenericImage<Pixel = image::Rgba<u8>>,
    {
        let apply_color = |p: u8, c: u8| -> u8 {
            let p = p as u16;
            let c = c as u16;
            ((p * c) / 255) as u8
        };

        println!("Color: {} {} {} {}", c.red, c.blue, c.green, c.alpha);
        map::map_colors(image, |p: image::Rgba<u8>| {
            image::Rgba([
                apply_color(p[0], c.red),
                apply_color(p[1], c.blue),
                apply_color(p[2], c.green),
                apply_color(p[3], c.alpha),
            ])
        })
    }

    fn create_color_images(&mut self) {
        let gradient_img = image::open("ui/assets/images/gradient.png")
            .unwrap()
            .into_rgba8();

        let c1 = Color::from_hsva(120.0, 1.0, 1.0, 1.0).to_argb_u8();
        let c2 = Color::from_hsva(240.0, 1.0, 1.0, 1.0).to_argb_u8();
        let c3 = Color::from_hsva(0.0, 1.0, 1.0, 1.0).to_argb_u8();

        let gradient_img_1 = Self::map_image_color(&gradient_img, c1);
        let gradient_img_2 = Self::map_image_color(&gradient_img, c2);
        let gradient_img_3 = Self::map_image_color(&gradient_img, c3);

        let gradient_img_1 = rotate_about_center(
            &gradient_img_1,
            0.0_f32.to_radians(),
            Interpolation::Bicubic,
            image::Rgba([0, 0, 0, 0]),
        );
        let gradient_img_2 = rotate_about_center(
            &gradient_img_2,
            120.0_f32.to_radians(),
            Interpolation::Bicubic,
            image::Rgba([0, 0, 0, 0]),
        );
        let gradient_img_3 = rotate_about_center(
            &gradient_img_3,
            240.0_f32.to_radians(),
            Interpolation::Bicubic,
            image::Rgba([0, 0, 0, 0]),
        );

        let background_color = image::Rgba([0,0,0,255]);
        let blended_image = imageproc::map::map_pixels(&gradient_img_1, |x, y, p1| {
            let p2: &image::Rgba<u8> = gradient_img_2.get_pixel(x, y);
            let p3 = gradient_img_3.get_pixel(x, y);

            let blend_pixel = |p1 : &image::Rgba<u8>, p2 : &image::Rgba<u8>| -> image::Rgba<u8> {
                let rs = p1[0] as u16;
                let gs = p1[1] as u16;
                let bs = p1[2] as u16;
                let als = p1[3] as u16;

                let rd = p2[0] as u16;
                let gd = p2[1] as u16;
                let bd = p2[2] as u16;
                let ald = p2[3] as u16;

                // GL_ONE_MINUS_SRC_ALPHA
                // let rr = (rs * als) / 255 + (rd * (255 - als)) / 255;
                // let gr = (gs * als) / 255 + (gd * (255 - als)) / 255;
                // let br = (bs * als) / 255 + (bd * (255 - als)) / 255;
                // let alr = (als * als) / 255 + (ald * (255 - als)) / 255;

                // GL_DST_ALPHA
                let rr = std::cmp::min((rs * als) / 255 + (rd * ald) / 255, 255);
                let gr = std::cmp::min((gs * als) / 255 + (gd * ald) / 255, 255);
                let br = std::cmp::min((bs * als) / 255 + (bd * ald) / 255,255);
                let alr = std::cmp::min((als * als) / 255 + (ald * ald) / 255, 255);

                image::Rgba([rr as u8,gr as u8,br as u8,alr as u8])
            };

            let pr = blend_pixel(&p1, &background_color);
            let pr = blend_pixel(&p2, &pr);
            blend_pixel(&p3, &pr)
        });

        self.blended_image
            .make_mut_bytes()
            .copy_from_slice(blended_image.as_bytes());
    }

    pub fn update_color_image(&self) {
        self.window
            .global::<crate::ui::AppLogic>()
            .set_color_image(slint::Image::from_rgba8(self.blended_image.clone()));
    }

    pub fn run(&self) -> Result<(), PlatformError> {
        self.window.run()?;

        Result::Ok(())
    }
}
