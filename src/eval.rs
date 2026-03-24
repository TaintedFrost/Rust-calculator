use heapless::{String as HString, Vec};

pub fn eval(expr: &str) -> Result<f64, &'static str> {
    fn precedence(op: char) -> i8 {
        match op {
            '+' | '-' => 1,
            '*' | '/' => 2,
            '^' => 3,
            'r' | 'l' | 's' | 'c' | 't' | 'k' => 4,
            '!' => 5,
            _ => 0,
        }
    }

    fn apply_op(nums: &mut Vec<f64, 32>, op: char) -> Result<(), &'static str> {
        match op {
            'r' | 'l' | 's' | 'c' | 't' | 'k' => {
                let a = nums.pop().ok_or("syntax")?;
                let r = match op {
                    'r' => {
                        if a < 0.0 { return Err("neg_sqrt"); }
                        libm::sqrt(a)
                    }
                    'l' => {
                        if a <= 0.0 { return Err("neg_log"); }
                        libm::log10(a)
                    }
                    's' => libm::sin(a * (core::f64::consts::PI / 180.0)),
                    'c' => libm::cos(a * (core::f64::consts::PI / 180.0)),
                    't' => libm::tan(a * (core::f64::consts::PI / 180.0)),
                    'k' => {
                        let tv = libm::tan(a * (core::f64::consts::PI / 180.0));
                        if libm::fabs(tv) < 1e-15 { return Err("div0"); }
                        1.0 / tv
                    }
                    _ => unreachable!(),
                };
                if !r.is_finite() { return Err("overflow"); }
                nums.push(r).map_err(|_| "overflow")?;
                Ok(())
            }
            '!' => {
                let a = nums.pop().ok_or("syntax")?;
                let a_floor = libm::floor(a);
                if (a - a_floor).abs() > 1e-12 { return Err("nonint_fact"); }
                if a_floor < 0.0 { return Err("neg_fact"); }
                const MAX_N: i32 = 20;
                let n = a_floor as i32;
                if n > MAX_N { return Err("overflow"); }
                let mut acc: f64 = 1.0;
                for i in 1..=n {
                    acc *= i as f64;
                    if !acc.is_finite() { return Err("overflow"); }
                }
                nums.push(acc).map_err(|_| "overflow")?;
                Ok(())
            }
            '+' | '-' | '*' | '/' | '^' => {
                let b = nums.pop().ok_or("syntax")?;
                let a = nums.pop().ok_or("syntax")?;
                let r = match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' => {
                        if b == 0.0 { return Err("div0"); }
                        a / b
                    }
                    '^' => {
                        let b_floor = libm::floor(b);
                        if (b - b_floor).abs() > 1e-12 { return Err("nonint_exp"); }
                        let bexp = b_floor as i128;
                        if bexp < 0 { return Err("neg_exp"); }
                        const MAX_EXP: i128 = 12;
                        if bexp > MAX_EXP { return Err("exp_too_large"); }
                        let r = libm::pow(a, bexp as f64);
                        if !r.is_finite() { return Err("overflow"); }
                        r
                    }
                    _ => unreachable!(),
                };
                if !r.is_finite() { return Err("overflow"); }
                nums.push(r).map_err(|_| "overflow")?;
                Ok(())
            }

            _ => Err("op"),
        }
    }

    let mut nums: Vec<f64, 32> = Vec::new();
    let mut ops: Vec<char, 32> = Vec::new();

    let mut cur: HString<20> = HString::new();
    let mut prev_was_num_or_close = false;
    let mut it = expr.chars().peekable();

    while let Some(ch) = it.next() {
        if ch.is_ascii_whitespace() { continue; }
        if ch.is_digit(10) || ch == '.' {
            cur.push(ch).map_err(|_| "overflow")?;
            while let Some(&nx) = it.peek() {
                if nx.is_digit(10) || nx == '.' {
                    let nx = it.next().unwrap();
                    cur.push(nx).map_err(|_| "overflow")?;
                } else { break; }
            }
            let val: f64 = cur.as_str().parse().map_err(|_| "parse")?;
            nums.push(val).map_err(|_| "overflow")?;
            cur.clear();
            prev_was_num_or_close = true;
            continue;
        }
        if ch == '(' {
            if prev_was_num_or_close {
                while let Some(&top) = ops.last() {
                    if top != '(' && precedence(top) >= precedence('*') {
                        let top = ops.pop().unwrap();
                        apply_op(&mut nums, top)?;
                    } else { break; }
                }
                ops.push('*').map_err(|_| "overflow")?;
            }
            ops.push('(').map_err(|_| "overflow")?;
            prev_was_num_or_close = false;
            continue;
        }
        if ch == ')' {
            while let Some(&top) = ops.last() {
                if top == '(' { break; }
                let top = ops.pop().unwrap();
                apply_op(&mut nums, top)?;
            }
            if ops.last().copied() == Some('(') { ops.pop(); }
            else { return Err("mismatch_paren"); }
            prev_was_num_or_close = true;
            continue;
        }
        if ch.is_ascii_alphabetic() {
            let mut name: HString<8> = HString::new();
            name.push(ch).map_err(|_| "overflow")?;
            while let Some(&nx) = it.peek() {
                if nx.is_ascii_alphabetic() {
                    let nx = it.next().unwrap();
                    name.push(nx).map_err(|_| "overflow")?;
                } else { break; }
            }
            let mut fname: HString<8> = HString::new();
            for ch2 in name.chars() {
                fname.push(ch2.to_ascii_lowercase()).map_err(|_| "overflow")?;
            }
            let fn_ch = match fname.as_str() {
                "r" | "sqrt" => 'r',
                "l" | "log"  => 'l',
                "s" | "sin"  => 's',
                "c" | "cos"  => 'c',
                "t" | "tan"  => 't',
                "k" | "cot"  => 'k',
                _ => return Err("badfunc"),
            };

            if prev_was_num_or_close {
                while let Some(&top) = ops.last() {
                    if top != '(' && precedence(top) >= precedence('*') {
                        let top = ops.pop().unwrap();
                        apply_op(&mut nums, top)?;
                    } else { break; }
                }
                ops.push('*').map_err(|_| "overflow")?;
            }

            ops.push(fn_ch).map_err(|_| "overflow")?;
            prev_was_num_or_close = false;
            continue;
        }
        if ch == '!' {
            while let Some(&top) = ops.last() {
                if top != '(' && precedence(top) >= precedence('!') {
                    let top = ops.pop().unwrap();
                    apply_op(&mut nums, top)?;
                } else { break; }
            }
            ops.push('!').map_err(|_| "overflow")?;
            prev_was_num_or_close = true;
            continue;
        }
        if matches!(ch, '+' | '-' | '*' | '/' | '^') {
            if ch == '-' && !prev_was_num_or_close {
                nums.push(0.0).map_err(|_| "overflow")?;
            }

            if ch == '^' {
                while let Some(&top) = ops.last() {
                    if top != '(' && precedence(top) > precedence(ch) {
                        let top = ops.pop().unwrap();
                        apply_op(&mut nums, top)?;
                    } else { break; }
                }
            } else {
                while let Some(&top) = ops.last() {
                    if top != '(' && precedence(top) >= precedence(ch) {
                        let top = ops.pop().unwrap();
                        apply_op(&mut nums, top)?;
                    } else { break; }
                }
            }

            ops.push(ch).map_err(|_| "overflow")?;
            prev_was_num_or_close = false;
            continue;
        }

        return Err("badchar");
    }
    if !cur.is_empty() {
        let val: f64 = cur.as_str().parse().map_err(|_| "parse")?;
        nums.push(val).map_err(|_| "overflow")?;
        cur.clear();
    }

    while let Some(op) = ops.pop() {
        if op == '(' || op == ')' { return Err("mismatch_paren"); }
        apply_op(&mut nums, op)?;
    }

    if nums.len() == 1 {
        Ok(nums.pop().unwrap())
    } else {
        Err("syntax")
    }
}

pub fn format_f64(v: f64) -> heapless::String<24> {
    use core::fmt::Write;
    let mut s: heapless::String<24> = heapless::String::new();

    if !v.is_finite() {
        write!(s, "ERR").ok();
        return s;
    }

    let abs = libm::fabs(v);
    let decimals: usize = if abs != 0.0 && abs < 1e-6 { 9 } else { 6 };
    write!(s, "{:.*}", decimals, v).ok();

    if s.as_str().contains('.') {
        while s.ends_with('0') { s.pop(); }
        if s.ends_with('.') { s.pop(); }
    }
    s
}
