//! sub/superscript stuff
use {once_cell::sync::Lazy, std::collections::HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// script mode (super/subscript)
pub enum ScriptMode {
    /// superscript
    Superscript,
    /// subscript
    Subscript,
}

/// mappings for converting ascii to superscript
static SUPERSCRIPT_MAP: Lazy<HashMap<char, char>> = Lazy::new(|| {
    [
        ('0', '⁰'),
        ('1', '¹'),
        ('2', '²'),
        ('3', '³'),
        ('4', '⁴'),
        ('5', '⁵'),
        ('6', '⁶'),
        ('7', '⁷'),
        ('8', '⁸'),
        ('9', '⁹'),
        ('+', '⁺'),
        ('-', '⁻'),
        ('=', '⁼'),
        ('(', '⁽'),
        (')', '⁾'),
        ('a', 'ᵃ'),
        ('b', 'ᵇ'),
        ('c', 'ᶜ'),
        ('d', 'ᵈ'),
        ('e', 'ᵉ'),
        ('f', 'ᶠ'),
        ('g', 'ᵍ'),
        ('h', 'ʰ'),
        ('i', 'ⁱ'),
        ('j', 'ʲ'),
        ('k', 'ᵏ'),
        ('l', 'ˡ'),
        ('m', 'ᵐ'),
        ('n', 'ⁿ'),
        ('o', 'ᵒ'),
        ('p', 'ᵖ'),
        ('r', 'ʳ'),
        ('s', 'ˢ'),
        ('t', 'ᵗ'),
        ('u', 'ᵘ'),
        ('v', 'ᵛ'),
        ('w', 'ʷ'),
        ('x', 'ˣ'),
        ('y', 'ʸ'),
        ('z', 'ᶻ'),
        ('A', 'ᴬ'),
        ('B', 'ᴮ'),
        ('D', 'ᴰ'),
        ('E', 'ᴱ'),
        ('G', 'ᴳ'),
        ('H', 'ᴴ'),
        ('I', 'ᴵ'),
        ('J', 'ᴶ'),
        ('K', 'ᴷ'),
        ('L', 'ᴸ'),
        ('M', 'ᴹ'),
        ('N', 'ᴺ'),
        ('O', 'ᴼ'),
        ('P', 'ᴾ'),
        ('R', 'ᴿ'),
        ('T', 'ᵀ'),
        ('U', 'ᵁ'),
        ('V', 'ⱽ'),
        ('W', 'ᵂ'),
    ]
    .into_iter()
    .collect()
});

/// mappings for converting ascii to subscript
static SUBSCRIPT_MAP: Lazy<HashMap<char, char>> = Lazy::new(|| {
    [
        ('0', '₀'),
        ('1', '₁'),
        ('2', '₂'),
        ('3', '₃'),
        ('4', '₄'),
        ('5', '₅'),
        ('6', '₆'),
        ('7', '₇'),
        ('8', '₈'),
        ('9', '₉'),
        ('+', '₊'),
        ('-', '₋'),
        ('=', '₌'),
        ('(', '₍'),
        (')', '₎'),
        ('a', 'ₐ'),
        ('e', 'ₑ'),
        ('h', 'ₕ'),
        ('i', 'ᵢ'),
        ('j', 'ⱼ'),
        ('k', 'ₖ'),
        ('l', 'ₗ'),
        ('m', 'ₘ'),
        ('n', 'ₙ'),
        ('o', 'ₒ'),
        ('p', 'ₚ'),
        ('r', 'ᵣ'),
        ('s', 'ₛ'),
        ('t', 'ₜ'),
        ('u', 'ᵤ'),
        ('v', 'ᵥ'),
        ('x', 'ₓ'),
    ]
    .into_iter()
    .collect()
});

/// convert between super/subscript
pub fn convert_script(text: &str, mode: ScriptMode) -> String {
    let map = match mode {
        ScriptMode::Superscript => &SUPERSCRIPT_MAP,
        ScriptMode::Subscript => &SUBSCRIPT_MAP,
    };

    text.chars()
        .map(|c| map.get(&c).copied().unwrap_or(c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superscript_conversion() {
        assert_eq!(convert_script("x2", ScriptMode::Superscript), "ˣ²");
        assert_eq!(convert_script("a+b", ScriptMode::Superscript), "ᵃ⁺ᵇ");
    }

    #[test]
    fn test_subscript_conversion() {
        assert_eq!(convert_script("H2O", ScriptMode::Subscript), "H₂O");
        assert_eq!(convert_script("x0", ScriptMode::Subscript), "ₓ₀");
    }

    #[test]
    fn test_unmapped_chars() {
        assert_eq!(convert_script("xyz", ScriptMode::Subscript), "ₓyz");
    }
}
