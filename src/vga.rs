pub use crate::arch::x86_64::port::*;

unsafe fn pokeb(s: usize, o: usize, val: u8) {
	// #define pokeb(S,O,V)    *(unsigned char*)(16uL * (S) + (O)) = (V)
	*((16*s + o) as *mut u8) = val;
}

unsafe fn pokew(s: usize, o: usize, val: u16) {
	// #define pokew(S,O,V)    *(unsigned short*)(16uL * (S) + (O)) = (V)
	*((16*s + o) as *mut u16) = val;
}

const VGA_AC_INDEX: u16 = 0x3C0;
const VGA_AC_WRITE: u16 = 0x3C0;
const VGA_AC_READ: u16 = 0x3C1;
const VGA_MISC_WRITE: u16 = 0x3C2;
const VGA_SEQ_INDEX: u16 = 0x3C4;
const VGA_SEQ_DATA: u16 = 0x3C5;
const VGA_DAC_READ_INDEX: u16 = 0x3C7;
const VGA_DAC_WRITE_INDEX: u16 = 0x3C8;
const VGA_DAC_DATA: u16 = 0x3C9;
const VGA_MISC_READ: u16 = 0x3CC;
const VGA_GC_INDEX: u16 = 0x3CE;
const VGA_GC_DATA: u16 = 0x3CF;
const VGA_CRTC_INDEX: u16 = 0x3D4;
const VGA_CRTC_DATA: u16 = 0x3D5;
const VGA_INSTAT_READ: u16 = 0x3DA;

const VGA_NUM_SEQ_REGS: u16 = 5;
const VGA_NUM_CRTC_REGS: u16 = 25;
const VGA_NUM_GC_REGS: u16 = 9;
const VGA_NUM_AC_REGS: u16 = 21;
#[allow(unused_parens)]
const VGA_NUM_REGS: u16 = (1 + VGA_NUM_SEQ_REGS + VGA_NUM_CRTC_REGS
	+ VGA_NUM_GC_REGS + VGA_NUM_AC_REGS);

#[allow(non_upper_case_globals)]
static g_80x25_text: [u8; 61] = [
	/* MISC */
	0x67,
	/* SEQ */
	0x03, 0x00, 0x03, 0x00, 0x02,
	/* CRTC */
	0x5F, 0x4F, 0x50, 0x82, 0x55, 0x81, 0xBF, 0x1F,
	0x00, 0x4F, 0x0D, 0x0E, 0x00, 0x00, 0x00, 0x50,
	0x9C, 0x0E, 0x8F, 0x28, 0x1F, 0x96, 0xB9, 0xA3,
	0xFF,
	/* GC */
	0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x0E, 0x00,
	0xFF,
	/* AC */
	0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x14, 0x07,
	0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
	0x0C, 0x00, 0x0F, 0x08, 0x00
];

#[allow(non_upper_case_globals)]
static g_90x60_text: [u8; 61] = [
	/* MISC */
	0xE7,
	/* SEQ */
	0x03, 0x01, 0x03, 0x00, 0x02,
	/* CRTC */
	0x6B, 0x59, 0x5A, 0x82, 0x60, 0x8D, 0x0B, 0x3E,
	0x00, 0x47, 0x06, 0x07, 0x00, 0x00, 0x00, 0x00,
	0xEA, 0x0C, 0xDF, 0x2D, 0x08, 0xE8, 0x05, 0xA3,
	0xFF,
	/* GC */
	0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x0E, 0x00,
	0xFF,
	/* AC */
	0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x14, 0x07,
	0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
	0x0C, 0x00, 0x0F, 0x08, 0x00,
];

pub unsafe fn set_text_mode(high_res: bool) {
	// https://files.osdev.org/mirrors/geezer/osd/graphics/modes.c
	let rows;
	let cols;
	let ht;
	
	if high_res {
		write_regs(g_90x60_text.as_ptr());
		cols = 90;
		rows = 60;
		ht = 8;
	}
	else {
		write_regs(g_80x25_text.as_ptr());
		cols = 80;
		rows = 25;
		ht = 16;
	}
	
//	// Set font
//	if ht >= 16 {
//		write_font(g_8x16_font, 16);
//	} else {
//		write_font(g_8x8_font, 8);
//	}
	
	// Tell the BIOS what we've done, so BIOS text output works OK
	pokew(0x40, 0x4A, cols);	/* columns on screen */
	pokew(0x40, 0x4C, cols * rows * 2); /* framebuffer size */
	pokew(0x40, 0x50, 0);		/* cursor pos'n */
	pokeb(0x40, 0x60, ht - 1);	/* cursor shape */
	pokeb(0x40, 0x61, ht - 2);
	pokeb(0x40, 0x84, rows as u8 - 1);	/* rows on screen - 1 */
	pokeb(0x40, 0x85, ht);		/* char height */
	
	// set white-on-black attributes for all text
	for i in 0..cols*rows {
//		pokeb(0xB800, i as usize * 2 + 1, 7);
//		pokeb(0xB8000, i as usize * 2 + 1, 7);
	}
}

pub unsafe fn write_regs(mut regs: *const u8) {
	// Write MISCELLANEOUS reg
	outb(VGA_MISC_WRITE, *regs);
	regs = (regs as usize + 1) as *mut u8;
	
	// Write SEQUENCER regs
	for i in 0..VGA_NUM_SEQ_REGS {
		outb(VGA_SEQ_INDEX, i as u8);
		outb(VGA_SEQ_DATA, *regs);
		regs = (regs as usize + 1) as *mut u8;
	}
	
	// Unlock CRTC registers
	outb(VGA_CRTC_INDEX, 0x03);
	outb(VGA_CRTC_DATA, inb(VGA_CRTC_DATA) | 0x80);
	outb(VGA_CRTC_INDEX, 0x11);
	outb(VGA_CRTC_DATA, inb(VGA_CRTC_DATA) & !0x80);
	
	// Make sure they remain unlocked
//	*regs.offset(0x03) |= 0x80;
//	*regs.offset(0x11) &= !0x80;
	
	// Write CRTC regs
	for i in 0..VGA_NUM_CRTC_REGS {
		outb(VGA_CRTC_INDEX, i as u8);
		outb(VGA_CRTC_DATA, *regs);
		regs = (regs as usize + 1) as *mut u8;
	}
	
	// Write GRAPHICS CONTROLLER regs
	for i in 0..VGA_NUM_GC_REGS {
		outb(VGA_GC_INDEX, i as u8);
		outb(VGA_GC_DATA, *regs);
		regs = (regs as usize + 1) as *mut u8;
	}
	
	// Write ATTRIBUTE CONTROLLER regs
	for i in 0..VGA_NUM_AC_REGS {
		inb(VGA_INSTAT_READ);
		outb(VGA_AC_INDEX, i as u8);
		outb(VGA_AC_WRITE, *regs);
		regs = (regs as usize + 1) as *mut u8;
	}
	
	// Lock 16-color palette and unblank display
	inb(VGA_INSTAT_READ);
	outb(VGA_AC_INDEX, 0x20);
}
