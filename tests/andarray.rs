use std::sync::Arc;
use rand::Rng;
use reda_unit::t;
use tracing::{info, Level};
use youram::{
    circuit::{AndArray, AndArrayArg, CircuitFactory}, 
    pdk::{Enviroment, Pdk}, 
    simulate::{CircuitSimulator, NgSpice, VoltageAtMeas}, 
    ErrorContext
};
use approx::assert_abs_diff_eq;

const PDK: &str = "./platforms/nangate45";
const TEMP: &str = "./temp";
const AND_SIZE: usize = 8;

fn main_result() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    let pdk = Arc::new(Pdk::load(PDK).context("load pdk")?);
    let mut factory = CircuitFactory::new(pdk.clone());
    let andarray = factory.module(AndArrayArg::new(AND_SIZE))?;
    let pvt = pdk.pvt();
    let env = Enviroment::new(pvt.clone(), t!(0.5 n), 0.0.into());

    // test en = 1
    {
        let mut simulator = CircuitSimulator::create(
            andarray.clone(), 
            env.clone(), 
            pdk.clone(), 
            format!("{TEMP}/simulate.sp"), 
            format!("{TEMP}/andarray.sp"), 
        )?;
    
        simulator.write_logic1_stimulate(AndArray::enbale_pn())?;
        let inputs: Vec<bool> = rand::rng().random_iter().take(AND_SIZE).collect();
        for (i, input) in inputs.iter().enumerate() {
            if *input {
                simulator.write_logic1_stimulate(AndArray::input_pn(i))?;
            } else {
                simulator.write_logic0_stimulate(AndArray::input_pn(i))?;
            }
        }

        for i in 0..inputs.len() {
            let meas = VoltageAtMeas::new(format!("output{i}"), AndArray::output_pn(i).to_string(), t!(10. n));
            simulator.write_measurement(Box::new(meas))?;
        }

        simulator.write_trans(t!(0.5 n), t!(0.0), t!(15. n))?;
        let result = simulator.simulate(NgSpice, TEMP)?;

        for (index, expect_value) in inputs.into_iter().enumerate() {
            let expect_value = if expect_value { pvt.voltage.to_f64() } else { 0.0 }; 
            let name = format!("output{index}");
            let got_value = result.get(&name).unwrap().to_f64();
            info!("{name}: got {got_value}, expect {expect_value}");
            assert_abs_diff_eq!(expect_value, got_value, epsilon = 1e-2);
        }
    }

    // test en = 0
    {
        let mut simulator = CircuitSimulator::create(
            andarray.clone(), 
            env.clone(), 
            pdk.clone(), 
            format!("{TEMP}/simulate.sp"), 
            format!("{TEMP}/andarray.sp"), 
        )?;
    
        simulator.write_logic0_stimulate(AndArray::enbale_pn())?;
        let inputs: Vec<bool> = rand::rng().random_iter().take(AND_SIZE).collect();
        for (i, input) in inputs.iter().enumerate() {
            if *input {
                simulator.write_logic1_stimulate(AndArray::input_pn(i))?;
            } else {
                simulator.write_logic0_stimulate(AndArray::input_pn(i))?;
            }
        }

        for i in 0..inputs.len() {
            let meas = VoltageAtMeas::new(format!("output{i}"), AndArray::output_pn(i).to_string(), t!(10. n));
            simulator.write_measurement(Box::new(meas))?;
        }

        simulator.write_trans(t!(0.5 n), t!(0.0), t!(15. n))?;
        let result = simulator.simulate(NgSpice, TEMP)?;

        for (name, got_value) in result {
            info!("{name}: got {got_value}");
            assert_abs_diff_eq!(0.0, got_value.to_f64(), epsilon = 1e-2);
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