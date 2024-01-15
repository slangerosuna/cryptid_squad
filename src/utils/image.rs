use glium::texture::RawImage2d;

pub fn load(
    path: String
) -> std::io::Result<ImageData> {
    let file_type = path.split('.').last().unwrap();

    match file_type {
        "png" => load_png(path),
        _ => panic!("Unsupported image type: {}", file_type),
    }
}

pub struct ImageData {
    raw_bytes: Vec<u8>,
    image_dimensions: (u32, u32),
}

impl glium::texture::Texture2dDataSource<'static> for ImageData {
    type Data = u8;

    fn into_raw(self) -> RawImage2d<'static, u8> {
        let image_dimensions = self.image_dimensions;
        RawImage2d::from_raw_rgba_reversed(&self.raw_bytes[..], image_dimensions)
    }
}

fn load_png(
    path: String
) -> std::io::Result<ImageData> {
    //TODO make actually work
    //let data = std::fs::read(path).unwrap();
    
    

    
    Ok(ImageData {
        raw_bytes: vec![255, 255, 255, 255],
        image_dimensions: (1, 1),
    })
}
