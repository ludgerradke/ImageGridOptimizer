use std::fs;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba};
use image::imageops;
use image::{imageops::resize};
use image::imageops::FilterType;

/// Adds a white border around the given image.
///
/// # Parameters
///
/// - `img`: The image to which the border will be added.
/// - `border_size`: The size of the border to be added.
///
/// # Returns
///
/// A new image with the added border.
fn add_white_border(img: &DynamicImage, border_size: u32) -> DynamicImage {
    let (width, height) = img.dimensions();
    let new_width = width + 2 * border_size;
    let new_height = height + 2 * border_size;

    let mut new_img = DynamicImage::new_rgba8(new_width, new_height);

    // Fill the entire image with white color
    for y in 0..new_height {
        for x in 0..new_width {
            new_img.put_pixel(x, y, image::Rgba([255u8, 255u8, 255u8, 255u8]));
        }
    }

    // Copy the original image to the center of the new image
    imageops::overlay(&mut new_img, img, border_size as i64, border_size as i64);

    new_img
}

/// Loads images from a directory with an optional filter.
///
/// # Parameters
///
/// - `dir`: The directory containing the images.
/// - `filter`: An optional filter for image extensions or filenames.
///
/// # Returns
///
/// A vector of loaded images.
fn load_images(dir: &str, filter: Option<String>) -> Vec<DynamicImage> {
    const BORDER_SIZE: u32 = 5; // Size of the white border

    fs::read_dir(dir)
        .expect("Failed to read directory")
        .filter_map(|entry| {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_file() && (filter.is_none()
                || path.extension().and_then(|s| s.to_str()).map_or(false, |ext| ext == filter.as_ref().unwrap())) {
                let mut img = image::open(&path).expect("Failed to open image");
                img = DynamicImage::from(scale_to_standard_width(img, 500));
                Some(add_white_border(&img, BORDER_SIZE))
            } else {
                None
            }
        })
        .collect()
}

/// Creates a collage from a vector of images.
///
/// # Parameters
///
/// - `images`: A vector of images to be used in the collage.
///
/// # Returns
///
/// A single image representing the collage.
fn create_collage(mut images: Vec<DynamicImage>) -> DynamicImage {
    let mode = "area";
    if mode == "area" {
        images.sort_by(|a, b| {
            let area_a = a.dimensions().0 * a.dimensions().1;
            let area_b = b.dimensions().0 * b.dimensions().1;
            area_b.cmp(&area_a)
        });
    }
    else {
        images.sort_by(|a, b| {
            let width_a = a.width();
            let width_b = b.width();
            width_b.cmp(&width_a)
        });
    }


    let first_image = images.remove(0);
    let mut collage = first_image;

    let mut count = 1;
    for img in images {
        collage = place_image(collage, img);
        collage.save(format!("collage_step_{}.png", count)).unwrap();
        println!("{}", count);
        count += 1;
    }

    collage
}

/// Places a new image onto a collage.
///
/// # Parameters
///
/// - `collage`: The existing collage.
/// - `new_image`: The new image to be placed on the collage.
///
/// # Returns
///
/// A new collage with the new image placed.
fn place_image(mut collage: DynamicImage, new_image: DynamicImage) -> DynamicImage {
    let (width, height) = collage.dimensions();
    let (new_width, new_height) = new_image.dimensions();
    let mut min_width = width;
    let mut min_height = height;
    let mut min_scope = new_width * new_height;
    let mut found = false;
    let mut boundary = false;

    for y in 0..height {
        for x in 0..width {
            boundary = false;
            let pixel = collage.get_pixel(x, y);
            if pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0 {
                continue;
            }
            // Check the neighbors
            let neighbors = [
                (x.saturating_sub(1), y), // Left
                (x + 1, y),               // Right
                (x, y.saturating_sub(1)), // Above
                (x, y + 1),               // Below
            ];

            for &(nx, ny) in &neighbors {
                if nx < width && ny < height {
                    let neighbor_pixel = collage.get_pixel(nx, ny);
                    if neighbor_pixel[0] == 255 && neighbor_pixel[1] == 255 && neighbor_pixel[2] == 255 {
                        boundary = true;
                    }
                }
            }
            if !boundary{
                continue
            }
            if is_empty_space(&collage, x, y, new_width, new_height) {
                if x + new_width <= width && y + new_height <= height {
                    collage.copy_from(&new_image, x, y).unwrap();
                    return collage
                }
                let mut tmp_width = x + new_width + 1;
                let mut tmp_height = y + new_height + 1;
                if tmp_width < width{
                    tmp_width = width;
                }
                if tmp_height < height{
                    tmp_height = height;
                }
                let scope_delta = (tmp_height * tmp_width) - (width * height);
                if scope_delta < min_scope{
                    min_width = tmp_width;
                    min_height = tmp_height;
                    found = true;
                    min_scope = scope_delta;
                }
            }
        }
    }
    if found {
        let mut new_collage = DynamicImage::new_rgb8(min_width, min_height);
        new_collage.copy_from(&collage, 0, 0).unwrap();
        place_image(new_collage, new_image)
    } else {
        if width > height {
            let mut new_collage = DynamicImage::new_rgb8(width, height + new_height);
            new_collage.copy_from(&collage, 0, 0).unwrap();
            return place_image(new_collage, new_image)
        }
        else {
            let mut new_collage = DynamicImage::new_rgb8(width + new_width, height);
            new_collage.copy_from(&collage, 0, 0).unwrap();
            return place_image(new_collage, new_image)
        }
    }
}

/// Checks if a space in the collage is empty.
///
/// # Parameters
///
/// - `collage`: The existing collage.
/// - `x`: The x-coordinate of the top-left corner of the space.
/// - `y`: The y-coordinate of the top-left corner of the space.
/// - `width`: The width of the space.
/// - `height`: The height of the space.
///
/// # Returns
///
/// A boolean indicating if the space is empty.
fn is_empty_space(collage: &DynamicImage, x: u32, y: u32, mut width: u32, mut height: u32) -> bool {
    let (collage_width, collage_height) = collage.dimensions();

    if x + width > collage_width {
        width = collage_width - x;
    }
    if y + height > collage_height{
        height = collage_height - y;
    }

    for j in y..(y + height) {
        for i in x..(x + width) {
            let pixel = collage.get_pixel(i, j);
            if pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0 {
                return false;
            }
        }
    }
    true
}


/// Scales an image to a standard width while maintaining its aspect ratio.
///
/// # Parameters
///
/// - `img`: The image to be scaled.
/// - `standard_width`: The standard width to scale the image to.
///
/// # Returns
///
/// A new image scaled to the standard width.
fn scale_to_standard_width(img: DynamicImage, standard_width: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (current_width, current_height) = img.dimensions();

    // Calculate the new height while maintaining the aspect ratio.
    let new_height = (standard_width as f64 / current_width as f64 * current_height as f64) as u32;

    // Resize the image.
    resize(&img, standard_width, new_height, FilterType::Lanczos3)
}


/// Processes images from a directory and creates a collage.
///
/// # Parameters
///
/// - `dir`: The directory containing the images.
/// - `filter`: An optional filter for image extensions or filenames.
///
/// # Returns
///
/// A single image representing the collage.
pub fn process_images(dir: &str, filter: Option<String>) -> DynamicImage {
    let images_vec = load_images(dir, filter);
    create_collage(images_vec)
}
