use derive_builder::Builder;
use reda_unit::{Time, Voltage};
use super::Meas;

#[derive(Debug, Clone, Copy)]
pub enum Edge {
    Fall,
    Rise,
}

#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct DelayMeas {
    pub name: String,

    pub trig_net_name: String,
    pub trig_edge: Edge,
    pub trig_voltage: Voltage,
    pub trig_time_delay: Time,

    pub targ_net_name: String,
    pub targ_edge: Edge,
    pub targ_voltage: Voltage,
    pub targ_time_delay: Time,
}

impl Meas for DelayMeas {
    fn name(&self) -> &str {
        &self.name
    }

    fn write_command(&self, out: &mut dyn std::io::Write) -> std::io::Result<()> {
        let command = format!(
            ".meas tran {} TRIG v({}) VAL={} {}=1 TD={} TARG v({}) VAL={} {}=1 TD={}\n",
            
            self.name,

            self.trig_net_name,
            self.trig_voltage,
            self.trig_edge,
            self.trig_time_delay,

            self.targ_net_name,
            self.targ_voltage,
            self.targ_edge,
            self.targ_time_delay,
        );

        out.write_all(command.as_bytes())
    }
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fall => write!(f, "FALL"),
            Self::Rise => write!(f, "RISE"),
        }
    
    }
}