use std::path::Path;

#[derive(Debug, Clone)]
pub struct EpdImage {
    image: image::DynamicImage,
}

#[derive(Debug, Clone, Copy)]
pub struct EpdImageFormat {
    /// width in px
    pub width: u32,
    /// height in px
    pub height: u32,
}

impl EpdImage {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let image = image::io::Reader::open(path)?.decode()?;

        Ok(Self { image })
    }

    pub fn export(self, format: &EpdImageFormat) -> anyhow::Result<Vec<u8>> {
        let mut data = vec![];

        let grayimage = self
            .image
            .resize_exact(
                format.width,
                format.height,
                image::imageops::FilterType::Gaussian,
            )
            .grayscale()
            .into_luma8();

        let bwimage = imageproc::contrast::threshold(&grayimage, 0x88).into_raw();
        let px_chunks = bwimage.chunks_exact(8);

        // Pack the luma8 image (1byte per px) to one that only has one bit per pixel
        for px_chunk in px_chunks {
            let px = px_chunk[0] & 0x01 << 7
                | px_chunk[1] & 0x01 << 6
                | px_chunk[2] & 0x01 << 5
                | px_chunk[3] & 0x01 << 4
                | px_chunk[4] & 0x01 << 3
                | px_chunk[5] & 0x01 << 2
                | px_chunk[6] & 0x01 << 1
                | px_chunk[7] & 0x01;

            data.push(px)
        }

        // TODO: include remainding chunk

        Ok(data)
    }
}
