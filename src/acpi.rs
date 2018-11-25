use std::mem;

use neli::{Nl,StreamReadBuffer,StreamWriteBuffer};
use neli::err::{SerError,DeError};

#[derive(Clone)]
pub enum AcpiGenlAttr {
    Unspec = 0,
    Event = 1,
    UnrecognizedVariant,
}

impl Nl for AcpiGenlAttr {
    type SerIn = ();
    type DeIn = ();

    fn serialize(&self, mem: &mut StreamWriteBuffer) -> Result<(), SerError> {
        let val = self.clone() as u16;
        val.serialize(mem)
    }

    fn deserialize<B>(mem: &mut StreamReadBuffer<B>) -> Result<Self, DeError> where B: AsRef<[u8]> {
        let val = u16::deserialize(mem)?;
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

#[derive(Debug)]
pub struct AcpiEvent {
    pub device_class: String,
    pub bus_id: String,
    pub event_type: u32,
    pub event_data: u32,
}

impl Nl for AcpiEvent {
    type SerIn = ();
    type DeIn = ();

    fn serialize(&self, mem: &mut StreamWriteBuffer) -> Result<(), SerError> {
        self.device_class.serialize_with(mem, 20)?;
        self.bus_id.serialize_with(mem, 15)?;
        self.event_type.serialize(mem)?;
        self.event_data.serialize(mem)?;
        Ok(())
    }

    fn deserialize<B>(mem: &mut StreamReadBuffer<B>) -> Result<Self, DeError> where B: AsRef<[u8]> {
        Ok(AcpiEvent {
            device_class: String::deserialize_with(mem, 20)?,
            bus_id: String::deserialize_with(mem, 15)?,
            event_type: u32::deserialize(mem)?,
            event_data: u32::deserialize(mem)?,
        })
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
        acpi_event_serialized.write(&vec![65, 65, 65, 65, 65, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                    0, 0, 0, 0, 0]).unwrap();
        acpi_event_serialized.write(&vec![65, 65, 65, 65, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        acpi_event_serialized.write_u32::<NativeEndian>(5).unwrap();
        acpi_event_serialized.write_u32::<NativeEndian>(7).unwrap();

        let acpi_event = AcpiEvent {
            device_class: "AAAAAA".to_string(),
            bus_id: "AAAAA".to_string(),
            event_type: 5,
            event_data: 7,
        };
        let mut state = StreamWriteBuffer::new_growable(None);
        acpi_event.serialize(&mut state).unwrap();

        assert_eq!(state.as_ref(), acpi_event_serialized.get_ref().as_slice());
    }
}
