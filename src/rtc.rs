use stm32f103xx::rtc;


/*
    Configuring RTC registers requires the following process:

    poll RTOFF, make sure it is 1
    set CNF to enter config mode
    write to the registers
    clear CNF
    poll RTOFF to make sure it's back to 1
*/

struct Rtc {
    regs: rtc::RegisterBlock;
}


impl Rtc {
    fn perform_write(&mut self, regs, func: impl Fn(&mut Self)) {
        // This process is documented on page 485 of the stm32f103 manual
        // Wait for the last write operation to be done
        while self.regs.clr.read().rtoff().bit() == false {}
        // Put the clock into config mode
        self.regs.crl.modify(|w| w.cnf().set_bit());

        // Perform the write opertaion
        func(self);

        // Take the device out of config mode
        self.regs.crl.modify(|w| w.cnf().clear_bit());
        // Wait for the write to be done
        while self.regs.clr.read().rtoff().bit() == false {}
    }

    fn clear_alarm_flag(&mut self) {
        self.perform_write(|s| s.regs.crl.modify(|w| w.alrf().clear_bit()))
    }
}
