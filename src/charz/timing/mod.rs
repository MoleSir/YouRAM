use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}, sync::Arc};
use reda_unit::{Capacitance, Number, Time};
use tracing::{debug, info};
use crate::{circuit::{Shr, Sram}, export, pdk::{Enviroment, Pdk, Pvt}, simulate::{DelayMeasBuilder, Edge, NgSpice, SpiceCommand}, ErrorContext, YouRAMResult};
use super::{CharzError, SramTransactionGenerator};

#[derive(Debug)]
pub struct TimingCharzResult {
    pub delay_hl: Time,
    pub delay_lh: Time,
    pub slew_hl: Time,
    pub slew_lh: Time,
}

/// Timeing charz for Sram
/// 
/// # Deafult:
/// - command: ngspice
/// - temp_folder: "./temp"
/// - simulate_path: "./temp/simulator.sp"
/// - circuit_path: "./temp/<sram_name>.sp"
/// 
/// # Example
/// 
/// ```no_run
/// let result = TimingCharz::config()
///     .sram(sram)
///     .command(NgSpice)
///     .pdk(pdk)
///     .....
///     .period(t!(10 n))
///     .analyze()?;
/// ```
pub struct TimingCharz<'a> {
    pub sram: Option<Shr<Sram>>,
    pub period: Option<Time>,
    pub pvt: Option<Pvt>,
    pub input_net_transitions: Option<&'a [Time]>,
    pub output_net_capacitances: Option<&'a [Capacitance]>,
    pub pdk: Option<Arc<Pdk>>,
    
    pub command: Option<Box<dyn SpiceCommand>>,

    pub temp_folder: Option<PathBuf>,
    pub simulate_path: Option<PathBuf>,

    /// if no circuit_path, create <temp>/<sram_name>.sp. if have, include it directiontly
    pub circuit_path: Option<PathBuf>,
}

impl<'a> TimingCharz<'a> {
    pub fn analyze(self) -> YouRAMResult<Vec<Vec<TimingCharzResult>>> {
        info!("execute timing charz");
        
        // extract all args
        debug!("extract arguments");
        let sram = self.sram.ok_or(CharzError::LackFunctionTestConfigField("sram"))?;
        let period = self.period.ok_or(CharzError::LackFunctionTestConfigField("period"))?;
        let pvt = self.pvt.ok_or(CharzError::LackFunctionTestConfigField("pvt"))?;
        let pdk = self.pdk.ok_or(CharzError::LackFunctionTestConfigField("pdk"))?;
        let input_net_transitions = self.input_net_transitions.ok_or(CharzError::LackFunctionTestConfigField("input_net_transitions"))?;
        let output_net_capacitances = self.output_net_capacitances.ok_or(CharzError::LackFunctionTestConfigField("input_net_transitions"))?;

        let command = self.command.ok_or(CharzError::LackFunctionTestConfigField("command"))?;

        let temp_folder =  self.temp_folder.unwrap_or_else(|| "./temp".into());
        let simulate_path = self.simulate_path.unwrap_or_else(|| temp_folder.join("simulate.sp"));
        let circuit_path = match self.circuit_path {
            Some(circuit_path) => circuit_path,
            None => {
                let circuit_path = temp_folder.join(sram.read().name.to_string());
                export::write_spice(sram.clone(), &circuit_path).with_context(|| format!("write sram"))?;
                circuit_path
            }
        };

        // for all input_net_transition and output_net_capacitance
        let mut all_result = vec![];
        for &input_net_transition in input_net_transitions.iter() {
            let mut result_in_same_slew = vec![];
            for &output_net_capacitance in output_net_capacitances.iter() {
                let env = Enviroment::new(pvt.clone(), input_net_transition, output_net_capacitance);
                let result = Self::analyze_in_env(
                    sram.clone(), 
                    period, 
                    env, 
                    pdk.clone(), 
                    &command, 
                    &simulate_path, 
                    &circuit_path, 
                    &temp_folder
                )?;
                result_in_same_slew.push(result);
            }
            all_result.push(result_in_same_slew);
        }

        Ok(all_result)
    }

