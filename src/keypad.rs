use embassy_time::{Duration, Timer};
use embassy_rp::gpio::{Output, Input};
use crate::input::handle_input;

pub const KEYMAP_4X4: [[char; 4]; 4] = [
    ['b','(',')','A'],
    ['^','r','.','B'],
    ['s','c','t','C'],
    ['l','k','!','D'],
];

pub const KEYMAP_4X3: [[char; 3]; 4] = [
    ['1','2','3'],
    ['4','5','6'],
    ['7','8','9'],
    ['a','0','#'],
];

async fn click(buzzer: &mut Output<'_>) {
    buzzer.set_high();
    Timer::after(Duration::from_millis(50)).await;
    buzzer.set_low();
}

pub async fn scan_keypad_4x4<'a>(
    cols: &mut [Output<'a>; 4],
    rows: &[Input<'a>; 4],
    expr: &mut heapless::String<64>,
    fresh: &mut bool,
    last: &mut Option<f64>,
    buzzer: &mut Output<'a>,
) {
    for c in cols.iter_mut() { c.set_high(); }
    for ci in 0..4 {
        cols[ci].set_low();
        Timer::after(Duration::from_millis(5)).await;
        for ri in 0..4 {
            if rows[ri].is_low() {
                Timer::after(Duration::from_millis(20)).await;
                if rows[ri].is_low() {
                    let k = KEYMAP_4X4[ri][ci];
                    defmt::info!("Pressed {}", k);
                    click(buzzer).await;
                    // long press 
                    const SAMPLE_MS: u64 = 50;
                    const LONG_MS: u64 = 1000;
                    let mut elapsed_ms: u64 = 0;

                    loop {
                        if !rows[ri].is_low() {
                            handle_input(k, expr, fresh, last);
                            break;
                        }
                        if elapsed_ms >= LONG_MS {
                            if k == 'b' {
                                expr.clear();
                                *fresh = true;
                                defmt::info!("Long press backspace -> clear");
                            } else {
                                handle_input(k, expr, fresh, last);
                            }
                            while rows[ri].is_low() {
                                Timer::after(Duration::from_millis(SAMPLE_MS)).await;
                            }
                            break;
                        }
                        Timer::after(Duration::from_millis(SAMPLE_MS)).await;
                        elapsed_ms += SAMPLE_MS;
                    }
                    break;
                }
            }
        }
        cols[ci].set_high();
    }
}

pub async fn scan_keypad_4x3<'a>(
    cols: &mut [Output<'a>; 3],
    rows: &[Input<'a>; 4],
    expr: &mut heapless::String<64>,
    fresh: &mut bool,
    last: &mut Option<f64>,
    buzzer: &mut Output<'a>,
) {
    for c in cols.iter_mut() { c.set_high(); }
    for ci in 0..3 {
        cols[ci].set_low();
        Timer::after(Duration::from_millis(5)).await;
        for ri in 0..4 {
            if rows[ri].is_low() {
                Timer::after(Duration::from_millis(20)).await;
                if rows[ri].is_low() {
                    let k = KEYMAP_4X3[ri][ci];
                    defmt::info!("Pressed {}", k);
                    click(buzzer).await;
                    const SAMPLE_MS: u64 = 50;
                    const LONG_MS: u64 = 2000;
                    let mut elapsed_ms: u64 = 0;

                    loop {
                        if !rows[ri].is_low() {
                            handle_input(k, expr, fresh, last);
                            break;
                        }
                        if elapsed_ms >= LONG_MS {
                            if k == 'b' {
                                expr.clear();
                                *fresh = true;
                                defmt::info!("Long press backspace -> clear");
                            } else {
                                handle_input(k, expr, fresh, last);
                            }
                            while rows[ri].is_low() {
                                Timer::after(Duration::from_millis(SAMPLE_MS)).await;
                            }
                            break;
                        }
                        Timer::after(Duration::from_millis(SAMPLE_MS)).await;
                        elapsed_ms += SAMPLE_MS;
                    }
                    break;
                }
            }
        }
        cols[ci].set_high();
    }
}
