// #![deny(unsafe_code)]
#![no_main]
#![no_std]
// use aux5::entry;

// 安装Cargo bin工具 cargo install cargo-binutils
// rustup component add llvm-tools-preview

// cargo build --target thumbv7em-none-eabihf
// cargo readobj --target thumbv7em-none-eabihf --bin stm32 -- --file-header

// Flush to stm32
// openocd -s /c:/"Program Files (x86)"/xpack-openocd-0.11.0-4/scripts -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg
// /c:/"Program Files (x86)"/xpack-openocd-0.11.0-4

// cargo run -- -q -ex 'target remote :3333' -ex 'load' -ex 'set print asm-demangle on' -ex 'set style sources off' -ex 'b main' -ex 'c' target/thumbv7em-none-eabihf/debug/stm32

// https://github.com/stm32-rs/stm32f1xx-hal
// https://zhuanlan.zhihu.com/p/51733085
// https://www.bilibili.com/read/cv16680191/

// pick a panicking behavior
//! Blinks an LED
//!
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.
//!
//! Note: Without additional hardware, PC13 should not be used to drive an LED, see page 5.1.2 of
//! the reference manual for an explanation. This is not an issue on the blue pill.

use panic_halt as _;

use core::ptr;

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*, timer::Timer};

// 引入println等
pub use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM};
pub use stm32f1xx_hal::stm32::{self, gpioa::RegisterBlock};

// use cortex_m_semihosting::hprintln;

// 通过gpioc-brr操作led灯
fn blink_by_brr(v:bool){
  let gpioc = unsafe { &*stm32::GPIOC::ptr() };
  if v {
    gpioc.bsrr.write(|w| w.bs13().set_bit());
  }else{
    gpioc.bsrr.write(|w| w.br13().set_bit());
  }

  // cortex_m::asm::bkpt();
}

// 通过寄存器地址操作led灯
// GPIO对应一个内存偏移地址：0x4001_1000，GPIOC（stm32f103LED灯对应pc13）偏移：0x10
fn blink_by_addr(v:bool){
  unsafe{
    // Memory map and register boundary addresses. Page 51,
    // Offset. Page 168
    // pc13 寄存器地址
    const GPIOC_BSRR: u32 = 0x4001_1000 + 0x10; // 寄存器地址+BSRR偏移地址

    // Trun off all LED
    let offset = if v { 1 << 13}  else {1 << 13 + 16};
    ptr::write_volatile(GPIOC_BSRR as *mut u32, offset);
  }
}

// 通过rcc处理RST开关
fn blink_by_moder_and_rcc(){
  let rcc = unsafe { &*stm32::RCC::ptr() };
  rcc.ahbenr.write(|w| w.crcen().set_bit());
}

// heml
#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let mut cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIOC peripheral

    // // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // // in order to configure the port. For pins 0-7, crl should be passed instead.


    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // blink_by_moder_and_rcc();
    //   let mut gpiob = dp.GPIOB.split();
    // let mut led = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);

    // let mut gpiod = dp.GPIOD.split();
    // let mut led = gpiod.pd2.into_push_pull_output(&mut gpiod.crl);

    iprintln!(&mut (cp.ITM.stim[0]), "hello world");
    // iprintln!(&mut (cp.ITM.stim[1]),"hello world");
    // hprintln!("Hello, world!").unwrap();

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();

    // Wait for the timer to trigger an update and change the state of the LED
    // gpioe.bsrr.write(|w| w.br13().set_bit());
    // gpioe.bsrr.write(|w| w.bs13().set_bit());

    led.set_high();
  
    loop {
        block!(timer.wait()).unwrap();

        // led.set_high();
        blink_by_brr(true);
        block!(timer.wait()).unwrap();
        // led.set_low();
        blink_by_brr(false);

    }
}
