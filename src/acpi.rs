use std::mem;

use neli::consts::{CtrlCmd, GenlId};
use neli::err::{DeError, NlError, SerError};
use neli::genl::Genlmsghdr;
use neli::nl::Nlmsghdr;
use neli::{Nl, StreamReadBuffer, StreamWriteBuffer};

pub fn acpi_event(msg: Nlmsghdr<GenlId, Genlmsghdr<CtrlCmd, u16>>) -> Result<AcpiEvent, NlError> {
    let attr_handle = msg.nl_payload.get_attr_handle();
    Ok(attr_handle.get_attr_payload_as::<AcpiEvent>(1)?)
}

#[derive(Clone)]
pub enum AcpiGenlAttr {
    Unspec = 0,
    Event = 1,
    UnrecognizedVariant,
}

impl Nl for AcpiGenlAttr {
    fn serialize(&self, mem: &mut StreamWriteBuffer) -> Result<(), SerError> {
        let val = self.clone() as u16;
        val.serialize(mem)
    }

    fn deserialize<B>(mem: &mut StreamReadBuffer<B>) -> Result<Self, DeError>
    where
        B: AsRef<[u8]>,
    {
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

#[derive(Debug, PartialEq)]
pub struct AcpiEvent {
    pub device_class: String,
    pub bus_id: String,
    pub event_type: u32,
    pub event_data: u32,
}

impl AcpiEvent {
    // This is the total size of both string buffer sizes in the C struct
    const MAGIC_STRING_SIZE_CONST: usize = 36;
}

impl Nl for AcpiEvent {
    fn serialize(&self, mem: &mut StreamWriteBuffer) -> Result<(), SerError> {
        mem.set_size_hint(20);
        self.device_class.serialize(mem)?;
        mem.set_size_hint(16);
        self.bus_id.serialize(mem)?;
        self.event_type.serialize(mem)?;
        self.event_data.serialize(mem)?;
        Ok(())
    }

    fn deserialize<B>(mem: &mut StreamReadBuffer<B>) -> Result<Self, DeError>
    where
        B: AsRef<[u8]>,
    {
        Ok(AcpiEvent {
            device_class: {
                mem.set_size_hint(20);
                String::deserialize(mem)?
            },
            bus_id: {
                mem.set_size_hint(16);
                String::deserialize(mem)?
            },
            event_type: u32::deserialize(mem)?,
            event_data: u32::deserialize(mem)?,
        })
    }

    fn size(&self) -> usize {
        Self::MAGIC_STRING_SIZE_CONST + self.event_type.size() + self.event_data.size()
    }
}

#[cfg(test)]
mod test {
    extern crate byteorder;

    use self::byteorder::{NativeEndian, WriteBytesExt};
    use super::*;
    use std::io::{Cursor, Write};

    #[test]
    fn test_acpi_event_serialize() {
        let mut acpi_event_serialized = Cursor::new(Vec::new());
        acpi_event_serialized
            .write(&vec![
                65, 65, 65, 65, 65, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ])
            .unwrap();
        acpi_event_serialized
            .write(&vec![65, 65, 65, 65, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
            .unwrap();
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

    #[test]
    fn test_acpi_event_deserialize() {
        let acpi_event_deserialized = AcpiEvent {
            device_class: "AAAAAA".to_string(),
            bus_id: "AAAAA".to_string(),
            event_type: 5,
            event_data: 7,
        };

        let mut acpi_event_buffer =
            StreamWriteBuffer::new_growable(Some(acpi_event_deserialized.size()));
        acpi_event_buffer
            .write(&vec![
                65, 65, 65, 65, 65, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ])
            .unwrap();
        acpi_event_buffer
            .write(&vec![65, 65, 65, 65, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
            .unwrap();
        acpi_event_buffer.write_u32::<NativeEndian>(5).unwrap();
        acpi_event_buffer.write_u32::<NativeEndian>(7).unwrap();

        let mut acpi_event_serialized = StreamReadBuffer::new(acpi_event_buffer);

        let acpi_event = AcpiEvent::deserialize(&mut acpi_event_serialized).unwrap();

        assert_eq!(acpi_event, acpi_event_deserialized);
    }
}
