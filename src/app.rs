use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use image::EncodableLayout;
use slint::Color;
use slint::ComponentHandle;
use slint::PlatformError;
use slint::SharedPixelBuffer;

use image;
use imageproc::geometric_transformations::Interpolation;
use imageproc::geometric_transformations::rotate_about_center;
use imageproc::map;
use slint::Weak;

use crate::ui::MainWindow;

const WHEEL_IMAGE_WIDTH: u32 = 300;
const WHEEL_IMAGE_HEIGHT: u32 = 300;

// const WHEEL_IMAGE_WIDTH: u32 = 300;
// const WHEEL_IMAGE_HEIGHT: u32 = 300;

struct HSVParams {
    saturation: f32,
    value: f32,
}

struct AppBackend {
    blended_image: SharedPixelBuffer<slint::Rgba8Pixel>,
    hsv_params: Arc<(Mutex<HSVParams>, Condvar)>,
}

impl AppBackend {
    fn new() -> Self {
        let hsv_params = Arc::new((
            Mutex::new(HSVParams {
                saturation: 1.0,
                value: 1.0,
            }),
            Condvar::new(),
        ));

        Self {
            blended_image: SharedPixelBuffer::new(WHEEL_IMAGE_WIDTH, WHEEL_IMAGE_HEIGHT),
            hsv_params,
        }
    }

    fn start(&mut self, window: &MainWindow) {
        let hsv_params = Arc::clone(&self.hsv_params);
        let mut blended_image = self.blended_image.clone();
        let window = window.as_weak();

        thread::spawn(move || {
            let mut hsv_saturation = 1.0;
            let mut hsv_value = 1.0;

            let (lock, cvar) = &*hsv_params;

            let final_blended_image = image::ImageBuffer::from_raw(
                blended_image.width(),
                blended_image.height(),
                Vec::from(blended_image.as_bytes()),
            );

            let final_blended_image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
                final_blended_image.unwrap();

            loop {
                {
                    let mut hsv_params = lock.lock().unwrap();

                    while hsv_saturation == hsv_params.saturation && hsv_value == hsv_params.value {
                        hsv_params = cvar.wait(hsv_params).unwrap();
                    }

                    hsv_saturation = hsv_params.saturation;
                    hsv_value = hsv_params.value;
                }

                let final_blended_image =
                    imageproc::map::map_pixels(&final_blended_image, |_x, _y, p| {
                        let mut c = Color::from_argb_u8(p[3], p[0], p[1], p[2]).to_hsva();
                        c.saturation = hsv_saturation;
                        c.value = hsv_value;
                        let c =
                            Color::from_hsva(c.hue, c.saturation, c.value, c.alpha).to_argb_u8();
                        image::Rgba([c.red, c.green, c.blue, c.alpha])
                    });

                blended_image
                    .make_mut_bytes()
                    .copy_from_slice(final_blended_image.as_bytes());
                let blended_image = blended_image.clone();

                let r = window.upgrade_in_event_loop(move |window| {
                    window
                        .global::<crate::ui::AppLogic>()
                        .set_color_image(slint::Image::from_rgba8(blended_image));
                });

                match r {
                    Err(_e) => {
                        println!("Failed to udate screen wheel image");
                    }
                    _ => {}
                }
            }
        });
    }

    fn set_hsv_saturation(&mut self, saturation: f32) {
        let (lock, cvar) = &*self.hsv_params;
        let mut hsv_params = lock.lock().unwrap();
        hsv_params.saturation = saturation;
        cvar.notify_one();
    }

    fn set_hsv_value(&mut self, value: f32) {
        let (lock, cvar) = &*self.hsv_params;
        let mut hsv_params = lock.lock().unwrap();
        hsv_params.value = value;
        cvar.notify_one();
    }

    fn update_color_image(&mut self, window: &Weak<MainWindow>) {
        let blended_image = self.blended_image.clone();
        let r = window.upgrade_in_event_loop(move |window| {
            window
                .global::<crate::ui::AppLogic>()
                .set_color_image(slint::Image::from_rgba8(blended_image));
        });

        match r {
            Err(_e) => {
                println!("Failed to udate screen wheel image");
            }
            _ => {}
        }
    }
}

pub struct App {
    pub window: MainWindow,
    backend: Rc<RefCell<AppBackend>>,
}

impl App {
    pub fn new() -> Self {
        let window = MainWindow::new().unwrap();
        let backend = Rc::new(RefCell::new(AppBackend::new()));

        {
            let backend = backend.clone();
            window
                .global::<crate::ui::AppLogic>()
                .on_hsv_saturation_changed(move |saturation| {
                    let mut backend = backend.borrow_mut();

                    backend.set_hsv_saturation(saturation / 100.0);
                    // backend.update_color_image(&window_weak);
                });
        }

        {
            let backend = backend.clone();
            window
                .global::<crate::ui::AppLogic>()
                .on_hsv_value_changed(move |value| {
                    let mut backend = backend.borrow_mut();

                    backend.set_hsv_value(value / 100.0);
                    // backend.update_color_image(&window_weak);
                });
        }

        window
            .global::<crate::ui::AppLogic>()
            .on_format_color(move |color| {
                slint::SharedString::from(format!("#{:02x}{:02x}{:02x}", color.red(), color.green(), color.blue()))
            });

        let backend = backend.clone();
        let mut app = Self { window, backend };

        app.create_color_images();
        app.backend.borrow_mut().start(&app.window);

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

        let gradient_img = image::imageops::resize(
            &gradient_img,
            WHEEL_IMAGE_WIDTH,
            WHEEL_IMAGE_HEIGHT,
            image::imageops::FilterType::Nearest,
        );

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

        let background_color = image::Rgba([0, 0, 0, 255]);
        let blended_image = imageproc::map::map_pixels(&gradient_img_1, |x, y, p1| {
            let p2: &image::Rgba<u8> = gradient_img_2.get_pixel(x, y);
            let p3 = gradient_img_3.get_pixel(x, y);

            let blend_pixel = |p1: &image::Rgba<u8>, p2: &image::Rgba<u8>| -> image::Rgba<u8> {
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
                let br = std::cmp::min((bs * als) / 255 + (bd * ald) / 255, 255);
                let alr = std::cmp::min((als * als) / 255 + (ald * ald) / 255, 255);

                image::Rgba([rr as u8, gr as u8, br as u8, alr as u8])
            };

            let pr = blend_pixel(&p1, &background_color);
            let pr = blend_pixel(&p2, &pr);
            blend_pixel(&p3, &pr)
        });

        // let backend = self.backend.as_ref();

        self.backend
            .borrow_mut()
            .blended_image
            .make_mut_bytes()
            .copy_from_slice(blended_image.as_bytes());

        let window_weak = self.window.as_weak();
        self.backend.borrow_mut().update_color_image(&window_weak);
    }

    pub fn run(&self) -> Result<(), PlatformError> {
        self.window.run()?;

        Result::Ok(())
    }
}
