use std::ffi::{CStr, OsStr};

use dandelion_agent_lib::FACTORY;

use super::instance::Instance;
use crate::anyhow::Result;

pub mod external;

pub use external::ExternalCollection;

#[derive(Debug)]
pub enum Collection {
    Empty,
    InProcess,
    External(ExternalCollection),
}

impl Collection {
    pub fn open<S: AsRef<OsStr>>(filename: S) -> Result<Self> {
        Ok(Self::External(ExternalCollection::open(filename)?))
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::InProcess => FACTORY.len(),
            Self::External(ext) => ext.len(),
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self, 0, self.len())
    }

    pub fn item(&self, index: usize) -> Item<'_> {
        let len = self.len();
        assert!(index < len, "index {index} must be strictly less than {len}");
        Item::new(self, index)
    }

    pub fn name(&self, index: usize) -> &CStr {
        self.item(index).name()
    }

    pub fn description(&self, index: usize) -> &CStr {
        self.item(index).description()
    }

    pub fn instantiate<'coll>(&'coll self, index: usize, args: &[&str]) -> Result<Instance<'coll>> {
        self.item(index).instantiate(args)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Item<'coll> {
    collection: &'coll Collection,
    index: usize,
}

impl<'coll> Item<'coll> {
    fn new(collection: &'coll Collection, index: usize) -> Self {
        Self { collection, index }
    }

    pub fn collection(self) -> &'coll Collection {
        self.collection
    }

    pub fn index(self) -> usize {
        self.index
    }

    pub fn name(self) -> &'coll CStr {
        match self.collection {
            Collection::Empty => unreachable!(),
            Collection::InProcess => FACTORY[self.index].name,
            Collection::External(ext) => ext.name(self.index),
        }
    }

    pub fn description(self) -> &'coll CStr {
        match self.collection {
            Collection::Empty => unreachable!(),
            Collection::InProcess => FACTORY[self.index].desc,
            Collection::External(ext) => ext.description(self.index),
        }
    }

    pub fn instantiate(self, args: &[&str]) -> Result<Instance<'coll>> {
        match self.collection {
            Collection::Empty => unreachable!(),
            Collection::InProcess => {
                let mut agent = (FACTORY[self.index].ctor)();
                agent.init(args)?;
                Ok(Instance::new(self, agent))
            },
            Collection::External(ext) => {
                let agent = ext.alloc(self.index, args)?;
                Ok(Instance::new(self, agent))
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Iter<'coll> {
    collection: &'coll Collection,
    index: usize,
    len: usize,
}

impl<'coll> Iter<'coll> {
    fn new(collection: &'coll Collection, index: usize, len: usize) -> Self {
        Self { collection, index, len }
    }
}

impl<'coll> Iterator for Iter<'coll> {
    type Item = Item<'coll>;

    fn count(self) -> usize {
        let remaining = self.len - self.index;
        remaining
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.count();
        (remaining, Some(remaining))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let index = self.index.saturating_add(n);
        if index < self.len {
            self.index = index + 1;
            Some(Item::new(self.collection, index))
        } else {
            self.index = self.len;
            None
        }
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }
}
