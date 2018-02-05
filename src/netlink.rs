use std::str;

use acpi::AcpiEvent;

use neli::socket::NlSocket;
use neli::nlhdr::{NlHdr,NlAttrHdr};
use neli::genlhdr::{GenlHdr};
use neli::ffi::{NlFamily,Nlmsg,NlFlags,CtrlAttr,GenlId,CtrlCmd,CtrlAttrMcastGrp};
use neli::err::{NlError};

const ACPI_FAMILY_NAME: &'static str = "acpi_event";

pub fn resolve_acpi_family_id() -> Result<u32, NlError> {
    let mut s = NlSocket::new(NlFamily::Generic)?;
    let attrs = vec![NlAttrHdr::new_str_payload(None, CtrlAttr::FamilyName,
                     ACPI_FAMILY_NAME)?];
    let genl_hdr = GenlHdr::new(CtrlCmd::Getfamily, 2, attrs)?;
    let msg = NlHdr::<GenlId, GenlHdr>::new(None, GenlId::Ctrl,
                                              vec![NlFlags::Request, NlFlags::Ack],
                                              None, None, genl_hdr);
    s.sendmsg(msg, 0)?;
    let resp = s.recvmsg::<Nlmsg, GenlHdr>(Some(4096), 0)?;
    let mut resp_handle = resp.nl_payload.get_attr_handle::<CtrlAttr>();
    let mut mcastgroups = resp_handle.get_nested_attributes::<u16>(CtrlAttr::McastGroups)?;
    let mut mcastgroup = mcastgroups.get_nested_attributes::<CtrlAttrMcastGrp>(1u16)?;
    let id = mcastgroup.get_payload_as::<u32>(CtrlAttrMcastGrp::Id)?;
    Ok(id)
}

pub fn acpi_listen(s: &mut NlSocket) -> Result<AcpiEvent, NlError> {
    let msg = s.recvmsg::<Nlmsg, GenlHdr>(Some(4096), 0)?;
    let mut attr_handle = msg.nl_payload.get_attr_handle::<u16>();
    Ok(attr_handle.get_payload_as::<AcpiEvent>(1)?)
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
