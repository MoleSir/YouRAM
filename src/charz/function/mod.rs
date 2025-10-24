mod random;
mod mats;
mod marchx;
mod marchcminus;
mod marchc;
pub use random::*;
pub use mats::*;
pub use marchx::*;
pub use marchcminus::*;
pub use marchc::*;

use std::{collections::HashMap, path::PathBuf, sync::Arc};
use approx::AbsDiffEq;
use reda_unit::{t, Time};
use tracing::{debug, error, info, warn};
use crate::{circuit::{Shr, Sram}, pdk::{Enviroment, Pdk}, simulate::{ExecuteCommand, NgSpice, VoltageAtMeas}, YouRAMResult};
use super::{CharzError, SramTransactionGenerator};

pub struct FunctionTest {
    pub sram: Shr<Sram>,
    pub period: Time,
    pub env: Enviroment,
    pub pdk: Arc<Pdk>,
    
    pub policy: Box<dyn FunctionTestPolicy>,
    pub command: Box<dyn ExecuteCommand>,

    pub temp_folder: PathBuf,
    pub simulate_path: PathBuf,
    pub circuit_path: PathBuf,
}

impl FunctionTest {
    pub fn test(self) -> YouRAMResult<bool> {
        let mut charz = FunctionCharz::new(self.sram.clone(), self.period);
        self.policy.generate_transactions(&mut charz)?;
    
        let voltage = self.env.voltage();
        let result = charz.transactions.simulate(self.env, self.pdk, self.command, self.simulate_path, self.circuit_path, self.temp_folder)?;
        let expect_result = charz.target_meas_result;
    
        info!("check simulation result");
    
        let mut failed_size = 0;
        for (name, target_value) in expect_result.iter() {
            debug!("check meas {}", name);
            match result.get(name) {
                None => {
                    error!("the .meas task {} not found!", name);
                    return Ok(false);
                }
                Some(real_value) => {
                    let target_value = if *target_value { voltage } else { 0.0.into() }; 
                    if !target_value.to_f64().abs_diff_eq(&real_value.to_f64(), 1e-2) {
                        debug!("check meas {} failed, expect {}, not got {}", name, target_value, real_value);
                        failed_size += 1;
                    }
                }
            }
        }
    
        if failed_size == 0 {
            info!("functional test pass in all {} test",expect_result.len());
            Ok(true)
        } else {
            warn!("functional test failed, {} failed in {} test", failed_size, expect_result.len());
            Ok(false)
        }
    }


}

pub struct FunctionCharz {
    pub transactions: SramTransactionGenerator,
    pub target_meas_result: HashMap<String, bool>, 
}

pub trait FunctionTestPolicy {
    fn generate_transactions(&self, charz: &mut FunctionCharz) -> YouRAMResult<()>;
}

impl FunctionCharz {
    pub fn new(sram: Shr<Sram>, period: Time) -> Self {
        Self { 
            transactions: SramTransactionGenerator::new(sram, period),
            target_meas_result: HashMap::new()
        }
    }

    pub fn generate_transactions(&mut self, policy: impl FunctionTestPolicy) -> YouRAMResult<()> {
        policy.generate_transactions(self)
    }

    pub fn add_write_transaction(&mut self, address: usize, word: usize) {
        self.transactions.add_write_transaction(address, word);
    }

    pub fn add_read_transaction(&mut self, address: usize) {
        if !self.transactions.add_read_transaction(address) {
            return;
        }

        let bits = self.transactions.memory(address).unwrap().clone();
        // Wow, the last transaction is read, if there is `size` transactions
        // This read transaction's index is `transaction-1`, it will be enbale by `transaction-1` clock
        // So, we can read output in No. `transaction`'s clock rise
        let meas_time = self.transactions.clock_rise_time(self.transactions.transaction_size()) - t!(1 n);

        // for each bit of ouput port, add a meas
        for (bit_index, &bit) in bits.iter().enumerate() {
            let meas_index = self.target_meas_result.len();
            let meas_name = format!("dout{}_{}", bit_index, meas_index);
            let port_name = Sram::data_output_pn(bit_index);        
            let meas = VoltageAtMeas::new(meas_name.clone(), port_name.to_string(), meas_time);

            self.transactions.add_measurement(meas);
            self.target_meas_result.insert(meas_name, bit);
        } 
    }

}

/// Deafult:
/// - policy: random
/// - execute: ngspice
/// - temp_folder: "./temp"
pub struct FunctionTestBuilder {
    pub sram: Option<Shr<Sram>>,
    pub period: Option<Time>,
    pub env: Option<Enviroment>,
    pub pdk: Option<Arc<Pdk>>,
    
    pub policy: Option<Box<dyn FunctionTestPolicy>>,
    pub command: Option<Box<dyn ExecuteCommand>>,

    pub temp_folder: Option<PathBuf>,
    pub simulate_path: Option<PathBuf>,
    pub circuit_path: Option<PathBuf>,
}

impl Default for FunctionTestBuilder {
    fn default() -> Self {
        Self {
            sram: None,
            period: None,
            env: None, 
            pdk: None,
            policy: Some(Box::new(RandomPolicy)),
            command: Some(Box::new(NgSpice)),
            temp_folder: Some("./temp".into()),
            simulate_path: None, 
            circuit_path: None,
        }
    }
}

impl FunctionTestBuilder {
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

    pub fn env(self, env: impl Into<Enviroment>) -> Self {
        let mut build = self;
        build.env = Some(env.into());
        build
    }

    pub fn pdk(self, pdk: Arc<Pdk>) -> Self {
        let mut build = self;
        build.pdk = Some(pdk);
        build
    }

    pub fn policy<T: FunctionTestPolicy + 'static>(mut self, policy: impl Into<Box<T>>) -> Self {
        let policy: Box<T> = policy.into();
        self.policy = Some(policy);
        self
    }

    pub fn command<T: ExecuteCommand + 'static>(mut self, command: impl Into<Box<T>>) -> Self {
        let command: Box<T> = command.into();
        self.command = Some(command);
        self
    }

    pub fn policy_box(mut self, policy: Box<dyn FunctionTestPolicy>) -> Self {
        self.policy = Some(policy.into());
        self
    }

    pub fn command_box(mut self, command: Box<dyn ExecuteCommand>) -> Self {
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

    pub fn build(self) -> Result<FunctionTest, CharzError> {
        let sram = self.sram.ok_or(CharzError::LackFunctionTestConfigFeild("sram"))?;
        let temp_folder =  self.temp_folder.unwrap_or_else(|| "./temp".into());
        let simulate_path = self.simulate_path.unwrap_or_else(|| temp_folder.join("simulate.sp"));
        let circuit_path = self.circuit_path.unwrap_or_else(|| temp_folder.join(sram.read().name.to_string()));

        Ok(FunctionTest {
            period: self.period.ok_or(CharzError::LackFunctionTestConfigFeild("period"))?,
            env: self.env.ok_or(CharzError::LackFunctionTestConfigFeild("env"))?,
            pdk: self.pdk.ok_or(CharzError::LackFunctionTestConfigFeild("pdk"))?,
            policy: self.policy.ok_or(CharzError::LackFunctionTestConfigFeild("policy"))?,
            command: self.command.ok_or(CharzError::LackFunctionTestConfigFeild("command"))?,
            
            sram,
            temp_folder,
            simulate_path,
            circuit_path,
        })
    }
}
