pub fn render_math_in_text(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            let is_block = chars.peek() == Some(&'$');
            if is_block {
                chars.next();
            }

            let mut formula = String::new();
            let mut closed = false;

            while let Some(c) = chars.next() {
                if c == '$' {
                    if is_block {
                        if chars.peek() == Some(&'$') {
                            chars.next();
                            closed = true;
                            break;
                        } else {
                            formula.push(c);
                        }
                    } else {
                        closed = true;
                        break;
                    }
                } else {
                    formula.push(c);
                }
            }

            if closed {
                let converted = convert_latex_symbols(&formula);
                if is_block {
                    out.push_str(&format!("\n   {}\n", converted.trim()));
                } else {
                    out.push_str(&converted);
                }
            } else {
                if is_block {
                    out.push_str("$$");
                } else {
                    out.push('$');
                }
                out.push_str(&formula);
            }
        } else {
            out.push(ch);
        }
    }

    out
}

pub fn convert_latex_symbols(latex: &str) -> String {
    let mut s = latex.to_string();

    // Punkte VOR den Akzenten auflösen
    s = s.replace(r"\dots", "…");
    s = s.replace(r"\cdots", "⋯");
    s = s.replace(r"\ddots", "⋱");
    s = s.replace(r"\vdots", "⋮");

    // Danach erst Akzente und andere Befehle...
    s = replace_accents(&s);
    // ...

    // 1. Spezifische Umgebungen & Brüche
    s = replace_matrix_environments(&s);
    s = replace_binom(&s);
    s = replace_frac_cmd(&s, r"\tfrac");
    s = replace_frac_cmd(&s, r"\dfrac");
    s = replace_frac_cmd(&s, r"\frac");

    // 2. Unäre Befehle & Akzente
    s = replace_accents(&s);
    s = replace_unary_cmd(&s, r"\sqrt", "√");
    s = replace_unary_cmd(&s, r"\textbf", "");
    s = replace_unary_cmd(&s, r"\mathrm", "");
    s = replace_unary_cmd(&s, r"\mathbf", "");
    s = replace_unary_cmd(&s, r"\mathit", "");
    s = replace_unary_cmd(&s, r"\text", "");

    // 3. Delimiter & Abstände bereinigen
    s = s.replace(r"\left", "");
    s = s.replace(r"\right", "");
    s = s.replace(r"\,", " ");
    s = s.replace(r"\;", " ");
    s = s.replace(r"\!", "");
    s = s.replace(r"\quad", "   ");
    s = s.replace(r"\qquad", "      ");
    s = s.replace(r"\{", "{");
    s = s.replace(r"\}", "}");

    // 4. Modulo & Verwandtes VOR \pm matchen
    s = replace_unary_cmd(&s, r"\pmod", " mod ");
    s = s.replace(r"\bmod", " mod ");

    // 5. Kryptographie-Operatoren
    s = s.replace(r"\oplus", "⊕");
    s = s.replace(r"\otimes", "⊗");
    s = s.replace(r"\odot", "⊙");

    // 6. Blackboard & Zahlenmengen
    s = s.replace(r"\mathbb{R}", "ℝ");
    s = s.replace(r"\mathbb{N}", "ℕ");
    s = s.replace(r"\mathbb{Z}", "ℤ");
    s = s.replace(r"\mathbb{Q}", "ℚ");
    s = s.replace(r"\mathbb{C}", "ℂ");
    s = s.replace(r"\mathbb{P}", "ℙ");

    // 7. Große Operatoren & Analysis
    s = s.replace(r"\sum", "∑");
    s = s.replace(r"\prod", "∏");
    s = s.replace(r"\coprod", "∐");
    s = s.replace(r"\int", "∫");
    s = s.replace(r"\iint", "∬");
    s = s.replace(r"\iiint", "∭");
    s = s.replace(r"\oint", "∮");
    s = s.replace(r"\partial", "∂");
    s = s.replace(r"\nabla", "∇");
    s = s.replace(r"\infty", "∞");

    // 8. Standard-Mathematikfunktionen
    for func in &[
        "sin", "cos", "tan", "cot", "sec", "csc", "arcsin", "arccos", "arctan", "sinh", "cosh",
        "tanh", "coth", "ln", "log", "exp", "lim", "sup", "inf", "max", "min", "det", "deg", "gcd",
        "Pr",
    ] {
        s = s.replace(&format!(r"\{func}"), func);
    }

    // 9. Logik & Pfeile
    s = s.replace(r"\iff", "⟺");
    s = s.replace(r"\implies", "⟹");
    s = s.replace(r"\impliedby", "⟸");
    s = s.replace(r"\longleftrightarrow", "⟷");
    s = s.replace(r"\leftrightarrow", "↔");
    s = s.replace(r"\rightarrow", "→");
    s = s.replace(r"\to", "→");
    s = s.replace(r"\leftarrow", "←");
    s = s.replace(r"\mapsto", "↦");
    s = s.replace(r"\uparrow", "↑");
    s = s.replace(r"\downarrow", "↓");
    s = s.replace(r"\forall", "∀");
    s = s.replace(r"\exists", "∃");
    s = s.replace(r"\nexists", "∄");
    s = s.replace(r"\land", "∧");
    s = s.replace(r"\lor", "∨");
    s = s.replace(r"\neg", "¬");
    s = s.replace(r"\top", "⊤");
    s = s.replace(r"\bot", "⊥");
    s = s.replace(r"\vdash", "⊢");
    s = s.replace(r"\vDash", "⊨");
    s = s.replace(r"\qed", "∎");

    // 10. Mengenlehre & Relationen
    s = s.replace(r"\notin", "∉");
    s = s.replace(r"\in", "∈");
    s = s.replace(r"\subseteq", "⊆");
    s = s.replace(r"\supseteq", "⊇");
    s = s.replace(r"\subsetneq", "⊊");
    s = s.replace(r"\supset", "⊃");
    s = s.replace(r"\subset", "⊂");
    s = s.replace(r"\cup", "∪");
    s = s.replace(r"\cap", "∩");
    s = s.replace(r"\setminus", "∖");
    s = s.replace(r"\emptyset", "∅");
    s = s.replace(r"\empty", "∅");

    // 11. Arithmetik & Vergleich
    s = s.replace(r"\pm", "±");
    s = s.replace(r"\mp", "∓");
    s = s.replace(r"\cdot", "·");
    s = s.replace(r"\times", "×");
    s = s.replace(r"\div", "÷");
    s = s.replace(r"\ast", "∗");
    s = s.replace(r"\star", "⋆");
    s = s.replace(r"\circ", "∘");
    s = s.replace(r"\bullet", "•");
    s = s.replace(r"\leq", "≤");
    s = s.replace(r"\le", "≤");
    s = s.replace(r"\geq", "≥");
    s = s.replace(r"\ge", "≥");
    s = s.replace(r"\neq", "≠");
    s = s.replace(r"\ne", "≠");
    s = s.replace(r"\approx", "≈");
    s = s.replace(r"\equiv", "≡");
    s = s.replace(r"\sim", "∼");
    s = s.replace(r"\simeq", "≃");
    s = s.replace(r"\cong", "≅");
    s = s.replace(r"\propto", "∝");
    s = s.replace(r"\parallel", "∥");
    s = s.replace(r"\perp", "⟂");
    s = s.replace(r"\dots", "…");
    s = s.replace(r"\cdots", "⋯");
    s = s.replace(r"\ddots", "⋱");
    s = s.replace(r"\vdots", "⋮");

    // 12. Griechische Buchstaben
    s = s.replace(r"\alpha", "α");
    s = s.replace(r"\beta", "β");
    s = s.replace(r"\gamma", "γ");
    s = s.replace(r"\Gamma", "Γ");
    s = s.replace(r"\delta", "δ");
    s = s.replace(r"\Delta", "Δ");
    s = s.replace(r"\epsilon", "ε");
    s = s.replace(r"\varepsilon", "ε");
    s = s.replace(r"\zeta", "ζ");
    s = s.replace(r"\eta", "η");
    s = s.replace(r"\theta", "θ");
    s = s.replace(r"\vartheta", "ϑ");
    s = s.replace(r"\Theta", "Θ");
    s = s.replace(r"\iota", "ι");
    s = s.replace(r"\kappa", "κ");
    s = s.replace(r"\lambda", "λ");
    s = s.replace(r"\Lambda", "Λ");
    s = s.replace(r"\mu", "μ");
    s = s.replace(r"\nu", "ν");
    s = s.replace(r"\xi", "ξ");
    s = s.replace(r"\Xi", "Ξ");
    s = s.replace(r"\pi", "π");
    s = s.replace(r"\varpi", "ϖ");
    s = s.replace(r"\Pi", "Π");
    s = s.replace(r"\rho", "ρ");
    s = s.replace(r"\varrho", "ϱ");
    s = s.replace(r"\sigma", "σ");
    s = s.replace(r"\varsigma", "ς");
    s = s.replace(r"\Sigma", "Σ");
    s = s.replace(r"\tau", "τ");
    s = s.replace(r"\upsilon", "υ");
    s = s.replace(r"\Upsilon", "Υ");
    s = s.replace(r"\phi", "φ");
    s = s.replace(r"\varphi", "ϕ");
    s = s.replace(r"\Phi", "Φ");
    s = s.replace(r"\chi", "χ");
    s = s.replace(r"\psi", "ψ");
    s = s.replace(r"\Psi", "Ψ");
    s = s.replace(r"\omega", "ω");
    s = s.replace(r"\Omega", "Ω");

    convert_sub_and_superscripts(&s)
}

