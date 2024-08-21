use std::ffi::CStr;
use std::sync::{Condvar, Mutex};

use dandelion_agent_lib::Agent;

use super::anyhow::Result;
use super::collection::{Collection, Item};

pub struct Instance<'coll> {
    item: Item<'coll>,
    cv: Condvar,
    mutex: Mutex<State<'coll>>,
}

struct State<'coll> {
    agent: Box<dyn Agent + 'coll>,
    busy: bool,
}

impl<'coll> Instance<'coll> {
    pub(crate) fn new(item: Item<'coll>, agent: Box<dyn Agent + 'coll>) -> Self {
        let busy = false;
        let mutex = Mutex::new(State { agent, busy });
        let cv = Condvar::new();
        Self { item, cv, mutex }
    }

    pub fn item(&self) -> Item<'coll> {
        self.item
    }

    pub fn collection(&self) -> &'coll Collection {
        self.item.collection()
    }

    pub fn index(&self) -> usize {
        self.item.index()
    }

    pub fn name(&self) -> &'coll CStr {
        self.item.name()
    }

    pub fn send(&self, data: &[u8]) -> Result<()> {
        let mut guard = self.mutex.lock().unwrap();
        guard.agent.send(data)
    }

    pub fn recv(&self) -> Result<Recv<'coll, '_>> {
        let mut guard = self.mutex.lock().unwrap();
        while guard.busy {
            guard = self.cv.wait(guard).unwrap();
        }
        let count = guard.agent.recv_begin()?;
        guard.busy = true;
        Ok(Recv { instance: self, count, done: false })
    }
}

pub struct Recv<'coll, 'inst: 'coll> {
    instance: &'inst Instance<'coll>,
    count: usize,
    done: bool,
}

impl<'coll, 'inst: 'coll> Drop for Recv<'coll, 'inst> {
    fn drop(&mut self) {
        if self.done {
            return;
        }

        let mut guard = self.instance.mutex.lock().unwrap();
        let _notify = NotifyGuard(&self.instance.cv);
        guard.busy = false;

        guard.agent.recv_abort().ok();
    }
}

impl<'coll, 'inst: 'coll> Recv<'coll, 'inst> {
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn read_into(&self, index: usize, into: &mut Vec<u8>) -> Result<()> {
        assert!(index < self.count);

        let mut guard = self.instance.mutex.lock().unwrap();
        guard.agent.recv_read(index, into)
    }

    pub fn read(&self, index: usize) -> Result<Vec<u8>> {
        let mut vec = Vec::new();
        self.read_into(index, &mut vec)?;
        Ok(vec)
    }

    pub fn commit(&mut self, num: usize) -> Result<()> {
        if self.done {
            return Ok(());
        }

        let mut guard = self.instance.mutex.lock().unwrap();
        let _notify = NotifyGuard(&self.instance.cv);
        guard.busy = false;
        self.done = true;

        guard.agent.recv_commit(num)
    }

    pub fn commit_all(&mut self) -> Result<()> {
        self.commit(self.count)
    }
}

struct NotifyGuard<'inst>(&'inst Condvar);

impl Drop for NotifyGuard<'_> {
    fn drop(&mut self) {
        self.0.notify_one();
    }
}
