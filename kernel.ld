
SECTIONS {
	.shstrtab : { *(.shstrtab) }
	.text : { *(.text._start) *(.text) *(.text.*) }
	.rodata : { *(.rodata) *(.rodata.*) }
	.bss : { *(.bss) *(.bss.*) }
	.data.rel.ro : { *(.data.rel.ro) *(.data.rel.ro.*) }
}
