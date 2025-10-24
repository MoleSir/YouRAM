use reda_unit::Time;
use super::Meas;

#[derive(Debug)]
pub struct VoltageAtMeas {
    pub name: String,
    pub net_name: String,
    pub meas_time: Time,
}

impl VoltageAtMeas {
    pub fn new<S1, S2, T>(name: S1, net_name: S2, meas_time: T) -> Self 
    where 
        S1: Into<String>,
        S2: Into<String>,
        T:  Into<Time>
    {
        Self { name: name.into(), net_name: net_name.into(), meas_time: meas_time.into() }
    }
}

impl Meas for VoltageAtMeas {
    fn name(&self) -> &str {
        &self.name
    }

    fn write_command(&self, out: &mut dyn std::io::Write) -> std::io::Result<()> {
        let command = format!(
            ".meas tran {} FIND v({}) AT={}\n",
            self.name,
            self.net_name,
            self.meas_time,
        );
        out.write_all(command.as_bytes())
    }
}