    fn analyze_in_env(
        sram: Shr<Sram>, 
        period: Time, 
        env: Enviroment, 
        pdk: Arc<Pdk>,
        command: &impl SpiceCommand,
        simulate_path: impl Into<PathBuf>,
        circuit_path: impl Into<PathBuf>,
        temp_folder: impl AsRef<Path>,
    ) -> YouRAMResult<TimingCharzResult> {
        let mut transactions = SramTransactionGenerator::new(sram, period);
            
        // generate some unique address
        debug!("generate transactions");
        let addresses: HashSet<usize> = Self::generate_random_address(&mut transactions);

        // for each address, write 0 + read 0, and add delay and slew meas
        for &address in addresses.iter() {
            transactions.add_write_transaction(address, 0);
            transactions.add_read_transaction(address);
        
            let word_width = transactions.sram.read().word_width();

            /*
                        +---+   +---+   +---+
                clk:    |   |   |   |   |   |
                    +---+   +---+   +---+   +---
                        |       |       |
                        | write | read  | write | read  ...
                out:     |xxxxxxx|xxxxx--|
                            |    ^
                            |    |
                            |    | get output
                            |    |
                            delay_hl
            */  

            // read transaction's rise clock 
            let time_delay = transactions.last_clock_rise_time() - transactions.half_period();
            
            // for all output bit, and meas
            for bit in 0..word_width {
                let output_pin = Sram::data_output_pn(bit).to_string();
                
                // meas the clk rise to output down
                let meas = DelayMeasBuilder::default()
                    .name(format!("delay_hl_d{}_b{}", address, bit))
                    
                    .trig_net_name(Sram::clock_pn().to_string())
                    .trig_edge(Edge::Rise)
                    .trig_voltage(env.voltage() * pdk.input_threshold_pct_rise())
                    .trig_time_delay(time_delay)
                    
                    .targ_net_name(output_pin.clone())
                    .targ_edge(Edge::Fall)
                    .targ_voltage(env.voltage() * pdk.output_threshold_pct_fall())
                    .targ_time_delay(time_delay)
                    .build().unwrap();
                transactions.add_measurement(meas);
    
                // meas from output to output
                let meas = DelayMeasBuilder::default()
                    .name(format!("slew_hl_d{}_b{}", address, bit))
                    
                    .trig_net_name(output_pin.clone())
                    .trig_edge(Edge::Fall)
                    .trig_voltage(pdk.slew_upper_threshold_pct_fall() * env.voltage())
                    .trig_time_delay(time_delay)
                    
                    .targ_net_name(output_pin.clone())
                    .targ_edge(Edge::Fall)
                    .targ_voltage(pdk.slew_lower_threshold_pct_fall() * env.voltage())
                    .targ_time_delay(time_delay)
                    .build().unwrap();
                transactions.add_measurement(meas);
            }
        }

        // simulate
        debug!("spice simulate");
        let result = transactions.simulate(env, pdk, command, simulate_path, Some(circuit_path), temp_folder)?;

        // extract result
        debug!("extract timing result");
        Self::extract_result(&result)
    }

    fn generate_random_address(transactions: &mut SramTransactionGenerator) -> HashSet<usize> {
        let total_address_size: usize = 2usize.pow(transactions.sram.read().address_width() as u32);
        let address_count = 2.max(total_address_size / 10);
        (0..address_count).map(|_| transactions.random_address()).collect()
    }

    fn average(values: &[Number]) -> Number {
        let mut sum = Number::zero();
        let size = values.len();
        for &v in values.iter() {
            sum = sum + v;
        }
        sum / size as f64
    }

    fn extract_result(result: &HashMap<String, Number>) -> YouRAMResult<TimingCharzResult> {
        let mut delay_hls = vec![];
        let mut slew_hls = vec![];

        for (name, value) in result.iter() {
            if name.contains("delay_hl") {
                delay_hls.push(value.clone());
            } else if name.contains("slew_hl") {
                slew_hls.push(value.clone());
            }
        }

        // average 
        let delay_hl = Self::average(&delay_hls);
        let slew_hl = Self::average(&slew_hls);

        Ok(TimingCharzResult { 
            delay_hl: Time::from(delay_hl), 
            delay_lh: Time::from(delay_hl), 
            slew_hl: Time::from(slew_hl), 
            slew_lh: Time::from(slew_hl) 
        })
    }
}

impl<'a> Default for TimingCharz<'a> {
    fn default() -> Self {
        Self {
            sram: None,
            period: None,
            pdk: None,
            pvt: None,
            input_net_transitions: None,
            output_net_capacitances: None,
            command: Some(Box::new(NgSpice)),
            temp_folder: Some("./temp".into()),
            simulate_path: None, 
            circuit_path: None,
        }
    }
}

impl<'a> TimingCharz<'a> {
    pub fn config() -> Self {
        Self::default()
    }

    pub fn sram(self, sram: impl Into<Shr<Sram>>) -> Self {
        let mut build = self;
        build.sram = Some(sram.into());
        build
    }

    pub fn period(self, period: impl Into<Time>) -> Self {
        let mut build = self;
        build.period = Some(period.into());
        build
    }

    pub fn pvt(self, pvt: impl Into<Pvt>) -> Self {
        let mut build = self;
        build.pvt = Some(pvt.into());
        build
    }

    pub fn input_net_transitions(self, input_net_transitions: &'a [Time]) -> Self {
        let mut build = self;
        build.input_net_transitions = Some(input_net_transitions);
        build
    }

    pub fn output_net_capacitances(self, output_net_capacitances: &'a [Capacitance]) -> Self {
        let mut build = self;
        build.output_net_capacitances = Some(output_net_capacitances);
        build
    }

    pub fn pdk(self, pdk: Arc<Pdk>) -> Self {
        let mut build = self;
        build.pdk = Some(pdk);
        build
    }

    pub fn command<T: SpiceCommand + 'static>(mut self, command: impl Into<Box<T>>) -> Self {
        let command: Box<T> = command.into();
        self.command = Some(command);
        self
    }

    pub fn command_box(mut self, command: Box<dyn SpiceCommand>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn temp_folder(self, temp_folder: impl Into<PathBuf>) -> Self {
        let mut build = self;
        build.temp_folder = Some(temp_folder.into());
        build
    }

    pub fn simulate_path(self, simulate_path: impl Into<PathBuf>) -> Self {
        let mut build = self;
        build.simulate_path = Some(simulate_path.into());
        build
    }

    pub fn circuit_path(self, circuit_path: impl Into<PathBuf>) -> Self {
        let mut build = self;
        build.circuit_path = Some(circuit_path.into());
        build
    }
}
