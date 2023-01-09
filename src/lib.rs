pub mod clocks;
pub mod dir;
pub mod file;
pub mod sched;

pub use cap_std::fs::Dir;
pub use clocks::clocks_ctx;
pub use sched::sched_ctx;

use cap_rand::RngCore;
use std::path::Path;
use wasi_common::{file::FileCaps, Error, Table, WasiCtx, WasiFile};

pub struct WasiCtxBuilder(WasiCtx);
impl WasiCtxBuilder {
    pub fn new() -> Self {
        WasiCtxBuilder(WasiCtx::new(
            random_ctx(),
            clocks_ctx(),
            sched_ctx(),
            Table::new(),
        ))
    }
    pub fn env(mut self, var: &str, value: &str) -> Result<Self, wasi_common::StringArrayError> {
        self.0.push_env(var, value)?;
        Ok(self)
    }
    pub fn envs(mut self, env: &[(String, String)]) -> Result<Self, wasi_common::StringArrayError> {
        for (k, v) in env {
            self.0.push_env(k, v)?;
        }
        Ok(self)
    }
    pub fn arg(mut self, arg: &str) -> Result<Self, wasi_common::StringArrayError> {
        self.0.push_arg(arg)?;
        Ok(self)
    }
    pub fn args(mut self, arg: &[String]) -> Result<Self, wasi_common::StringArrayError> {
        for a in arg {
            self.0.push_arg(&a)?;
        }
        Ok(self)
    }
    pub fn preopened_dir(mut self, dir: Dir, guest_path: impl AsRef<Path>) -> Result<Self, Error> {
        let dir = Box::new(crate::dir::Dir::from_cap_std(dir));
        self.0.push_preopened_dir(dir, guest_path)?;
        Ok(self)
    }
    pub fn preopened_socket(mut self, fd: u32, socket: impl Into<Socket>) -> Result<Self, Error> {
        let socket: Socket = socket.into();
        let file: Box<dyn WasiFile> = socket.into();

        let caps = FileCaps::FDSTAT_SET_FLAGS
            | FileCaps::FILESTAT_GET
            | FileCaps::READ
            | FileCaps::POLL_READWRITE;

        self.0.insert_file(fd, file, caps);
        Ok(self)
    }
    pub fn stdin(mut self, f: Box<dyn WasiFile>) -> Self {
        self.0.set_stdin(f);
        self
    }
    pub fn stdout(mut self, f: Box<dyn WasiFile>) -> Self {
        self.0.set_stdout(f);
        self
    }
    pub fn stderr(mut self, f: Box<dyn WasiFile>) -> Self {
        self.0.set_stderr(f);
        self
    }
    pub fn build(self) -> WasiCtx {
        self.0
    }
}

pub fn random_ctx() -> Box<dyn RngCore + Send + Sync> {
    Box::new(cap_rand::std_rng_from_entropy(cap_rand::ambient_authority()))
}