pub fn replace_accents(input: &str) -> String {
    let mut res = input.to_string();
    let accents = [
        (r"\overline", '\u{0305}'), // Längere Befehle zuerst prüfen
        (r"\ddot", '\u{0308}'),
        (r"\check", '\u{030C}'),
        (r"\tilde", '\u{0303}'),
        (r"\vec", '\u{20D7}'),
        (r"\hat", '\u{0302}'),
        (r"\bar", '\u{0304}'),
        (r"\dot", '\u{0307}'),
    ];

    for (cmd, mark) in accents {
        while let Some(start) = res.find(cmd) {
            let after = &res[start + cmd.len()..];
            let trimmed_after = after.trim_start();
            let lead_spaces = after.len() - trimmed_after.len();

            if after.starts_with(|c: char| c.is_ascii_alphabetic()) {
                break;
            }

            if trimmed_after.starts_with('{') {
                if let Some(close) = find_matching_brace(trimmed_after) {
                    let inner = &trimmed_after[1..close];
                    let total_len = cmd.len() + lead_spaces + close + 1;
                    let replacement: String = inner.chars().flat_map(|ch| [ch, mark]).collect();
                    res.replace_range(start..start + total_len, &replacement);
                    continue;
                }
            } else if let Some(first_ch) = trimmed_after.chars().next() {
                let char_bytes = first_ch.len_utf8();
                let total_len = cmd.len() + lead_spaces + char_bytes;
                let replacement = format!("{}{}", first_ch, mark);
                res.replace_range(start..start + total_len, &replacement);
                continue;
            }
            break;
        }
    }
    res
}
pub fn replace_unary_cmd(input: &str, cmd: &str, prefix: &str) -> String {
    let mut res = input.to_string();
    while let Some(start) = res.find(cmd) {
        let after = &res[start + cmd.len()..];
        if let Some(open) = after.find('{') {
            if after[..open].trim().is_empty() {
                if let Some(close) = find_matching_brace(&after[open..]) {
                    let inner = &after[open + 1..open + close];
                    let total_len = cmd.len() + open + close + 1;
                    let replacement = format!("{}{}", prefix, inner);
                    res.replace_range(start..start + total_len, &replacement);
                    continue;
                }
            }
        }
        break;
    }
    res
}

