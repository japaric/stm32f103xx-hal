//! ENC28J60 demo: a RESTful LED using CoAP
//!
//! The server will expose the LED as a resource under the `/led` path. You can use the CoAP client
//! in the [`jnet`] crate to interact with the server.
//!
//! - `coap GET coap://192.168.1.33/led` will return the state of the LED: either "on" or "off".
//! - `coap PUT coap://192.168.1.33/led on` will change the state of the LED; the payload must be
//!   either "on" or "off".
//!
//! [`jnet`]: https://github.com/japaric/jnet

#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(lang_items)]
#![feature(nll)]
#![feature(try_from)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate enc28j60;
extern crate heapless;
extern crate jnet;
extern crate stm32f103xx_hal as hal;

use core::convert::TryInto;

use enc28j60::Enc28j60;
use hal::delay::Delay;
use hal::prelude::*;
use hal::spi::Spi;
use hal::stm32f103xx;
use heapless::LinearMap;
use jnet::{arp, coap, ether, icmp, mac, udp, Buffer, ipv4};

/* Constants */
const KB: u16 = 1024;

/* Network configuration */
const MAC: mac::Addr = mac::Addr([0x20, 0x18, 0x03, 0x01, 0x00, 0x00]);
const IP: ipv4::Addr = ipv4::Addr([192, 168, 1, 33]);

// disable tracing
// macro_rules! iprintln {
//     ($($tt: tt)*) => {};
// }

