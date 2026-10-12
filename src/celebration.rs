//! Bounded, time-based Rust canvas fireworks and balloons.
#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy)]
struct Balloon {
    x: f64,
    y: f64,
    speed: f64,
    width: f64,
    drift: f64,
    sway: f64,
    hue: u16,
    string: f64,
    phase: f64,
}
#[cfg(any(target_arch = "wasm32", test))]
fn balloons(count: u8, width: f64, height: f64, mut random: impl FnMut() -> f64) -> Vec<Balloon> {
    (0..count.min(100))
        .map(|i| Balloon {
            x: 20.0 + random() * (width - 40.0).max(0.0),
            y: height + 50.0 + random() * 100.0,
            speed: 0.9 + random() * 0.6,
            width: 16.0 + random() * 10.0,
            drift: 0.3 - random() * 0.6,
            sway: random() * 0.02,
            hue: (random() * 360.0) as u16,
            string: random() * 40.0 + 20.0,
            phase: i as f64,
        })
        .collect()
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use std::cell::RefCell;
    use wasm_bindgen::{prelude::*, JsCast};
    use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
    struct Particle {
        x: f64,
        y: f64,
        angle: f64,
        speed: f64,
        vy: f64,
        life: f64,
        decay: f64,
        hue: u16,
    }
    struct Animation {
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
        balloons: Vec<Balloon>,
        particles: Vec<Particle>,
        start: Option<f64>,
        last: f64,
        burst: usize,
        frame: i32,
        callback: Closure<dyn FnMut(f64)>,
    }
    impl Drop for Animation {
        fn drop(&mut self) {
            if let Some(w) = web_sys::window() {
                let _ = w.cancel_animation_frame(self.frame);
            }
            self.canvas.remove();
        }
    }
    thread_local! { static ANIMATION:RefCell<Option<Animation>>=const{RefCell::new(None)}; }
    const BURSTS: [(f64, usize); 4] = [(0.0, 80), (2.0, 60), (4.0, 60), (5.5, 40)];
    fn random() -> f64 {
        js_sys::Math::random()
    }
    impl Animation {
        fn draw(&mut self, timestamp: f64) -> bool {
            let start = *self.start.get_or_insert(timestamp);
            let elapsed = (timestamp - start) / 1000.0;
            let step = if self.last == 0.0 {
                1.0
            } else {
                ((timestamp - self.last) / 16.6667).clamp(0.0, 3.0)
            };
            self.last = timestamp;
            let width = self.canvas.width() as f64;
            let height = self.canvas.height() as f64;
            while self.burst < BURSTS.len() && elapsed >= BURSTS[self.burst].0 {
                for _ in 0..BURSTS[self.burst].1 {
                    self.particles.push(Particle {
                        x: random() * width,
                        y: height * (0.3 + random() * 0.6),
                        angle: random() * std::f64::consts::TAU,
                        speed: 1.0 + random() * 3.0,
                        vy: -5.0 - random() * 8.0,
                        life: 1.0,
                        decay: 0.003 + random() * 0.005,
                        hue: (random() * 360.0) as u16,
                    });
                }
                self.burst += 1;
            }
            self.ctx.clear_rect(0.0, 0.0, width, height);
            for p in &mut self.particles {
                p.x += p.angle.cos() * p.speed * step;
                p.y += p.vy * step;
                p.vy += 0.12 * step;
                p.life -= p.decay * step;
                if p.life <= 0.0 {
                    continue;
                }
                self.ctx.set_global_alpha(p.life);
                self.ctx
                    .set_fill_style_str(&format!("hsl({},100%,60%)", p.hue));
                self.ctx.begin_path();
                let _ = self
                    .ctx
                    .arc(p.x, p.y, 2.0 + p.life * 2.0, 0.0, std::f64::consts::TAU);
                self.ctx.fill();
            }
            self.particles.retain(|p| p.life > 0.0);
            self.ctx.set_global_alpha(1.0);
            for b in &mut self.balloons {
                b.x += (b.phase + timestamp * b.sway).sin() * b.drift * step;
                b.y -= b.speed * step;
                if b.y <= -80.0 {
                    continue;
                }
                let r = b.width / 2.0;
                let ctx = &self.ctx;
                ctx.set_stroke_style_str("#999");
                ctx.set_line_width(1.0);
                ctx.begin_path();
                ctx.move_to(b.x, b.y + r);
                ctx.line_to(b.x + 2.0, b.y + r + b.string);
                ctx.stroke();
                ctx.set_fill_style_str(&format!("hsl({},80%,55%)", b.hue));
                ctx.begin_path();
                let _ = ctx.ellipse(b.x, b.y, r, r * 1.2, 0.0, 0.0, std::f64::consts::TAU);
                ctx.fill();
                ctx.set_fill_style_str(&format!("hsl({},80%,65%)", b.hue));
                ctx.begin_path();
                let _ = ctx.ellipse(
                    b.x - r * 0.3,
                    b.y - r * 0.3,
                    r * 0.25,
                    r * 0.25,
                    0.0,
                    0.0,
                    std::f64::consts::TAU,
                );
                ctx.fill();
                ctx.set_stroke_style_str("#666");
                ctx.begin_path();
                ctx.move_to(b.x - r * 0.5, b.y + r * 1.1);
                ctx.line_to(b.x - r * 0.2, b.y + r * 1.3);
                ctx.line_to(b.x + r * 0.2, b.y + r * 1.3);
                ctx.line_to(b.x + r * 0.5, b.y + r * 1.1);
                ctx.stroke();
            }
            self.balloons.retain(|b| b.y > -80.0);
            elapsed < (height + 250.0) / 54.0 + 12.0
                && (self.burst < BURSTS.len()
                    || !self.particles.is_empty()
                    || !self.balloons.is_empty())
        }
    }
    pub fn stop() {
        ANIMATION.with(|cell| {
            cell.borrow_mut().take();
        });
    }
    pub fn start(count: u8) {
        stop();
        let result = (|| -> Result<Animation, JsValue> {
            let window = web_sys::window().ok_or(JsValue::NULL)?;
            let document = window.document().ok_or(JsValue::NULL)?;
            let canvas: HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;
            let width = window.inner_width()?.as_f64().unwrap_or(320.0);
            let height = window.inner_height()?.as_f64().unwrap_or(844.0);
            canvas.set_width(width as u32);
            canvas.set_height(height as u32);
            canvas.set_class_name("victory-effects");
            canvas.set_attribute(
                "style",
                "position:fixed;inset:0;pointer-events:none;z-index:9999",
            )?;
            canvas.set_attribute("aria-hidden", "true")?;
            canvas.set_attribute("data-balloon-count", &count.min(100).to_string())?;
            let ctx: CanvasRenderingContext2d =
                canvas.get_context("2d")?.ok_or(JsValue::NULL)?.dyn_into()?;
            document
                .body()
                .ok_or(JsValue::NULL)?
                .append_child(&canvas)?;
            let callback = Closure::wrap(Box::new(move |timestamp: f64| {
                let done = ANIMATION.with(|cell| {
                    let mut animation = cell.borrow_mut();
                    let Some(a) = animation.as_mut() else {
                        return false;
                    };
                    if !a.draw(timestamp) {
                        return true;
                    }
                    if let Some(w) = web_sys::window() {
                        match w.request_animation_frame(a.callback.as_ref().unchecked_ref()) {
                            Ok(frame) => a.frame = frame,
                            Err(_) => return true,
                        }
                    }
                    false
                });
                if done {
                    wasm_bindgen_futures::spawn_local(async {
                        stop();
                    });
                }
            }) as Box<dyn FnMut(f64)>);
            let frame = match window.request_animation_frame(callback.as_ref().unchecked_ref()) {
                Ok(frame) => frame,
                Err(error) => {
                    canvas.remove();
                    return Err(error);
                }
            };
            Ok(Animation {
                canvas,
                ctx,
                balloons: balloons(count, width, height, random),
                particles: Vec::with_capacity(240),
                start: None,
                last: 0.0,
                burst: 0,
                frame,
                callback,
            })
        })();
        match result {
            Ok(animation) => ANIMATION.with(|cell| *cell.borrow_mut() = Some(animation)),
            Err(error) => web_sys::console::warn_2(
                &JsValue::from_str("Sudoku celebration unavailable"),
                &error,
            ),
        }
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::{start, stop};
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_are_bounded_and_balloon_parameters_preserve_the_visual_range() {
        for (count, expected) in [(0, 0), (25, 25), (100, 100), (255, 100)] {
            let items = balloons(count, 320.0, 844.0, || 0.5);
            assert_eq!(items.len(), expected);
            for b in items {
                assert!((0.9..=1.5).contains(&b.speed));
                assert!((16.0..=26.0).contains(&b.width));
                assert!(b.y > 844.0);
                assert!((20.0..=300.0).contains(&b.x));
                assert!(b.drift.abs() <= 0.3);
                assert!(b.sway <= 0.02);
                assert!(b.hue < 360);
                assert!((20.0..=60.0).contains(&b.string));
                assert!(b.phase < 100.0);
            }
        }
    }
}
