use stm32f103xx::{rtc, rcc};


/*
    Configuring RTC registers requires the following process:

    poll RTOFF, make sure it is 1
    set CNF to enter config mode
    write to the registers
    clear CNF
    poll RTOFF to make sure it's back to 1


    The frequency of the rtc is calculated by
    f_rtcclk / (PRL[19:0] + 1)

    f_rtcclk can probably be configured to be the main clock. Then we can use
    rcc to figure out what that frequency is

    both vbat and vdd might have to be connected for the clock to keep counting
*/

pub struct Rtc {
    regs: rtc::RegisterBlock
}


impl Rtc {
    pub fn rtc(regs: rtc::RegisterBlock, rcc: &mut rcc::RegisterBlock) -> Self {
        // Configure the perscaler to use the LSE clock as defined in the documentation
        // in section 7.2.4. This gives a 32.768khz frequency for the RTC
        rcc.bdcr.modify(|_, w| {
            w
                // Enable external low speed oscilator
                .lseon().set_bit()
                // Enable the RTC
                .rtcen().set_bit()
                // Set the source of the RTC to LSE
                .rtcsel().lse()
        });

        // Set the prescaler to make it count up once every second
        // The manual on page 490 says that the prescaler value for this should be 7fffh
        regs.prll.write(|w| unsafe{w.bits(0x7fff)});
        regs.prlh.write(|w| unsafe{w.bits(0)});

        Rtc {
            regs
        }
    }

    pub fn set_alarm(&mut self, time_seconds: u32) {
        self.perform_write(|s| {
            // Enable alarm interrupt
            s.regs.crh.modify(|_, w| w.alrie().set_bit());

            // Reset counter
            s.regs.cnth.write(|w| unsafe{w.bits(0)});
            s.regs.cntl.write(|w| unsafe{w.bits(0)});

            // Set alarm time
            s.regs.alrh.write(|w| unsafe{w.alrh().bits((time_seconds >> 16) as u16)});
            s.regs.alrl.write(|w| unsafe{w.alrl().bits((time_seconds & 0x0000ffff) as u16)});
        })
    }

    pub fn read(&self) -> u32{
        // Wait for the APB1 interface to be ready
        while self.regs.crl.read().rsf().bit() == false {}

        ((self.regs.cnth.read().bits() << 16) as u32) + (self.regs.cntl.read().bits() as u32)
    }


    fn perform_write(&mut self, func: impl Fn(&mut Self)) {
        // This process is documented on page 485 of the stm32f103 manual
        // Wait for the last write operation to be done
        while self.regs.crl.read().rtoff().bit() == false {}
        // Put the clock into config mode
        self.regs.crl.modify(|_, w| w.cnf().set_bit());

        // Perform the write opertaion
        func(self);

        // Take the device out of config mode
        self.regs.crl.modify(|_, w| w.cnf().clear_bit());
        // Wait for the write to be done
        while self.regs.crl.read().rtoff().bit() == false {}
    }

    fn clear_alarm_flag(&mut self) {
        self.perform_write(|s| s.regs.crl.modify(|_, w| w.alrf().clear_bit()))
    }
}