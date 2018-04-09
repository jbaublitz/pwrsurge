use std::str;

use acpi::AcpiEvent;

use neli::{Nl,MemWrite};
use neli::socket::NlSocket;
use neli::nlhdr::{NlHdr,NlAttrHdr};
use neli::genlhdr::{GenlHdr};
use neli::ffi::{NlFamily,NlmF,CtrlAttr,GenlId,CtrlCmd,CtrlAttrMcastGrp};
use neli::err::{NlError};

const ACPI_FAMILY_NAME: &'static str = "acpi_event";

pub fn resolve_acpi_family_id() -> Result<u32, NlError> {
    let mut s = NlSocket::<GenlId, GenlHdr>::new(NlFamily::Generic)?;
    let attrs = vec![NlAttrHdr::new_str_payload(None, CtrlAttr::FamilyName,
                     ACPI_FAMILY_NAME)?];
    let genl_hdr = GenlHdr::new(CtrlCmd::Getfamily, 2, attrs)?;
    let mut mem = MemWrite::new_vec(Some(4096));
    let msg = NlHdr::<GenlId, GenlHdr>::new(None, GenlId::Ctrl,
                                            vec![NlmF::Request, NlmF::Ack],
                                            None, None, genl_hdr);
    msg.serialize(&mut mem)?;
    let mut mem_read = mem.into();
    s.send(mem_read, 0)?;

    let mem_resp = MemWrite::new_vec(Some(4096));
    let mut mem_read_resp = s.recv(mem_resp, 0)?;
    let resp = NlHdr::<GenlId, GenlHdr>::deserialize(&mut mem_read_resp)?;
    let mut resp_handle = resp.nl_payload.get_attr_handle::<CtrlAttr>();
    let mut mcastgroups = resp_handle.get_nested_attributes::<u16>(CtrlAttr::McastGroups)?;
    let mut mcastgroup = mcastgroups.get_nested_attributes::<CtrlAttrMcastGrp>(1u16)?;
    let id = mcastgroup.get_payload_as::<u32>(CtrlAttrMcastGrp::Id)?;
    Ok(id)
}

pub fn acpi_event(msg: NlHdr<GenlId, GenlHdr>) -> Result<AcpiEvent, NlError> {
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
