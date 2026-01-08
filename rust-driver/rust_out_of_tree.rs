// SPDX-License-Identifier: GPL-2.0

use core::pin::Pin;

use kernel::{
    c_str,
    fs::{File, Kiocb},
    iov::{IovIterSource},
    miscdevice::{MiscDevice, MiscDeviceOptions, MiscDeviceRegistration},
    new_mutex,
    prelude::*,
    sync::{Mutex},
    bindings::gpio_desc,
};

const MORSE_UNIT_US: u64 = 100_000;
const MORSE_FREQ: u64 = 1000;

module! {
    type: RustOutOfTree,
    name: "rust_out_of_tree",
    authors: ["Juraj Petras"],
    description: "Rust out-of-tree sample",
    license: "GPL",
}

#[pin_data]
struct RustOutOfTree {
    #[pin]
    _dev: MiscDeviceRegistration<RustMiscDevice>,
}

impl kernel::InPlaceModule for RustOutOfTree {
    fn init(_module: &'static ThisModule) -> impl PinInit<Self, Error> {
        pr_info!("Rust out-of-tree sample (init)\n");

        let options = MiscDeviceOptions {
            name: c_str!("rust_out_of_tree"),
        };

        try_pin_init!(Self {
            _dev <- MiscDeviceRegistration::register(options),
        })
    }
}

struct Inner {
    gpio: GpioOutputPin,
    buf: KVVec<u8>,
}

#[pin_data(PinnedDrop)]
struct RustMiscDevice {
    #[pin]
    inner: Mutex<Inner>,
}

#[vtable]
impl MiscDevice for RustMiscDevice {
    type Ptr = Pin<KBox<Self>>;

    fn open(_file: &File, _misc: &MiscDeviceRegistration<Self>) -> Result<Pin<KBox<Self>>> {
        pr_info!("Opening Rust Misc Device Sample\n");

        let gpio = match GpioOutputPin::new(17) {
            Ok(gpio) => gpio,
            Err(err) => {
                pr_err!("-> Failed to create GpioOutputPin: {:?}", err);
                return Err(err);
            }
        };

        pr_info!("-> Created GpioOutputPin\n");

        KBox::try_pin_init(
            try_pin_init! {
                RustMiscDevice {
                    inner <- new_mutex!(Inner {
                        gpio: gpio,
                        buf: KVVec::new(),
                    }),
                }
            },
            GFP_KERNEL,
        )
    }

    fn write_iter(mut kiocb: Kiocb<'_, Self::Ptr>, iov: &mut IovIterSource<'_>) -> Result<usize> {
        let me = kiocb.file();
        pr_info!("Writing to Rust Misc Device Sample\n");

        let mut inner = me.inner.lock();
        let len = iov.copy_from_iter_vec(&mut inner.buf, GFP_KERNEL)?;

        let Inner { gpio, buf } = &mut *inner;

        let mut input = core::str::from_utf8(buf.as_slice())?.trim();

        if let Some(last_newline) = input.rfind('\n') {
            input = &input[..last_newline + 1];
        }

        for line in input.lines() {
            let Some(cmd) = line.chars().next() else {
                continue;
            };

            let data = line.trim_start_matches(cmd);

            match cmd {
                't' => {
                    let (freq, duration_us) = match data.split_once(' ') {
                        Some((freq, duration_us)) => (freq, duration_us),
                        None => {
                            pr_err!("-> Invalid input\n");
                            return Err(EINVAL);
                        }
                    };

                    let freq = freq.parse::<u64>().map_err(|_| EINVAL)?;
                    let duration_us = duration_us.parse::<u64>().map_err(|_| EINVAL)?;
                    gpio.play_tone(freq, duration_us)?;
                }
                'm' => {
                    let text = data.trim();

                    for (i, ch) in text.chars().enumerate() {
                        let ch = ch.to_ascii_uppercase();
                        if ch == ' ' {
                            sleep_us(MORSE_UNIT_US * 7);
                            continue;
                        }

                        let code = morse_encode(ch).ok_or(EINVAL)?;

                        for (j, symbol) in code.chars().enumerate() {
                            let duration = match symbol {
                                '.' => MORSE_UNIT_US,
                                '-' => MORSE_UNIT_US * 3,
                                _ => return Err(EINVAL),
                            };

                            gpio.play_tone(MORSE_FREQ, duration)?;

                            if j + 1 < code.len() {
                                sleep_us(MORSE_UNIT_US); // intra-symbol gap
                            }
                        }

                        if i + 1 < text.len() {
                            sleep_us(MORSE_UNIT_US * 3); // inter-letter gap
                        }
                    }
                }
                other => {
                    pr_err!("-> Invalid command: {}\n", other);
                }
            }
        }

        let mut remaining = KVVec::new();
        remaining.extend_from_slice(&buf[input.len()..], GFP_KERNEL);

        buf.clear();
        buf.extend_from_slice(&remaining, GFP_KERNEL);

        Ok(len)
    }
}

#[pinned_drop]
impl PinnedDrop for RustMiscDevice {
    fn drop(self: Pin<&mut Self>) {
        pr_info!("Exiting the Rust Misc Device Sample\n");
    }
}

const GPIOD_FLAGS_BIT_DIR_SET: c_int =        1 << 0;
const GPIOD_FLAGS_BIT_DIR_OUT: c_int =        1 << 1;
const GPIOD_FLAGS_BIT_DIR_VAL: c_int =        1 << 2;
const GPIOD_FLAGS_BIT_OPEN_DRAIN: c_int =     1 << 3;

