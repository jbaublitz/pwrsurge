pub struct AcpiFilter {
    device_class_whitelist: Vec<String>,
}

impl AcpiFilter {
    pub fn new(whitelist: Vec<String>) -> Self {
        AcpiFilter {
            device_class_whitelist: whitelist,
        }
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
    evdev_value_whitelist: Vec<i32>,
}

impl EvdevFilter {
    pub fn new(
        type_whitelist: Vec<u16>,
        code_whitelist: Vec<u16>,
        value_whitelist: Vec<i32>,
    ) -> Self {
        EvdevFilter {
            evdev_type_whitelist: type_whitelist,
            evdev_code_whitelist: code_whitelist,
            evdev_value_whitelist: value_whitelist,
        }
    }

    pub fn contains_type(&self, ty: &u16) -> bool {
        self.evdev_type_whitelist.contains(ty) || self.evdev_type_whitelist.is_empty()
    }

    pub fn contains_code(&self, code: &u16) -> bool {
        self.evdev_code_whitelist.contains(code) || self.evdev_code_whitelist.is_empty()
    }

    pub fn contains_value(&self, value: &i32) -> bool {
        self.evdev_value_whitelist.contains(value) || self.evdev_value_whitelist.is_empty()
    }

    pub fn is_wildcard(&self) -> bool {
        self.evdev_type_whitelist.is_empty()
            && self.evdev_code_whitelist.is_empty()
            && self.evdev_value_whitelist.is_empty()
    }
}