fn main() {
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f103xx::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut flash = dp.FLASH.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let stim = &mut cp.ITM.stim[0];

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // LED
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // turn the LED off during initialization
    led.set_high();

    // SPI
    let mut rst = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
    rst.set_high();
    let mut ncs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    ncs.set_high();
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        enc28j60::MODE,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    // ENC28J60
    let mut delay = Delay::new(cp.SYST, clocks);
    let mut enc28j60 = Enc28j60::new(
        spi,
        ncs,
        enc28j60::Unconnected,
        rst,
        &mut delay,
        7 * KB,
        MAC.0,
    ).ok()
        .unwrap();

    // LED on after initialization
    led.set_low();

    // FIXME some frames are lost when sending right after initialization
    delay.delay_ms(100_u8);

    let mut cache = LinearMap::<_, _, [_; 8]>::new();

    let mut buf = [0; 128];
    loop {
        let mut buf = Buffer::new(&mut buf);
        let len = enc28j60.receive(buf.as_mut()).ok().unwrap();
        buf.truncate(len);

        if let Ok(mut eth) = ether::Frame::parse(buf) {
            iprintln!(stim, "\nRx({})", eth.as_bytes().len());
            iprintln!(stim, "* {:?}", eth);

            let mac_src = eth.get_source();

            match eth.get_type() {
                ether::Type::Arp => {
                    if let Ok(arp) = arp::Packet::parse(eth.payload_mut()) {
                        match arp.downcast() {
                            Ok(mut arp) => {
                                iprintln!(stim, "** {:?}", arp);

                                if !arp.is_a_probe() {
                                    cache.insert(arp.get_spa(), arp.get_sha()).ok();
                                }

                                // are they asking for us?
                                if arp.get_oper() == arp::Operation::Request && arp.get_tpa() == IP
                                {
                                    // reply the ARP request
                                    let tha = arp.get_sha();
                                    let tpa = arp.get_spa();

                                    arp.set_oper(arp::Operation::Reply);
                                    arp.set_sha(MAC);
                                    arp.set_spa(IP);
                                    arp.set_tha(tha);
                                    arp.set_tpa(tpa);
                                    iprintln!(stim, "\n** {:?}", arp);

                                    // update the Ethernet header
                                    eth.set_destination(tha);
                                    eth.set_source(MAC);
                                    iprintln!(stim, "* {:?}", eth);

                                    iprintln!(stim, "Tx({})", eth.as_bytes().len());
                                    enc28j60.transmit(eth.as_bytes()).ok().unwrap();
                                }
                            }
                            Err(arp) => {
                                iprintln!(stim, "** {:?}", arp);
                            }
                        }
                    } else {
                        iprintln!(stim, "Err(B)");
                    }
                }
                ether::Type::Ipv4 => {
                    if let Ok(mut ip) = ipv4::Packet::parse(eth.payload_mut()) {
                        iprintln!(stim, "** {:?}", ip);

                        let ip_src = ip.get_source();

                        if !mac_src.is_broadcast() {
                            cache.insert(ip_src, mac_src).ok();
                        }

                        match ip.get_protocol() {
                            ipv4::Protocol::Icmp => {
                                if let Ok(mut icmp) = icmp::Packet::parse(ip.payload_mut()) {
                                    iprintln!(stim, "*** {:?}", icmp);

                                    if icmp.get_type() == icmp::Type::EchoRequest
                                        && icmp.get_code() == 0
                                    {
                                        let icmp =
                                            icmp.set_type(icmp::Type::EchoReply).update_checksum();
                                        iprintln!(stim, "\n*** {:?}", icmp);

                                        // update the IP header
                                        let mut ip = ip.set_source(IP);
                                        ip.set_destination(ip_src);
                                        let ip = ip.update_checksum();
                                        iprintln!(stim, "** {:?}", ip);

                                        // update the Ethernet header
                                        eth.set_destination(*cache.get(&ip_src).unwrap());
                                        eth.set_source(MAC);
                                        iprintln!(stim, "* {:?}", eth);

                                        iprintln!(stim, "Tx({})", eth.as_bytes().len());
                                        enc28j60.transmit(eth.as_bytes()).ok().unwrap();
                                    }
                                } else {
                                    iprintln!(stim, "Err(C)");
                                }
                            }
                            ipv4::Protocol::Udp => {
                                if let Ok(mut udp) = udp::Packet::parse(ip.payload_mut()) {
                                    iprintln!(stim, "*** {:?}", udp);

                                    if udp.get_destination() == coap::PORT {
                                        if let Ok(mut coap) =
                                            coap::Message::parse(udp.payload_mut())
                                        {
                                            iprintln!(stim, "**** {:?}", coap);

                                            let path_is_led = coap.options()
                                                .filter_map(|opt| {
                                                    if opt.number() == coap::OptionNumber::UriPath {
                                                        Some(opt.value())
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .eq([b"led"].iter().cloned());

                                            // update the CoAP message
                                            coap.set_type(coap::Type::Acknowledgement);

                                            match coap.get_code().try_into() {
                                                Ok(coap::Method::Get) => {
                                                    if path_is_led {
                                                        coap.set_code(coap::Response::Content);

                                                        coap.clear_options();
                                                        coap.set_payload(if led.is_low() {
                                                            b"on"
                                                        } else {
                                                            b"off"
                                                        });
                                                    } else {
                                                        coap.set_code(coap::Response::BadRequest);
                                                    }
                                                }
                                                Ok(coap::Method::Put) => {
                                                    let mut ok = false;
                                                    if path_is_led {
                                                        match coap.payload() {
                                                            b"on" => {
                                                                led.set_low();
                                                                ok = true;
                                                            }
                                                            b"off" => {
                                                                led.set_high();
                                                                ok = true;
                                                            }
                                                            _ => {}
                                                        }
                                                    }

                                                    coap.clear_options();
                                                    if ok {
                                                        coap.set_code(coap::Response::Changed);
                                                    } else {
                                                        coap.set_code(coap::Response::BadRequest);
                                                    }
                                                    coap.set_payload(&[]);
                                                }
                                                _ => {}
                                            }

                                            iprintln!(stim, "\n**** {:?}", coap);

                                            // update the UDP header
                                            let coap_len = coap.len();
                                            let udp_src = udp.get_source();
                                            udp.truncate(coap_len);
                                            udp.set_source(coap::PORT);
                                            udp.set_destination(udp_src);
                                            udp.zero_checksum();
                                            iprintln!(stim, "*** {:?}", udp);

                                            // update the IP header
                                            let udp_len = udp.len();
                                            let mut ip = ip.set_source(IP);
                                            ip.set_destination(ip_src);
                                            ip.truncate(udp_len);
                                            let ip = ip.update_checksum();
                                            iprintln!(stim, "** {:?}", ip);

                                            // update the Ethernet header
                                            let ip_len = ip.len();
                                            eth.set_destination(*cache.get(&ip_src).unwrap());
                                            eth.set_source(MAC);
                                            eth.truncate(ip_len);
                                            iprintln!(stim, "* {:?}", eth);

                                            let bytes = eth.as_bytes();
                                            iprintln!(stim, "Tx({})", bytes.len());
                                            enc28j60.transmit(bytes).ok().unwrap();
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        iprintln!(stim, "Err(D)");
                    }
                }
                _ => {}
            }
        } else {
            iprintln!(stim, "Err(E)");
        }
    }
}
