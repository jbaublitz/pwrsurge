use std::slice::from_raw_parts;
use std::str::from_utf8_unchecked;

#[no_mangle]
pub extern "C" fn battery(bus_id: *const u8, event_type: u32, event_data: u32) -> i32 {
    println!("bus_id: {}\nevent_type: {}\nevent_data: {}",
             unsafe { from_utf8_unchecked(from_raw_parts(bus_id, 15)) }, event_type, event_data);
    0
}
