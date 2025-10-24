use std::sync::Arc;
use reda_unit::t;
use tracing::{info, Level};
use youram::{
    circuit::{CircuitFactory, Decoder, DecoderArg}, 
    pdk::{Enviroment, Pdk}, 
    simulate::{CircuitSimulator, NgSpice, VoltageAtMeas}, 
    ErrorContext
};
use approx::assert_abs_diff_eq;

const PDK: &str = "./platforms/nangate45";
const TEMP: &str = "./temp";
const INPUT_SIZE: usize = 4;
const OUTPUT_SIZE: usize = 2usize.pow(INPUT_SIZE as u32);

fn main_result() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    let pdk = Arc::new(Pdk::load(PDK).context("load pdk")?);
    let mut factory = CircuitFactory::new(pdk.clone());
    let decoder = factory.module(DecoderArg::new(INPUT_SIZE))?;
    let pvt = pdk.pvt();
    let env = Enviroment::new(pvt.clone(), t!(0.5 n), 0.0.into());

    for t in 0..OUTPUT_SIZE {
        info!("test input {t}");
        let mut simulator = CircuitSimulator::create(
            decoder.clone(), 
            env.clone(), 
            pdk.clone(), 
            format!("{TEMP}/simulate.sp"), 
            format!("{TEMP}/decoder.sp"), 
        )?;
        
        for input_index in 0..INPUT_SIZE {
            let is_logic1 = (t & 0x1 << input_index) != 0;
            if is_logic1 {
                simulator.write_logic1_stimulate(Decoder::address_pn(input_index))?;
            } else {
                simulator.write_logic0_stimulate(Decoder::address_pn(input_index))?;
            }
        }
     
        for i in 0..OUTPUT_SIZE {
            let meas = VoltageAtMeas::new(format!("output{i}"), Decoder::output_pn(i).to_string(), t!(10. n));
            simulator.write_measurement(Box::new(meas))?;
        }
        simulator.write_trans(t!(0.5 n), t!(0.0), t!(15. n))?;

        let result = simulator.simulate(NgSpice, TEMP)?;
        
        for (name, value) in result {
            info!("{name}: {value}");
            if name == format!("output{t}") {
                assert_abs_diff_eq!(value.to_f64(), pvt.voltage.to_f64(), epsilon = 1e-2);
            } else {
                assert_abs_diff_eq!(value.to_f64(), 0.0, epsilon = 1e-2);
            }
        }   
    }

    Ok(())
}

#[test]
fn main() {
    if let Err(e) = main_result() {
        eprint!("Err: {}\n", e);
    }
}