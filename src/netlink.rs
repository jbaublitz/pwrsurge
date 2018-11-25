use acpi::AcpiEvent;

use neli::socket::NlSocket;
use neli::nl::Nlmsghdr;
use neli::genl::Genlmsghdr;
use neli::consts::{GenlId,CtrlCmd};
use neli::err::{NlError};

pub fn resolve_acpi_family_id() -> Result<u32, NlError> {
    let mut s = NlSocket::<GenlId, Genlmsghdr<CtrlCmd>>::new_genl()?;
    let id = s.resolve_nl_mcast_group("acpi_event", "acpi_mc_group")?;
    Ok(id)
}

pub fn acpi_event(msg: Nlmsghdr<GenlId, Genlmsghdr<CtrlCmd>>) -> Result<AcpiEvent, NlError> {
    let mut attr_handle = msg.nl_payload.get_attr_handle::<u16>();
    Ok(attr_handle.get_payload_with::<AcpiEvent>(1, None)?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn test_resolve_acpi_family_id() {
        let id = resolve_acpi_family_id().unwrap();
        assert_eq!(id, 8)
    }
}
