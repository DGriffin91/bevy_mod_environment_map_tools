pub struct KTX2Writer<'a> {
    pub header: Header,
    pub dfd_bytes: &'a [u8],
    pub levels_descending: Vec<WriterLevel>,
}

impl<'a> KTX2Writer<'a> {
    pub fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
        let dfd_offset =
            ktx2::Header::LENGTH + self.levels_descending.len() * ktx2::LevelIndex::LENGTH;

        writer.write_all(
            &ktx2::Header {
                format: self.header.format,
                type_size: if self.header.supercompression_scheme.is_some() {
                    1
                } else {
                    self.header.type_size
                },
                pixel_width: self.header.pixel_width,
                pixel_height: self.header.pixel_height,
                pixel_depth: self.header.pixel_depth,
                layer_count: self.header.layer_count,
                face_count: self.header.face_count,
                supercompression_scheme: self.header.supercompression_scheme,
                level_count: self.levels_descending.len() as u32,
                index: ktx2::Index {
                    dfd_byte_length: self.dfd_bytes.len() as u32,
                    kvd_byte_length: 0,
                    sgd_byte_length: 0,
                    dfd_byte_offset: dfd_offset as u32,
                    kvd_byte_offset: 0,
                    sgd_byte_offset: 0,
                },
            }
            .as_bytes()[..],
        )?;

        let mut offset = dfd_offset + self.dfd_bytes.len();

        let mut levels = self
            .levels_descending
            .iter()
            .rev()
            .map(|level| {
                let index = ktx2::LevelIndex {
                    byte_offset: offset as u64,
                    byte_length: level.bytes.len() as u64,
                    uncompressed_byte_length: level.uncompressed_length as u64,
                };

                offset += level.bytes.len();

                index
            })
            .collect::<Vec<_>>();

        levels.reverse();

        for level in levels {
            writer.write_all(&level.as_bytes())?;
        }

        writer.write_all(self.dfd_bytes)?;

        for level in self.levels_descending.iter().rev() {
            writer.write_all(&level.bytes)?;
        }

        Ok(())
    }
}

pub struct WriterLevel {
    pub uncompressed_length: usize,
    pub bytes: Vec<u8>,
}

pub struct Header {
    pub format: Option<ktx2::Format>,
    pub type_size: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub pixel_depth: u32,
    pub layer_count: u32,
    pub face_count: u32,
    pub supercompression_scheme: Option<ktx2::SupercompressionScheme>,
}
