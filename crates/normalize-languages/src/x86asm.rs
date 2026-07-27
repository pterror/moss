//! x86 assembly support.

use crate::{Language, LanguageSymbols};

/// x86 Assembly language support.
pub struct X86Asm;

impl Language for X86Asm {
    fn name(&self) -> &'static str {
        "x86 Assembly"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["asm", "s", "S"]
    }
    fn grammar_name(&self) -> &'static str {
        "x86asm"
    }

    fn sniff_hints(&self) -> crate::SniffHints {
        // NASM/MASM use Intel syntax: unprefixed registers, `[bits N]`,
        // `section .text` without GAS-only directives.
        crate::SniffHints {
            shebang_patterns: &[],
            content_signals: &[
                ("[bits ", 3),
                ("[BITS ", 3),
                ("global _start", 2),
                ("section .text", 1),
                (".globl ", -3),
                (".type ", -2),
                ("%rip", -3),
                ("%rax", -2),
                ("%eax", -2),
            ],
        }
    }

    fn as_symbols(&self) -> Option<&dyn LanguageSymbols> {
        Some(self)
    }
}

impl LanguageSymbols for X86Asm {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_unused_kinds_audit;

    #[test]
    fn unused_node_kinds_audit() {
        #[rustfmt::skip]
        let documented_unused: &[&str] = &[
            "label_definition", "instruction", "identifier",
            "memory_expression", "binary_expression",
        ];
        validate_unused_kinds_audit(&X86Asm, documented_unused)
            .expect("x86 Assembly unused node kinds audit failed");
    }
}
