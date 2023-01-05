
SECTIONS {
	.shstrtab : { *(.shstrtab) }
	.text : { *(.text.start0) *(.text) *(.text.*) }
	.rodata : { *(.rodata) *(.rodata.*) }
	.bss : { *(.bss) *(.bss.*) }
	.data.rel.ro : { *(.data.rel.ro) *(.data.rel.ro.*) }
}
