extern crate dbus;

mod block;
mod disks;
mod drive;
pub(crate) mod utils;

pub use self::block::*;
pub use self::disks::*;
pub use self::drive::*;

use dbus::arg::{RefArg, Variant};

use dbus::blocking::stdintf::org_freedesktop_dbus::ObjectManager;
use dbus::blocking::{Connection, Proxy};
use std::collections::HashMap;
use std::ops::Deref;
use std::time::Duration;
use utils::*;

const DEST: &str = "org.freedesktop.UDisks2";
const PATH: &str = "/org/freedesktop/UDisks2";

pub struct UDisks2 {
    conn: Connection,
    cache: HashMap<dbus::Path<'static>, HashMap<String, HashMap<String, Variant<Box<RefArg>>>>>,
}

impl UDisks2 {
    pub fn new() -> Result<Self, dbus::Error> {
        let mut udisks2 = Self {
            conn: Connection::new_system()?,
            cache: Default::default(),
        };

        udisks2.update()?;
        Ok(udisks2)
    }

    fn path(&self) -> Proxy<&Connection> {
        self.conn
            .with_proxy(DEST, PATH, Duration::from_millis(3000))
    }

    /// Refresh the managed objects fetched from the DBus server.
    pub fn update(&mut self) -> Result<(), dbus::Error> {
        self.cache = self.path().get_managed_objects()?;
        Ok(())
    }

    fn get_object<T: ParseFrom>(&self, path: &str) -> Option<T> {
        self.cache
            .iter()
            .flat_map(|object| {
                if object.0.deref() == path {
                    T::parse_from(&object.0, &object.1)
                } else {
                    None
                }
            })
            .next()
    }

    /// Find the drive that corresponds to the given dbus object path.
    pub fn get_drive(&self, path: &str) -> Option<Drive> {
        self.get_object::<Drive>(path)
    }

    /// An iterator of `Drive` objects fetched from the inner cached managed objects.
    pub fn get_drives<'a>(&'a self) -> impl Iterator<Item = Drive> + 'a {
        self.cache
            .iter()
            .flat_map(|object| Drive::parse_from(&object.0, &object.1))
    }

    /// Find the block that corresponds to the given dbus object path.
    pub fn get_block(&self, path: &str) -> Option<Block> {
        self.get_object::<Block>(path)
    }

    /// An iterator of `Block` objects fetched from the inner cached managed objects.
    pub fn get_blocks<'a>(&'a self) -> impl Iterator<Item = Block> + 'a {
        self.cache
            .iter()
            .flat_map(|object| Block::parse_from(&object.0, &object.1))
    }
}