pub fn replace_frac_cmd(input: &str, cmd: &str) -> String {
    let mut res = input.to_string();
    let cmd_len = cmd.len();

    while let Some(start) = res.find(cmd) {
        let after = &res[start + cmd_len..];
        if let Some(open1) = after.find('{') {
            if after[..open1].trim().is_empty() {
                if let Some(close1) = find_matching_brace(&after[open1..]) {
                    let num = &after[open1 + 1..open1 + close1];
                    let rem = &after[open1 + close1 + 1..];
                    if let Some(open2) = rem.find('{') {
                        if rem[..open2].trim().is_empty() {
                            if let Some(close2) = find_matching_brace(&rem[open2..]) {
                                let den = &rem[open2 + 1..open2 + close2];
                                let total_len = cmd_len + open1 + close1 + 1 + open2 + close2 + 1;
                                let replacement = format!("({} / {})", num.trim(), den.trim());
                                res.replace_range(start..start + total_len, &replacement);
                                continue;
                            }
                        }
                    }
                }
            }
        }
        break;
    }
    res
}

pub fn replace_binom(input: &str) -> String {
    let mut res = input.to_string();
    while let Some(start) = res.find(r"\binom") {
        let after = &res[start + 6..];
        if let Some(open1) = after.find('{') {
            if after[..open1].trim().is_empty() {
                if let Some(close1) = find_matching_brace(&after[open1..]) {
                    let n = &after[open1 + 1..open1 + close1];
                    let rem = &after[open1 + close1 + 1..];
                    if let Some(open2) = rem.find('{') {
                        if rem[..open2].trim().is_empty() {
                            if let Some(close2) = find_matching_brace(&rem[open2..]) {
                                let k = &rem[open2 + 1..open2 + close2];
                                let total_len = 6 + open1 + close1 + 1 + open2 + close2 + 1;
                                let replacement = format!("({} über {})", n.trim(), k.trim());
                                res.replace_range(start..start + total_len, &replacement);
                                continue;
                            }
                        }
                    }
                }
            }
        }
        break;
    }
    res
}

