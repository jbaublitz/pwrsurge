pub struct AcpiFilter {
    device_class_whitelist: Vec<String>,
}

impl AcpiFilter {
    pub fn new(whitelist: Vec<String>) -> Self {
        AcpiFilter { device_class_whitelist: whitelist }
    }

    pub fn contains_device_class(&self, dev_class: &String) -> bool {
        self.device_class_whitelist.contains(dev_class)
    }

    pub fn is_wildcard(&self) -> bool {
        self.device_class_whitelist.len() == 0
    }
}

pub struct EvdevFilter {
    evdev_type_whitelist: Vec<u16>,
    evdev_code_whitelist: Vec<u16>,
}

impl EvdevFilter {
    pub fn new(type_whitelist: Vec<u16>, code_whitelist: Vec<u16>) -> Self {
        EvdevFilter { evdev_type_whitelist: type_whitelist,
                      evdev_code_whitelist: code_whitelist,
        }
    }

    pub fn contains_type(&self, ty: &u16) -> bool {
        self.evdev_type_whitelist.contains(ty)
    }

    pub fn contains_code(&self, code: &u16) -> bool {
        self.evdev_code_whitelist.contains(code)
    }

    pub fn is_wildcard(&self) -> bool {
        self.evdev_type_whitelist.len() == 0
            && self.evdev_code_whitelist.len() == 0
    }
}
