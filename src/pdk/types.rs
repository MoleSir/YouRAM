use std::{fmt::Display, hash::Hash};

use reda_unit::{Capacitance, Temperature, Time, Voltage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Process {
    #[serde(rename = "TT")]
    TypeType,

    #[serde(rename = "FF")]
    FastFast,

    #[serde(rename = "SS")]
    SlowSlow,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pvt {
    pub process: Process,
    pub voltage: Voltage,
    pub temperature: Temperature,
}

impl Pvt {
    pub fn new<P, V, T>(process: P, voltage: V, temperature: T) -> Self 
    where 
        P: Into<Process>,
        V: Into<Voltage>,
        T: Into<Temperature>
    {
        Self {
            process: process.into(),
            voltage: voltage.into(),
            temperature: temperature.into(),
        }
    }
}

impl Display for Pvt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P{}_V{}_T{}", self.process, self.voltage, self.temperature)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SlewLoad {
    pub slew: Time,
    pub load: Capacitance,
}

impl SlewLoad {
    pub fn new<S, L>(slew: S, load: L) -> Self 
    where 
        S: Into<Time>,
        L: Into<Capacitance>,
    {
        Self {
            slew: slew.into(),
            load: load.into(),
        }
    }
}

impl Hash for SlewLoad {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.slew.value().to_f64().to_bits());
        state.write_u64(self.load.value().to_f64().to_bits());
    }
}

impl Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeType => write!(f, "TT"),
            Self::FastFast => write!(f, "FF"),
            Self::SlowSlow => write!(f, "SS"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Enviroment {
    pvt: Pvt,
    input_slew: Time,
    output_load: Capacitance,
}

impl Enviroment {
    pub fn new(pvt: Pvt, input_slew: Time, output_load: Capacitance) -> Self {
        Self {
            pvt, input_slew, output_load
        }
    }

    pub fn process(&self) -> Process {
        self.pvt.process
    }

    pub fn voltage(&self) -> Voltage {
        self.pvt.voltage
    }

    pub fn temperature(&self) -> Temperature {
        self.pvt.temperature
    }

    pub fn input_slew(&self) -> Time {
        self.input_slew
    }

    pub fn output_load(&self) -> Capacitance {
        self.output_load
    }
}