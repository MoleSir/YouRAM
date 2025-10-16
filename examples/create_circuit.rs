use std::sync::Arc;
use anyhow::Context;
use tracing::Level;
use youram::{circuit::{BufferArg, CircuitFactory, DecoderArg, DriveStrength}, export, pdk::Pdk};

fn main_result() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    let pdk = Arc::new(Pdk::load("./platforms/nangate45").context("load pdk")?);
    let mut factory = CircuitFactory::new(pdk);
    let _buffer = factory.module(BufferArg { strength: DriveStrength::X1 }).context("create module")?;
    let _decoder = factory.module(DecoderArg::new(10))?;
    export::write_spice(_decoder, "./temp/decoder.sp").context("export spice")?;
    Ok(())
}

fn main() {
    if let Err(e) = main_result() {
        eprint!("Err: {:?}\n", e);
    }
}