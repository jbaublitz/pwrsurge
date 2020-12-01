use neli::{
    consts::{genl::*, nl::*},
    deserialize,
    err::{DeError, NlError, SerError},
    genl::Genlmsghdr,
    impl_var,
    nl::{Nlmsghdr, NlPayload},
    serialize,
    types::{DeBuffer, SerBuffer},
    Nl,
};

pub fn acpi_event(msg: Nlmsghdr<GenlId, Genlmsghdr<CtrlCmd, u16>>) -> Result<AcpiEvent, NlError> {
    let genl = match msg.nl_payload {
        NlPayload::Payload(genl) => genl,
        NlPayload::Err(e) => return Err(NlError::from(e)),
        NlPayload::Ack(_) => {
            return Err(NlError::new("Received unexpected ACK from netlink"));
        }
        NlPayload::Empty => {
            return Err(NlError::new("Received empty packet from netlink"));
        }
    };
    let attr_handle = genl.get_attr_handle();
    Ok(attr_handle.get_attr_payload_as::<AcpiEvent>(1)?)
}

impl_var!(
    pub AcpiGenlAttr,
    u16,
    Unspec => 0,
    Event => 1
);

#[derive(Debug, PartialEq)]
pub struct DeviceClass(pub String);

impl Nl for DeviceClass {
    fn serialize(&self, mem: SerBuffer) -> Result<(), SerError> {
        if mem.len() > self.size() {
            return Err(SerError::BufferNotFilled);
        } else if mem.len() < self.size() {
            return Err(SerError::UnexpectedEOB);
        }
        let position = self.0.size();
        self.0.serialize(&mut mem[..position])
    }

    fn deserialize(mem: DeBuffer) -> Result<Self, DeError> {
        if mem.len() > Self::type_size().expect("Constant size") {
            return Err(DeError::BufferNotParsed);
        } else if mem.len() < Self::type_size().expect("Constant size") {
            return Err(DeError::UnexpectedEOB);
        }
        let position = mem.iter().position(|elem| *elem == 0)
            .ok_or_else(|| {
                DeError::new("No null byte found in C string")
            })?;

        Ok(DeviceClass(String::deserialize(&mem[..position + 1])?))
    }

    fn size(&self) -> usize {
        Self::type_size().expect("Constant size")
    }

    fn type_size() -> Option<usize> {
        Some(20)
    }
}

#[derive(Debug, PartialEq)]
pub struct BusId(pub String);

impl Nl for BusId {
    fn serialize(&self, mem: SerBuffer) -> Result<(), SerError> {
        if mem.len() > self.size() {
            return Err(SerError::BufferNotFilled);
        } else if mem.len() < self.size() {
            return Err(SerError::UnexpectedEOB);
        }
        let position = self.0.size();
        self.0.serialize(&mut mem[..position])
    }

    fn deserialize(mem: DeBuffer) -> Result<Self, DeError> {
        if mem.len() > Self::type_size().expect("Constant size") {
            return Err(DeError::BufferNotParsed);
        } else if mem.len() < Self::type_size().expect("Constant size") {
            return Err(DeError::UnexpectedEOB);
        }
        let position = mem.iter().position(|elem| *elem == 0)
            .ok_or_else(|| {
                DeError::new("No null byte found in C string")
            })?;

        Ok(BusId(String::deserialize(&mem[..position + 1])?))
    }

    fn size(&self) -> usize {
        Self::type_size().expect("Constant size")
    }

    fn type_size() -> Option<usize> {
        Some(16)
    }
}

#[derive(Debug, PartialEq)]
pub struct AcpiEvent {
    pub device_class: DeviceClass,
    pub bus_id: BusId,
    pub event_type: u32,
    pub event_data: u32,
}

impl Nl for AcpiEvent {
    fn serialize(&self, mem: SerBuffer) -> Result<(), SerError> {
        serialize! {
            mem;
            self.device_class;
            self.bus_id;
            self.event_type;
            self.event_data
        };
        Ok(())
    }

    fn deserialize(mem: DeBuffer) -> Result<Self, DeError> {
        Ok(deserialize! {
            mem;
            AcpiEvent {
                device_class: DeviceClass,
                bus_id: BusId,
                event_type: u32,
                event_data: u32
            }
        })
    }

    fn size(&self) -> usize {
        self.device_class.size() + self.bus_id.size() + self.event_type.size() + self.event_data.size()
    }
    
    fn type_size() -> Option<usize> {
        DeviceClass::type_size()
            .and_then(|dcs| BusId::type_size().map(|bs| dcs + bs))
            .and_then(|acc| u32::type_size().map(|us| acc + us * 2))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::{Cursor, Write};

    use byteorder::{NativeEndian, WriteBytesExt};
    use neli::utils::serialize;

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
            device_class: DeviceClass("AAAAAA".to_string()),
            bus_id: BusId("AAAAA".to_string()),
            event_type: 5,
            event_data: 7,
        };
        let state = serialize(&acpi_event, false).unwrap();

        assert_eq!(state.as_slice(), acpi_event_serialized.get_ref().as_slice());
    }

    #[test]
    fn test_acpi_event_deserialize() {
        let acpi_event_deserialized = AcpiEvent {
            device_class: DeviceClass("AAAAAA".to_string()),
            bus_id: BusId("AAAAA".to_string()),
            event_type: 5,
            event_data: 7,
        };

        let mut acpi_event_buffer = Cursor::new(Vec::new());
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

        let acpi_event = AcpiEvent::deserialize(acpi_event_buffer.get_mut().as_mut_slice()).unwrap();

        assert_eq!(acpi_event, acpi_event_deserialized);
    }
}
