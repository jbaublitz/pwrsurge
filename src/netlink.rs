use nl::socket::NlSocket;
use nl::nlhdr::{NlHdr,NlAttrHdr};
use nl::genlhdr::{GenlHdr};
use nl::ffi::{NlFamily,NlType,NlFlags,NlaType,GenlType,GenlCmds,AttrTypeMcast};
use nl::err::{NlError};

const ACPI_FAMILY_NAME: &'static str = "acpi_event";

pub fn acpi_listen() -> Result<(), NlError> {
    let id = resolve_acpi_family_id()?;
    let mut s = NlSocket::connect(NlFamily::NlGeneric, None, Some(1 << (id - 1)))?;
    loop {
        let msg = s.recvmsg::<NlType, GenlHdr>(Some(4096), 0)?;
        println!("{:?}", msg);
    }
}

pub fn resolve_acpi_family_id() -> Result<u32, NlError> {
    let mut s = NlSocket::new(NlFamily::NlGeneric)?;
    let attrs = vec![NlAttrHdr::new_str_payload(None, NlaType::AttrFamilyName,
                     ACPI_FAMILY_NAME)];
    let genl_hdr = GenlHdr::new(GenlCmds::CmdGetfamily, 2, attrs)?;
    let msg = NlHdr::<GenlType, GenlHdr>::new(None, GenlType::IdCtrl,
                                              vec![NlFlags::NlRequest, NlFlags::NlAck],
                                              None, None, genl_hdr);
    s.sendmsg(msg, 0)?;
    let resp = s.recvmsg::<NlType, GenlHdr>(Some(4096), 0)?;
    let mut resp_handle = resp.nl_pl.get_attr_handle::<NlaType>();
    let mut mcastgroups = resp_handle.get_nested_attributes::<u16>(NlaType::AttrMcastGroups)?;
    let mut mcastgroup = mcastgroups.get_nested_attributes::<AttrTypeMcast>(1u16)?;
    let id = mcastgroup.get_payload_as::<u32>(AttrTypeMcast::GrpId)?;
    Ok(id)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_resolve_acpi_family_id() {
        let id = resolve_acpi_family_id().unwrap();
        assert_eq!(id, 8)
    }
}
