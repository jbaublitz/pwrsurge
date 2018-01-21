use std::mem;
use std::fmt::{self,Debug};

use neli::{Nl,NlSerState,NlDeState};
use neli::err::{SerError,DeError};

#[derive(Clone)]
pub enum AcpiGenlAttr {
    Unspec = 0,
    Event = 1,
    UnrecognizedVariant,
}

impl Default for AcpiGenlAttr {
    fn default() -> Self {
        AcpiGenlAttr::Unspec
    }
}

impl Nl for AcpiGenlAttr {
    fn serialize(&mut self, state: &mut NlSerState) -> Result<(), SerError> {
        let mut val = self.clone() as u16;
        val.serialize(state)
    }

    fn deserialize(state: &mut NlDeState) -> Result<Self, DeError> {
        let val = u16::deserialize(state)?;
        Ok(match val {
            i if i == 0 => AcpiGenlAttr::Unspec,
            i if i == 1 => AcpiGenlAttr::Event,
            _ => AcpiGenlAttr::UnrecognizedVariant,
        })
    }

    fn size(&self) -> usize {
        mem::size_of::<u16>()
    }
}

pub struct AcpiEvent {
    pub device_class: Vec<u8>,
    pub bus_id: Vec<u8>,
    pub event_type: u32,
    pub event_data: u32,
}

impl Debug for AcpiEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"AcpiEvent {{ device_class: {}, bus_id: {}, event_type: {}, event_data: {} }}"#,
               String::from_utf8_lossy(self.device_class.as_slice()),
               String::from_utf8_lossy(self.bus_id.as_slice()),
               self.event_type, self.event_data)
    }
}

impl Default for AcpiEvent {
    fn default() -> Self {
        AcpiEvent {
            device_class: Vec::new(),
            bus_id: Vec::new(),
            event_type: 0,
            event_data: 0,
        }
    }
}

impl Nl for AcpiEvent {
    fn serialize(&mut self, state: &mut NlSerState) -> Result<(), SerError> {
        self.device_class.serialize(state)?;
        self.bus_id.serialize(state)?;
        self.event_type.serialize(state)?;
        self.event_data.serialize(state)?;
        Ok(())
    }

    fn deserialize(state: &mut NlDeState) -> Result<Self, DeError> {
        let mut acpi_event = AcpiEvent::default();
        state.set_usize(20);
        acpi_event.device_class = Vec::<u8>::deserialize(state)?;
        state.set_usize(15);
        acpi_event.bus_id = Vec::<u8>::deserialize(state)?;
        acpi_event.event_type = u32::deserialize(state)?;
        acpi_event.event_data = u32::deserialize(state)?;
        Ok(acpi_event)
    }

    fn size(&self) -> usize {
        self.device_class.len() + self.bus_id.len()
            + self.event_type.size() + self.event_data.size()
    }
}

#[cfg(test)]
mod test {
    extern crate byteorder;

    use super::*;
    use std::io::{Cursor,Write};
    use self::byteorder::{WriteBytesExt,NativeEndian};

    #[test]
    fn test_acpi_event_serialize() {
        let mut acpi_event_serialized = Cursor::new(Vec::new());
        acpi_event_serialized.write(&vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9]).unwrap();
        acpi_event_serialized.write(&vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4]).unwrap();
        acpi_event_serialized.write_u32::<NativeEndian>(5).unwrap();
        acpi_event_serialized.write_u32::<NativeEndian>(7).unwrap();

        let mut acpi_event = AcpiEvent {
            device_class: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            bus_id: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4],
            event_type: 5,
            event_data: 7,
        };
        let mut state = NlSerState::new();
        acpi_event.serialize(&mut state).unwrap();

        assert_eq!(state.into_inner(), acpi_event_serialized.into_inner());
    }
}
