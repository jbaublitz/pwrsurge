use nl::{Nl,NlSerState,NlDeState};
use nl::err::{SerError,DeError};

pub struct AcpiEvent {
    device_class: Vec<u8>,
    bus_id: Vec<u8>,
    event_type: u32,
    event_data: u32,
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
