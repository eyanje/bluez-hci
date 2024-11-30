use std::io::{Error, Result};
use std::io::ErrorKind::{InvalidInput};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct HciFilter {
	type_mask: u32,
	event_mask: u64,
	opcode: u16,
}

impl HciFilter {
    /// Return the type mask
    pub fn get_type_mask(&self) -> u32 {
        self.type_mask
    }

    /// Set the type mask
    pub fn set_type_mask(&mut self, type_mask: u32) {
        self.type_mask = type_mask
    }
    
    /// Clear the type mask
    pub fn clear_type_mask(&mut self) {
        self.type_mask = 0;
    }

    /// Set a ptype in the type mask
    pub fn set_type(&mut self, t: u8) -> Result<()> {
        if t < 32 {
            self.type_mask |= 1 << t;
            Ok(())
        } else {
            Err(Error::new(InvalidInput, "Ptype out of range"))
        }
    }

    /// Clear a ptype in the type mask
    pub fn unset_type(&mut self, t: u8) -> Result<()> {
        if t < 32 {
            self.type_mask &= !(1 << t);
            Ok(())
        } else {
            Err(Error::new(InvalidInput, "Ptype out of range"))
        }
    }


    /// Return the event mask
    pub fn get_event_mask(&self) -> u64 {
        self.event_mask
    }

    /// Set the event mask
    pub fn set_event_mask(&mut self, event_mask: u64) {
        self.event_mask = event_mask
    }

    /// Clear the event mask
    pub fn clear_event_mask(&mut self) {
        self.event_mask = 0;
    }

    /// Enable an event in the event mask
    pub fn set_event(&mut self, event: u8) -> Result<()> {
        if event < 64 {
            self.event_mask |= 1 << event;
            Ok(())
        } else {
            Err(Error::new(InvalidInput, "Event out of range"))
        }
    }
    
    /// Remove an event from the event mask
    pub fn unset_event(&mut self, event: u8) -> Result<()> {
        if event < 64 {
            self.event_mask &= !(1 << event);
            Ok(())
        } else {
            Err(Error::new(InvalidInput, "Event out of range"))
        }
    }


    /// Return the current opcode
    pub fn get_opcode(&self) -> u16 {
        self.opcode
    }

    /// Set opcode
    pub fn set_opcode(&mut self, opcode: u16) {
        self.opcode = opcode
    }
}

