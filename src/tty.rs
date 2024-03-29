use core::arch::asm;
use crate::arch::x86_64::port::*;

// https://wiki.osdev.org/Serial_Ports#Programming_the_Serial_Communications_Port
// https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming

/// COM 1
const PORT: u16 = 0x3F8;

pub unsafe fn enable_serial_tty() {
	outb(PORT + 1, 0x00); // Disable all interrupts
	outb(PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
	outb(PORT + 0, 0x03); // Set divisor to 3 (lo byte): 115200 / 3 = 38400 baud
	outb(PORT + 1, 0x00); //                  (hi byte)
	outb(PORT + 3, 0b00_000_0_11); // Disable DLAB, 8 bits, no parity, one stop bit
	outb(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
	outb(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
	outb(PORT + 4, 0x1E); // Set in loopback mode, test the serial chip
	
	outb(PORT + 0, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)
	let read_byte = inb(PORT + 0);
	if read_byte != 0xAE {
		panic!("Serial chip quick test failed: got byte {}, expected {}", read_byte, 0xAE);
	}
	
	outb(PORT + 1, 0x01); // Reenable interrupts
	outb(PORT + 4, 0x0F); // Enter normal operating mode
}

pub unsafe fn write_tty_char(ch: u8) {
	// Wait for tx port empty
	while inb(PORT + 5) & 0x20 == 0 {
		asm!("pause");
	}
	
	// Send char
	outb(PORT, ch);
}

pub unsafe fn write_tty(msg: &[u8]) {
	for &ch in msg {
		// TODO: This is an ugly hack for now
		if ch == b'\n' {
			write_tty_char(b'\r');
		}
		write_tty_char(ch);
	}
}

pub unsafe fn write_tty_nl_only() {
	write_tty(b"\r\n");
}

pub unsafe fn write_tty_ln(msg: &[u8]) {
	write_tty(msg);
	write_tty_nl_only();
}

#[inline(always)]
pub fn tty_writer() -> TtyWriter {
	TtyWriter
}

pub struct TtyWriter;
impl core::fmt::Write for TtyWriter {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		unsafe {
			write_tty(s.as_bytes());
		}
		Ok(())
	}
}

pub unsafe fn read_tty_char() -> Option<u8> {
	let status = inb(PORT + 5);
	
	// Always read the Receive Buffer Register
	// to clear the interrupt status even
	// if there's no "valid" data in the buffer
	let c = inb(PORT);
	
	if status & 0x1 == 1 {
		Some(c)
	} else {
		None
	}
}
