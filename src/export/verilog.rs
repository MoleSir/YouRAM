use std::{fs::File, io::{BufWriter, Write}, path::Path};
use tracing::info;

use crate::{circuit::{Shr, Sram}, YouRAMResult};

pub fn write_verilog<P: AsRef<Path>>(sram: Shr<Sram>, path: P) -> YouRAMResult<()> {
    let sram_ref = sram.read();
    let path = path.as_ref();

    info!("write sram {} to {:?}", sram_ref.name, path);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writer.write_all("module sram #(\n".as_bytes())?;
    writer.write_all(format!("    parameter ADDR_WIDTH = {},\n", sram_ref.address_width()).as_bytes())?;
    writer.write_all(format!("    parameter DATA_WIDTH = {} \n", sram_ref.word_width()).as_bytes())?;
    writer.write_all(include_str!("./template.v").as_bytes())?;

    Ok(())
}