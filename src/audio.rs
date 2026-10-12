//! Rust synthesis and a single, gesture-unlocked Web Audio service.
#[cfg(any(target_arch = "wasm32", test))]
use crate::state::SoundType;

#[cfg(any(target_arch = "wasm32", test))]
const BOOMS: [f64; 20] = [
    0.05, 0.15, 0.3, 0.5, 0.7, 0.95, 1.2, 1.5, 1.8, 2.1, 2.5, 2.9, 3.3, 3.7, 4.1, 4.5, 5.0, 5.5,
    6.0, 6.5,
];

#[cfg(any(target_arch = "wasm32", test))]
fn samples(kind: SoundType, victory: bool, rate: f64) -> Vec<f32> {
    use std::f64::consts::TAU;
    let duration = if kind == SoundType::Beep {
        if victory {
            8.0
        } else {
            0.12
        }
    } else {
        0.38
    };
    let mut seed = 0x12345678u32;
    let mut low = 0.0;
    let mut phase = [0.0; 3];
    (0..(duration * rate) as usize)
        .map(|i| {
            let t = i as f64 / rate;
            let value = match kind {
                SoundType::None => 0.0,
                SoundType::Beep if !victory => 0.33 * (TAU * 660.0 * t).sin() * (-46.0 * t).exp(),
                SoundType::Beep => {
                    let sweep = if t < 3.0 {
                        0.5 * 3f64.powf(t / 3.0)
                    } else {
                        1.5 * 0.4f64.powf((t - 3.0) / 3.0)
                    };
                    let mut v = 0.0;
                    for (j, f) in [523.0, 659.0, 784.0].into_iter().enumerate() {
                        phase[j] += TAU * f * sweep / rate;
                        v += phase[j].sin();
                    }
                    0.22 * v * (1.0 - t / 8.0).max(0.0).powi(2)
                }
                SoundType::Explosion => {
                    seed ^= seed << 13;
                    seed ^= seed >> 17;
                    seed ^= seed << 5;
                    let noise = seed as f64 / u32::MAX as f64 * 2.0 - 1.0;
                    // A broader noise band carries the transient on small phone speakers.
                    low += (1.0 - (-TAU * 1400.0 / rate).exp()) * (noise - low);
                    let thump = (TAU * (120.0 * t - 65.0 * t * t)).sin() * (-24.0 * t).exp();
                    let attack = (t / 0.003).min(1.0);
                    (0.9 * low + 0.3 * thump) * attack * (1.0 - t / duration).max(0.0).powi(2)
                }
            };
            value as f32
        })
        .collect()
}

