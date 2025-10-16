use derive_builder::Builder;

use reda_unit::Number;

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
    pub trig_voltage: Number,
    pub trig_time_delay: Number,

    pub targ_net_name: String,
    pub targ_edge: Edge,
    pub targ_voltage: Number,
    pub targ_time_delay: Number,
}

impl Meas for DelayMeas {
    fn name(&self) -> &str {
        &self.name
    }

    fn write_command(&self, out: &mut dyn std::io::Write) -> std::io::Result<()> {
        let command = format!(
            ".meas tran {} TRIG v({}) VAL={} {}=1 TD={} TARG v({}) VAL={} {}=1 TD={}",
            
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