const GPIOD_ASIS: c_int      = 0;
const GPIOD_IN: c_int        = GPIOD_FLAGS_BIT_DIR_SET;
const GPIOD_OUT_LOW: c_int   = GPIOD_FLAGS_BIT_DIR_SET | GPIOD_FLAGS_BIT_DIR_OUT;
const GPIOD_OUT_HIGH: c_int  = GPIOD_FLAGS_BIT_DIR_SET | GPIOD_FLAGS_BIT_DIR_OUT | GPIOD_FLAGS_BIT_DIR_VAL;
const GPIOD_OUT_LOW_OPEN_DRAIN: c_int = GPIOD_OUT_LOW | GPIOD_FLAGS_BIT_OPEN_DRAIN;
const GPIOD_OUT_HIGH_OPEN_DRAIN: c_int = GPIOD_OUT_HIGH | GPIOD_FLAGS_BIT_OPEN_DRAIN;

const TASK_UNINTERRUPTIBLE: u32 = 2;

struct GpioOutputPin {
    desc: *mut gpio_desc,
}

unsafe impl Send for GpioOutputPin {}
unsafe impl Sync for GpioOutputPin {}

const GPIO_BASE_OFFSET: u32 = 512;

impl GpioOutputPin {
    fn new(gpio: u32) -> Result<Self> {
        let desc = unsafe { gpio_to_desc(GPIO_BASE_OFFSET + gpio) };
        if desc.is_null() {
            pr_info!("Failed to get gpio desc\n");
            return Err(EIO);
        }

        let ret = unsafe { gpiod_direction_output(desc, 0) };
        if ret < 0 {
            pr_info!("Failed to set gpio direction\n");
            return Err(EIO);
        }

        Ok(Self { desc })
    }

    fn set(&mut self, value: bool) -> Result<()> {
        let value = if value { 1 } else { 0 };
        let ret = unsafe { gpiod_set_value(self.desc, value) };

        if ret < 0 {
            return Err(EIO);
        }

        Ok(())
    }

    fn get(&mut self) -> bool {
        let value = unsafe { gpiod_get_value(self.desc) };
        value != 0
    }

    fn toggle(&mut self) -> Result<()> {
        let value = self.get();
        self.set(!value)?;

        Ok(())
    }

    fn play_tone(&mut self, freq: u64, duration_us: u64) -> Result<()> {
        if freq == 0 {
            sleep_us(duration_us);
            return Ok(());
        }

        let period_ns = div64(1_000_000_000, freq);
        let delay_ns = period_ns >> 1;
        let delay_us = div64(delay_ns, 1000);
        let cycles = div64(duration_us * freq, 1_000_000);

        for _ in 0..cycles {
            self.toggle()?;
            sleep_us(delay_us);
            self.toggle()?;
            sleep_us(delay_us);
        }

        Ok(())
    }
}

impl Drop for GpioOutputPin {
    fn drop(&mut self) {
        // NOTE: Causes refcount underflow as gpio_to_desc doesn't increment refcount
        // unsafe { gpiod_put(self.desc) };
    }
}

unsafe extern "C" {
    unsafe fn gpio_to_desc(gpio: u32) -> *mut gpio_desc;

    unsafe fn gpiod_direction_output(gpio: *mut gpio_desc, value: c_int) -> c_int;

    unsafe fn gpiod_get_value(gpio: *mut gpio_desc) -> c_int;

    unsafe fn gpiod_set_value(gpio: *mut gpio_desc, value: c_int) -> c_int;

    unsafe fn gpiod_put(gpio: *mut gpio_desc);
}


fn morse_encode(c: char) -> Option<&'static str> {
    match c.to_ascii_uppercase() {
        'A' => Some(".-"),
        'B' => Some("-..."),
        'C' => Some("-.-."),
        'D' => Some("-.."),
        'E' => Some("."),
        'F' => Some("..-."),
        'G' => Some("--."),
        'H' => Some("...."),
        'I' => Some(".."),
        'J' => Some(".---"),
        'K' => Some("-.-"),
        'L' => Some(".-.."),
        'M' => Some("--"),
        'N' => Some("-."),
        'O' => Some("---"),
        'P' => Some(".--."),
        'Q' => Some("--.-"),
        'R' => Some(".-."),
        'S' => Some("..."),
        'T' => Some("-"),
        'U' => Some("..-"),
        'V' => Some("...-"),
        'W' => Some(".--"),
        'X' => Some("-..-"),
        'Y' => Some("-.--"),
        'Z' => Some("--.."),
        '0' => Some("-----"),
        '1' => Some(".----"),
        '2' => Some("..---"),
        '3' => Some("...--"),
        '4' => Some("....-"),
        '5' => Some("....."),
        '6' => Some("-...."),
        '7' => Some("--..."),
        '8' => Some("---.."),
        '9' => Some("----."),
        _ => None,
    }
}

fn div64(a: u64, b: u64) -> u64 {
    unsafe { kernel::bindings::div64_u64(a, b) }
}

fn sleep_us(us: u64) {
    unsafe {
        kernel::bindings::usleep_range_state(
            us as usize,
            us as usize,
            TASK_UNINTERRUPTIBLE,
        )
    }
}
