pub(crate) struct InterruptController {
    pub master_enable: bool,
    pub interrupt_enable_flags: u8,
    pub interrupt_request_flags: u8,
}

impl InterruptController {
    pub fn new() -> InterruptController {
        InterruptController {
            master_enable: false,
            interrupt_enable_flags: 0,
            interrupt_request_flags: 0,
        }
    }
}
