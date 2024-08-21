use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, time};

use dandelion_agent_lib::anyhow::{anyhow, Result};
use dandelion_agent_lib::{agent_factory, Agent, AgentBox};

agent_factory!(MAILDROP_FACTORY, new_maildrop, c"maildrop");

fn new_maildrop() -> AgentBox {
    Box::new(MaildropAgent::new())
}

const SUFFIX: &str = ".dmxf";

#[derive(Debug, Clone)]
struct Config {
    inbox: PathBuf,
    outbox: PathBuf,
}

struct MaildropAgent {
    counter: u32,
    config: Option<Config>,
    recv: Vec<PathBuf>,
}

impl MaildropAgent {
    fn new() -> Self {
        Self { counter: 0, config: None, recv: Vec::new() }
    }

    fn next_counter(&mut self) -> u32 {
        let result = self.counter;
        self.counter = self.counter.wrapping_add(1);
        result
    }

    fn inbox(&self) -> &Path {
        self.config.as_ref().unwrap().inbox.as_path()
    }

    fn outbox(&self) -> &Path {
        self.config.as_ref().unwrap().outbox.as_path()
    }
}

impl Agent for MaildropAgent {
    fn init(&mut self, args: &[&str]) -> Result<()> {
        let mut inbox = Option::<PathBuf>::None;
        let mut outbox = Option::<PathBuf>::None;

        for arg in args {
            if let Some(value) = arg.strip_prefix("inbox=") {
                inbox = Some(value.into());
            } else if let Some(value) = arg.strip_prefix("outbox=") {
                outbox = Some(value.into());
            } else {
                return Err(anyhow!("failed to parse argument: {}", arg));
            }
        }

        let inbox = inbox.ok_or_else(|| anyhow!("missing required argument 'inbox'"))?;
        let outbox = outbox.ok_or_else(|| anyhow!("missing required argument 'outbox'"))?;

        fs::create_dir_all(&inbox)?;
        fs::create_dir_all(&outbox)?;

        self.config = Some(Config { inbox, outbox });
        Ok(())
    }

    fn send(&mut self, message: &[u8]) -> Result<()> {
        let mut tmp_path = self.outbox().to_owned();
        let mut secs;
        let mut nanos;
        let mut counter;
        let mut file = loop {
            let ts = time::SystemTime::now().duration_since(time::UNIX_EPOCH)?;
            secs = ts.as_secs();
            nanos = ts.subsec_nanos();
            counter = self.next_counter();
            tmp_path.push(format!("{secs:012}-{nanos:09}-{counter:08x}{SUFFIX}.tmp"));
            match fs::OpenOptions::new().write(true).create_new(true).open(&tmp_path) {
                Ok(f) => break f,
                Err(err) if err.kind() == ErrorKind::AlreadyExists => {},
                Err(err) => return Err(err.into()),
            }
            tmp_path.pop();
        };

        let mut unlink_guard = UnlinkGuard::new(&tmp_path);

        file.write_all(message)?;
        file.sync_all()?;

        let mut final_path = tmp_path.clone();
        final_path.set_extension("");

        fs::rename(&tmp_path, &final_path)?;
        unlink_guard.0.take();

        Ok(())
    }

    fn recv_begin(&mut self) -> Result<usize> {
        self.recv.clear();
        for entry in fs::read_dir(self.inbox())? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_file() {
                let path = entry.path();
                if is_matching_path(&path) {
                    self.recv.push(path);
                }
            }
        }
        Ok(self.recv.len())
    }

    fn recv_read(&mut self, index: usize, into: &mut Vec<u8>) -> Result<()> {
        assert!(index < self.recv.len());
        let path = self.recv[index].as_path();
        let _ = File::open(path)?.read_to_end(into)?;
        Ok(())
    }

    fn recv_commit(&mut self, num: usize) -> Result<()> {
        assert!(num <= self.recv.len());
        for i in 0..num {
            fs::remove_file(&self.recv[i])?;
        }
        self.recv.clear();
        Ok(())
    }
}

struct UnlinkGuard<'a>(Option<&'a Path>);

impl<'a> UnlinkGuard<'a> {
    fn new(path: &'a Path) -> Self {
        Self(Some(path))
    }
}

impl Drop for UnlinkGuard<'_> {
    fn drop(&mut self) {
        if let Some(path) = &self.0 {
            fs::remove_file(path).ok();
        }
    }
}

fn is_matching_path(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_string_lossy();
    name.ends_with(SUFFIX) && !name.starts_with('.')
}
