//! Super bad, adhoc disassembling utils for debugging

/*
use core::{fmt, iter};
use core::fmt::Write;

use zydis::{AddressWidth, Decoder, Formatter, FormatterStyle, MachineMode, OutputBuffer};

static CODE: &'static [u8] = &[
	0x51, 0x8D, 0x45, 0xFF, 0x50, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 0x08,
	0xFF, 0x15, 0xA0, 0xA5, 0x48, 0x76, 0x85, 0xC0, 0x0F, 0x88, 0xFC,
	0xDA, 0x02, 0x00,
];

fn dis(code: &[u8]) -> Dis {
//	let binary_path = env::args().nth(1)
//		.expect("Required arg <elf-path> not given");
	
//	let binary_file_buf = fs::read(Path::new(&binary_path)).unwrap();
//	let binary_elf = match Elf::from_bytes(&binary_file_buf) {
//		Elf::Elf64(elf64) => elf64,
//		_ => panic!("elf32 is not supported"),
//	};
	
	let dis = Decoder::new(MachineMode::LONG_64, AddressWidth::_64)?;
	let instfmt = Formatter::new(FormatterStyle::INTEL)?;
	
	let mut fmt_buf_raw = [0u8; 256];
	let mut fmt_buf = OutputBuffer::new(&mut fmt_buf_raw);
	
	let mut cursor = 0usize;
	for (inst, ip) in dis.instruction_iterator(&CODE, 0x0) {
		instfmt.format_instruction(&inst, &mut fmt_buf, Some(0x0), None);
		println!("\x1b[0m0x{:016X} \x1b[34;1m{} \x1b[32;1m{}", ip, InstBytesFmt {width: 6, bytes: &CODE[cursor..cursor+inst.length as usize]}, fmt_buf);
		cursor += inst.length as usize;
	}
	
	Ok(())
}

pub struct Dis<'a> {
	code: &'a [u8]
}

impl<'a> fmt::Display for Dis<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let dis = Decoder::new(MachineMode::LONG_64, AddressWidth::_64)?;
		let instfmt = Formatter::new(FormatterStyle::INTEL)?;
		
		let mut fmt_buf_raw = [0u8; 256];
		let mut fmt_buf = OutputBuffer::new(&mut fmt_buf_raw);
		
		let mut cursor = 0usize;
		for (inst, ip) in dis.instruction_iterator(&CODE, 0x0) {
			instfmt.format_instruction(&inst, &mut fmt_buf, Some(0x0), None);
			
			writeln!(f, "\x1b[0m0x{:016X} \x1b[34;1m{} \x1b[32;1m{}", ip, InstBytesFmt {width: 6, bytes: &CODE[cursor..cursor+inst.length as usize]}, fmt_buf)?;
//			println!("\x1b[0m0x{:016X} \x1b[34;1m{} \x1b[32;1m{}", ip, InstBytesFmt {width: 6, bytes: &CODE[cursor..cursor+inst.length as usize]}, fmt_buf);
			cursor += inst.length as usize;
		}
		Ok(())
	}
}

#[derive(Debug)]
struct InstBytesFmt<'a> {
	pub bytes: &'a [u8],
	pub width: usize,
}

impl<'a> fmt::Display for InstBytesFmt<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let byte_iter = (0..self.width)
			.map(|i| (i, self.bytes.get(i).copied()));
		
		for (i, ob) in byte_iter {
			if i > 0 {
				f.write_char(' ')?;
			}
			
			if let Some(b) = ob {
				if (i >= self.width-1) && (self.bytes.len() > self.width) {
					f.write_str("..")?;
				} else {
					write!(f, "{:02X}", b)?;
				}
			} else {
				f.write_str("  ")?;
			}
		}
		
//		for (i, b) in self.bytes.iter().copied().take(self.width).enumerate() {
//			if i > 0 {
//				f.write_char(' ')?;
//			}
//			
//			if (i >= self.width-1) && (self.bytes.len() > self.width) {
//				f.write_str("..")?;
//			} else {
//				write!(f, "{:02X}", b)?;
//			}
//		}
		Ok(())
	}
}
*/
