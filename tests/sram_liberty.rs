use std::sync::Arc;
use reda_unit::t;
use tracing::{error, Level};
use youram::{
    circuit::{CircuitFactory, SramArg}, export, pdk::Pdk, simulate::NgSpice, ErrorContext
};

const PDK: &str = "./platforms/nangate45";
const TEMP: &str = "./temp";
const ADDRESS_WIDTH: usize = 4;
const WORD_WIDTH: usize = 4;

fn main_result() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    let pdk = Arc::new(Pdk::load(PDK).context("load pdk")?);
    let mut factory = CircuitFactory::new(pdk.clone());
    let sram = factory.module(SramArg::new(ADDRESS_WIDTH, WORD_WIDTH))?;

    export::write_liberty(
        sram.clone(), 
        format!("{}/sram.lib", TEMP),
        t!(10. n), 
        pdk, 
        Box::new(NgSpice), 
        TEMP
    )?;

    Ok(())
}

#[test]
fn main() {
    if let Err(e) = main_result() {
        error!("Err: {}\n", e);
        panic!("");
    }
}