pub fn replace_matrix_environments(input: &str) -> String {
    let mut res = input.to_string();
    let envs = [
        ("matrix", "[", "]"),
        ("pmatrix", "(", ")"),
        ("bmatrix", "[", "]"),
        ("vmatrix", "|", "|"),
        ("Bmatrix", "{", "}"),
    ];

    for (env, left, right) in envs {
        let begin_tag = format!(r"\begin{{{}}}", env);
        let end_tag = format!(r"\end{{{}}}", env);

        while let Some(start) = res.find(&begin_tag) {
            if let Some(end) = res.find(&end_tag) {
                if end > start {
                    let content = &res[start + begin_tag.len()..end];
                    let rows: Vec<String> = content
                        .split(r"\\")
                        .map(|row| {
                            let cols: Vec<&str> = row.split('&').map(|c| c.trim()).collect();
                            cols.join(", ")
                        })
                        .filter(|r| !r.is_empty())
                        .collect();

                    let replacement = format!("{} {} {}", left, rows.join(" ; "), right);
                    res.replace_range(start..end + end_tag.len(), &replacement);
                    continue;
                }
            }
            break;
        }
    }
    res
}

pub fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (idx, ch) in s.char_indices() {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

pub fn map_script_char(c: char, is_super: bool) -> char {
    if is_super {
        match c {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            '+' => '⁺',
            '-' => '⁻',
            '=' => '⁼',
            '(' => '⁽',
            ')' => '⁾',
            'a' => 'ᵃ',
            'b' => 'ᵇ',
            'c' => 'ᶜ',
            'd' => 'ᵈ',
            'e' => 'ᵉ',
            'f' => 'ᶠ',
            'g' => 'ᵍ',
            'h' => 'ʰ',
            'i' => 'ⁱ',
            'j' => 'ʲ',
            'k' => 'ᵏ',
            'l' => 'ˡ',
            'm' => 'ᵐ',
            'n' => 'ⁿ',
            'o' => 'ᵒ',
            'p' => 'ᵖ',
            'r' => 'ʳ',
            's' => 'ˢ',
            't' => 'ᵗ',
            'u' => 'ᵘ',
            'v' => 'ᵛ',
            'w' => 'ʷ',
            'x' => 'ˣ',
            'y' => 'ʸ',
            'z' => 'ᶻ',
            'T' => 'ᵀ',
            _ => c,
        }
    } else {
        match c {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            '+' => '₊',
            '-' => '₋',
            '=' => '₌',
            '(' => '₍',
            ')' => '₎',
            'a' => 'ₐ',
            'e' => 'ₑ',
            'o' => 'ₒ',
            'x' => 'ₓ',
            'i' => 'ᵢ',
            'j' => 'ⱼ',
            'k' => 'ₖ',
            'l' => 'ₗ',
            'm' => 'ₘ',
            'n' => 'ₙ',
            'p' => 'ₚ',
            's' => 'ₛ',
            't' => 'ₜ',
            _ => c,
        }
    }
}

pub fn convert_sub_and_superscripts(input: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c == '^' || c == '_' {
            let is_super = c == '^';
            i += 1;
            if i >= chars.len() {
                out.push(c);
                break;
            }

            if chars[i] == '{' {
                i += 1;
                while i < chars.len() && chars[i] != '}' {
                    out.push(map_script_char(chars[i], is_super));
                    i += 1;
                }
                if i < chars.len() && chars[i] == '}' {
                    i += 1;
                }
            } else {
                out.push(map_script_char(chars[i], is_super));
                i += 1;
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}
