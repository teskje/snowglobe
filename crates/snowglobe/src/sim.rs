use std::time::Duration;

use turmoil::{ToIpAddr, ToIpAddrs};

use crate::{Result, context};

pub struct Sim(turmoil::Sim<'static>);

impl From<turmoil::Sim<'static>> for Sim {
    fn from(sim: turmoil::Sim<'static>) -> Self {
        context::advance_time(sim.since_epoch());

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
        Fut: Future<Output = Result> + 'static,
    {
        self.0.host(addr, host)
    }

    pub fn client<Fut>(&mut self, addr: impl ToIpAddr, client: Fut)
    where
        Fut: Future<Output = Result> + 'static,
    {
        self.0.client(addr, client)
    }

    pub fn crash(&mut self, addrs: impl ToIpAddrs) {
        self.0.crash(addrs)
    }

    pub fn bounce(&mut self, addrs: impl ToIpAddrs) {
        self.0.bounce(addrs)
    }

    pub fn hold(&self, a: impl ToIpAddrs, b: impl ToIpAddrs) {
        self.0.hold(a, b)
    }

    pub fn release(&self, a: impl ToIpAddrs, b: impl ToIpAddrs) {
        self.0.release(a, b)
    }

    pub fn partition_oneway(&self, from: impl ToIpAddrs, to: impl ToIpAddrs) {
        self.0.partition_oneway(from, to)
    }

    pub fn repair_oneway(&self, from: impl ToIpAddrs, to: impl ToIpAddrs) {
        self.0.repair_oneway(from, to)
    }

    pub fn step(&mut self) -> Result<bool> {
        let res = self.0.step();

        let duration = self.0.since_epoch();
        context::advance_time(duration);

        res
    }

    pub fn run(&mut self) -> Result {
        let mut finished = false;
        while !finished {
            finished = self.step()?;
        }

        Ok(())
    }
}