#[cfg(any(target_arch = "wasm32", test))]
fn limited(value: f64) -> f32 {
    (0.95 * (value * 1.1).tanh()) as f32
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use std::{cell::RefCell, collections::VecDeque};
    use wasm_bindgen::{prelude::*, JsCast};
    use wasm_bindgen_futures::{spawn_local, JsFuture};
    use web_sys::{
        AudioBuffer, AudioBufferSourceNode, AudioContext, Event, EventTarget, GainNode,
        WaveShaperNode,
    };

    struct Voice {
        id: u64,
        source: AudioBufferSourceNode,
        _ended: Closure<dyn FnMut(Event)>,
    }
    struct Graph {
        context: AudioContext,
        master: GainNode,
        _limiter: WaveShaperNode,
        clips: Vec<Option<AudioBuffer>>,
    }
    struct Service {
        graph: Option<Graph>,
        voices: Vec<Voice>,
        pending: VecDeque<(SoundType, bool)>,
        kind: SoundType,
        volume: u8,
        resuming: bool,
        epoch: u64,
        next_id: u64,
        warned: bool,
    }
    impl Default for Service {
        fn default() -> Self {
            Self {
                graph: None,
                voices: vec![],
                pending: VecDeque::new(),
                kind: SoundType::Explosion,
                volume: 75,
                resuming: false,
                epoch: 0,
                next_id: 0,
                warned: false,
            }
        }
    }
    struct Binding {
        target: EventTarget,
        name: &'static str,
        callback: Closure<dyn FnMut(Event)>,
    }
    impl Drop for Binding {
        fn drop(&mut self) {
            let _ = self.target.remove_event_listener_with_callback_and_bool(
                self.name,
                self.callback.as_ref().unchecked_ref(),
                true,
            );
        }
    }
    thread_local! {
        static SERVICE:RefCell<Service> = RefCell::new(Service::default());
        static BINDINGS:RefCell<Vec<Binding>> = const {RefCell::new(Vec::new())};
    }
    fn hidden() -> bool {
        web_sys::window()
            .and_then(|w| w.document())
            .is_none_or(|d| d.hidden())
    }
    fn status(context: &AudioContext) -> String {
        js_sys::Reflect::get(context.as_ref(), &JsValue::from_str("state"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default()
    }
    impl Service {
        fn warn(&mut self, error: JsValue) {
            if !self.warned {
                web_sys::console::warn_2(
                    &JsValue::from_str(
                        "Sudoku audio unavailable; retrying on the next interaction",
                    ),
                    &error,
                );
                self.warned = true;
            }
        }
        fn cancel(&mut self) {
            self.epoch = self.epoch.wrapping_add(1);
            self.resuming = false;
            self.pending.clear();
            for voice in self.voices.drain(..) {
                voice
                    .source
                    .unchecked_ref::<web_sys::AudioScheduledSourceNode>()
                    .set_onended(None);
                let _ = voice
                    .source
                    .unchecked_ref::<web_sys::AudioScheduledSourceNode>()
                    .stop();
                let _ = voice.source.disconnect();
            }
        }
        fn create(&mut self) -> Result<(), JsValue> {
            self.cancel();
            let context = AudioContext::new()?;
            let graph = (|| {
                let master = context.create_gain()?;
                master.gain().set_value(self.volume as f32 / 100.0);
                let limiter = context.create_wave_shaper()?;
                let mut curve: Vec<f32> = (0..4097)
                    .map(|i| limited(i as f64 / 2048.0 - 1.0))
                    .collect();
                limiter.set_curve_opt_f32_slice(Some(&mut curve));
                master.connect_with_audio_node(&limiter)?;
                limiter.connect_with_audio_node(&context.destination())?;
                Ok::<_, JsValue>(Graph {
                    context: context.clone(),
                    master,
                    _limiter: limiter,
                    clips: vec![None, None, None],
                })
            })();
            match graph {
                Ok(graph) => self.graph = Some(graph),
                Err(error) => {
                    if let Ok(promise) = context.close() {
                        spawn_local(async move {
                            let _ = JsFuture::from(promise).await;
                        });
                    }
                    return Err(error);
                }
            }
            Ok(())
        }
        fn schedule(&mut self, kind: SoundType, victory: bool) -> Result<(), JsValue> {
            let Some(graph) = self.graph.as_mut() else {
                return Ok(());
            };
            let index = match kind {
                SoundType::None => return Ok(()),
                SoundType::Explosion => 0,
                SoundType::Beep => {
                    if victory {
                        2
                    } else {
                        1
                    }
                }
            };
            if graph.clips[index].is_none() {
                let data = samples(kind, victory, graph.context.sample_rate() as f64);
                let buffer = graph.context.create_buffer(
                    1,
                    data.len() as u32,
                    graph.context.sample_rate(),
                )?;
                buffer.copy_to_channel(&data, 0)?;
                graph.clips[index] = Some(buffer);
            }
            let times: &[f64] = if victory && kind == SoundType::Explosion {
                &BOOMS
            } else {
                &[0.0]
            };
            let now = graph.context.current_time();
            for delay in times {
                if self.voices.len() >= 32 {
                    break;
                }
                let source = graph.context.create_buffer_source()?;
                source.set_buffer(graph.clips[index].as_ref());
                source.connect_with_audio_node(&graph.master)?;
                self.next_id = self.next_id.wrapping_add(1);
                let id = self.next_id;
                let ended = Closure::wrap(Box::new(move |_: Event| {
                    // Release a callback only after its current invocation has returned.
                    spawn_local(async move {
                        SERVICE.with(|cell| {
                            let mut service = cell.borrow_mut();
                            if let Some(i) = service.voices.iter().position(|v| v.id == id) {
                                let voice = service.voices.swap_remove(i);
                                voice
                                    .source
                                    .unchecked_ref::<web_sys::AudioScheduledSourceNode>()
                                    .set_onended(None);
                                let _ = voice.source.disconnect();
                            }
                        });
                    });
                }) as Box<dyn FnMut(Event)>);
                source
                    .unchecked_ref::<web_sys::AudioScheduledSourceNode>()
                    .set_onended(Some(ended.as_ref().unchecked_ref()));
                if let Err(error) = source.start_with_when(now + delay) {
                    source
                        .unchecked_ref::<web_sys::AudioScheduledSourceNode>()
                        .set_onended(None);
                    let _ = source.disconnect();
                    return Err(error);
                }
                self.voices.push(Voice {
                    id,
                    source,
                    _ended: ended,
                });
            }
            Ok(())
        }
    }
    fn resume() {
        let pending = SERVICE.with(|cell| {
            let mut s = cell.borrow_mut();
            if s.kind == SoundType::None || s.volume == 0 || hidden() {
                return None;
            }
            if s.graph
                .as_ref()
                .is_some_and(|g| status(&g.context) == "closed")
            {
                s.cancel();
                s.graph = None;
            }
            if s.graph.is_none() {
                if let Err(error) = s.create() {
                    s.warn(error);
                    return None;
                }
            }
            let context = s.graph.as_ref()?.context.clone();
            if status(&context) == "running" || s.resuming {
                return None;
            }
            match context.resume() {
                Ok(promise) => {
                    s.resuming = true;
                    Some((promise, s.epoch))
                }
                Err(error) => {
                    s.warn(error);
                    None
                }
            }
        });
        if let Some((promise, epoch)) = pending {
            spawn_local(async move {
                let result = JsFuture::from(promise).await;
                SERVICE.with(|cell| {
                    let mut s = cell.borrow_mut();
                    if s.epoch != epoch {
                        return;
                    }
                    s.resuming = false;
                    if let Err(error) = result {
                        s.pending.clear();
                        s.warn(error);
                        return;
                    }
                    if hidden()
                        || !s
                            .graph
                            .as_ref()
                            .is_some_and(|g| status(&g.context) == "running")
                    {
                        s.pending.clear();
                        return;
                    }
                    s.warned = false;
                    while let Some((kind, victory)) = s.pending.pop_front() {
                        if let Err(error) = s.schedule(kind, victory) {
                            s.warn(error);
                            break;
                        }
                    }
                });
            });
        }
    }
    pub fn configure(kind: SoundType, volume: u8) {
        SERVICE.with(|cell| {
            let mut s = cell.borrow_mut();
            let volume = volume.min(100);
            if kind != s.kind || volume == 0 {
                s.cancel();
            }
            if s.kind == kind && s.volume == volume {
                return;
            }
            s.kind = kind;
            s.volume = volume;
            if let Some(graph) = s.graph.as_ref() {
                let gain = graph.master.gain();
                let now = graph.context.current_time();
                let _ = gain.cancel_scheduled_values(now);
                let _ = gain.set_value_at_time(gain.value(), now);
                let _ = gain.linear_ramp_to_value_at_time(
                    if kind == SoundType::None {
                        0.0
                    } else {
                        volume as f32 / 100.0
                    },
                    now + 0.02,
                );
            }
        });
    }
    pub fn play(kind: SoundType, volume: u8, victory: bool) {
        configure(kind, volume);
        if kind == SoundType::None || volume == 0 || hidden() {
            return;
        }
        resume();
        SERVICE.with(|cell| {
            let mut s = cell.borrow_mut();
            if s.graph
                .as_ref()
                .is_some_and(|g| status(&g.context) == "running")
            {
                if let Err(error) = s.schedule(kind, victory) {
                    s.warn(error);
                }
            } else if s.graph.is_some() && s.pending.len() < 32 {
                s.pending.push_back((kind, victory));
            }
        });
        // This call is synchronous with a button handler for previews/entries.
        // A delayed domino uses the context unlocked by its initiating entry.
        resume();
    }
    pub fn install() {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        BINDINGS.with(|bindings| {
            let mut bindings = bindings.borrow_mut();
            if !bindings.is_empty() {
                return;
            }
            for name in ["click", "pointerup", "keydown", "visibilitychange"] {
                let callback = Closure::wrap(Box::new(move |event: Event| {
                    if name == "visibilitychange" {
                        if hidden() {
                            SERVICE.with(|cell| cell.borrow_mut().cancel());
                        }
                    } else if event.is_trusted() {
                        resume();
                    }
                }) as Box<dyn FnMut(Event)>);
                let target: EventTarget = document.clone().into();
                if target
                    .add_event_listener_with_callback_and_bool(
                        name,
                        callback.as_ref().unchecked_ref(),
                        true,
                    )
                    .is_ok()
                {
                    bindings.push(Binding {
                        target,
                        name,
                        callback,
                    });
                }
            }
        });
    }
    pub fn cancel() {
        SERVICE.with(|cell| cell.borrow_mut().cancel());
    }
    pub fn shutdown() {
        BINDINGS.with(|b| b.borrow_mut().clear());
        SERVICE.with(|cell| {
            let mut s = cell.borrow_mut();
            s.cancel();
            if let Some(g) = s.graph.take() {
                let _ = g.master.disconnect();
                if let Ok(p) = g.context.close() {
                    spawn_local(async move {
                        let _ = JsFuture::from(p).await;
                    });
                }
            }
        });
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::{cancel, configure, install, play, shutdown};

#[cfg(test)]
mod tests {
    use super::*;
    fn rms(data: &[f32]) -> f64 {
        (data.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / data.len() as f64).sqrt()
    }
    #[test]
    fn volume_is_silent_monotonic_and_victory_mix_does_not_clip() {
        let rate = 48000.0;
        let clip = samples(SoundType::Explosion, false, rate);
        let levels: Vec<_> = [0.0, 0.25, 0.75, 1.0]
            .map(|level| {
                rms(&clip
                    .iter()
                    .map(|v| limited(*v as f64 * level))
                    .collect::<Vec<_>>())
            })
            .to_vec();
        assert_eq!(levels[0], 0.0);
        assert!(levels.windows(2).all(|pair| pair[0] < pair[1]));
        let mut mix = vec![0.0f64; (7.0 * rate) as usize];
        for delay in BOOMS {
            for (i, value) in clip.iter().enumerate() {
                mix[(delay * rate) as usize + i] += *value as f64;
            }
        }
        assert!(mix.into_iter().all(|v| limited(v).abs() < 1.0));
        assert!(samples(SoundType::None, false, rate)
            .iter()
            .all(|v| *v == 0.0));
    }
    #[test]
    fn default_explosion_is_at_least_six_db_above_legacy() {
        let rate = 48000.0;
        let mut seed = 0x12345678u32;
        let mut filtered = 0.0;
        let legacy: Vec<f32> = (0..(0.38 * rate) as usize)
            .map(|i| {
                let t = i as f64 / rate;
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                let noise = if t < 0.25 {
                    (seed as f64 / u32::MAX as f64 * 2.0 - 1.0) * (1.0 - t / 0.25).powi(2)
                } else {
                    0.0
                };
                filtered +=
                    (1.0 - (-std::f64::consts::TAU * 300.0 / rate).exp()) * (noise - filtered);
                (filtered * 0.5 * (-(0.5f64 / 0.001).ln() * t / 0.3).exp()) as f32
            })
            .collect();
        let louder: Vec<f32> = samples(SoundType::Explosion, false, rate)
            .iter()
            .map(|v| limited(*v as f64 * 0.75))
            .collect();
        assert!(20.0 * (rms(&louder) / rms(&legacy)).log10() >= 6.0);
    }
}
