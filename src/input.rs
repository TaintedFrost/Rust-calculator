pub fn handle_input<const N: usize>(
    k: char,
    expr: &mut heapless::String<N>,
    fresh: &mut bool,
    last: &mut Option<f64>,
) {
    use core::fmt::Write;
    fn map_ad(c: char) -> char {
        match c {
            'A' => '+',
            'B' => '-',
            'C' => '*',
            'D' => '/',
            other => other,
        }
    }

    match k {
        'b' => {
            if *fresh {
                expr.clear();
                *fresh = false;
            } else {
                expr.pop();
            }
        }
        '(' => {
            if *fresh { expr.clear(); *fresh = false; }
            if let Some(last_ch) = expr.chars().last() {
                if last_ch.is_digit(10) || last_ch == ')' || last_ch == '.' || last_ch == '!' {
                    expr.push('*').ok();
                }
            }
            expr.push('(').ok();
        }
        ')' => {
            if *fresh { expr.clear(); *fresh = false; }
            expr.push(')').ok();
        }
        '.' => {
            if *fresh {
                expr.clear();
                *fresh = false;
                expr.push('0').ok();
                expr.push('.').ok();
            } else {
                let mut found_dot = false;
                for ch in expr.chars().rev() {
                    if ch == '.' { found_dot = true; break; }
                    if !ch.is_digit(10) { break; }
                }
                if !found_dot && expr.len() < expr.capacity() {
                    if expr.chars().last().map(|c| !c.is_digit(10)).unwrap_or(true) {
                        expr.push('0').ok();
                    }
                    expr.push('.').ok();
                }
            }
        }

        '0'..='9' => {
            if *fresh { expr.clear(); *fresh = false; }
            if let Some(last_ch) = expr.chars().last() {
                if last_ch == ')' {
                    expr.push('*').ok();
                }
            }
            if let Some(last_ch) = expr.chars().last() {
                if last_ch == '0' {
                    let mut token_len = 0usize;
                    let mut has_dot = false;
                    for ch in expr.chars().rev() {
                        if ch.is_digit(10) {
                            token_len += 1;
                        } else if ch == '.' {
                            has_dot = true;
                            token_len += 1;
                        } else {
                            break;
                        }
                    }
                    if token_len == 1 && !has_dot {
                        expr.pop();
                        expr.push(k).ok();
                        return;
                    }
                }
            }
            expr.push(k).ok();
        }

        'a' => {
            if *fresh { expr.clear(); *fresh = false; }
            if let Some(val) = *last {
                if let Some(last_ch) = expr.chars().last() {
                    if last_ch.is_digit(10) || last_ch == ')' || last_ch == '.' || last_ch == '!' {
                        expr.push('*').ok();
                    }
                }
                let s = crate::eval::format_f64(val);
                let mut remaining = expr.capacity().saturating_sub(expr.len());
                for ch in s.as_str().chars() {
                    if remaining == 0 { break; }
                    expr.push(ch).ok();
                    remaining -= 1;
                }
            } else {
                // no saved answer so do nothing 
            }
        }
        'r' | 's' | 'c' | 't' | 'k' | 'l' => {
            if *fresh { *fresh = false; }
            if let Some(last_ch) = expr.chars().last() {
                if last_ch.is_digit(10) || last_ch == ')' || last_ch == '.' || last_ch == '!' {
                    expr.push('*').ok();
                }
            }
            expr.push(k).ok();
        }
        '!' => {
            if !expr.is_empty() {
                if let Some(last_ch) = expr.chars().last() {
                    if last_ch.is_digit(10) || last_ch == ')' {
                        expr.push('!').ok();
                    }
                }
            }
            *fresh = false;
            return;
        }
        '*' => {
            expr.clear();
            *fresh = true;
            return;
        }
        '#' => {
            if !expr.is_empty() {
                match crate::eval::eval(expr.as_str()) {
                    Ok(res) => {
                        *last = Some(res);
                        let mut tmp = heapless::String::<24>::new();
                        let res_s = crate::eval::format_f64(res);
                        write!(tmp, "={}", res_s.as_str()).ok();

                        if expr.len() + tmp.len() <= expr.capacity() {
                            expr.push_str(tmp.as_str()).ok();
                        } else {
                            let keep = expr.capacity().saturating_sub(tmp.len());
                            if keep > 0 {
                                let start = expr.len().saturating_sub(keep);
                                let tail = &expr.as_str()[start..];
                                let mut newexpr: heapless::String<N> = heapless::String::new();
                                newexpr.push_str(tail).ok();
                                newexpr.push_str(tmp.as_str()).ok();
                                expr.clear();
                                expr.push_str(newexpr.as_str()).ok();
                            } else {
                                expr.clear();
                                expr.push_str(tmp.as_str()).ok();
                            }
                        }
                        *fresh = true;
                    }
                    Err(e) => {
                        defmt::error!("Eval error: {}", e);
                        expr.clear();
                        expr.push_str("ERR").ok();
                        *fresh = true;
                    }
                }
            }
            return;
        }
        '^' | 'A' | 'B' | 'C' | 'D' | '+' | '-' | '/' => {
            let ch = if matches!(k, 'A' | 'B' | 'C' | 'D') { map_ad(k) } else { k };
            if expr.is_empty() {
                if ch == '-' {
                    expr.push('0').ok();
                    expr.push('-').ok();
                }
                return;
            }
            if let Some(last_ch) = expr.chars().last() {
                if last_ch.is_digit(10) || last_ch == ')' || last_ch == '!' {
                    expr.push(ch).ok();
                }
            }
            *fresh = false;
            return;
        }
        _ => {}
    }
}
