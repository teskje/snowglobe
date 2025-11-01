use std::time::Duration;

use turmoil::ToIpAddr;

pub struct Sim(turmoil::Sim<'static>);

impl From<turmoil::Sim<'static>> for Sim {
    fn from(sim: turmoil::Sim<'static>) -> Self {
        Self(sim)
    }
}

impl Sim {
    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }

    pub fn host<F, Fut>(&mut self, addr: impl ToIpAddr, host: F)
    where
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = turmoil::Result> + 'static,
    {
        self.0.host(addr, host)
    }

    pub fn client<Fut>(&mut self, addr: impl ToIpAddr, client: Fut)
    where
        Fut: Future<Output = turmoil::Result> + 'static,
    {
        self.0.client(addr, client)
    }

    pub fn step(&mut self) -> turmoil::Result<bool> {
        let res = self.0.step();

        let duration = self.0.since_epoch();
        crate::time::advance(duration);

        res
    }

    pub fn run(&mut self) -> turmoil::Result {
        let mut finished = false;
        while !finished {
            finished = self.step()?;
        }

        Ok(())
    }
}
