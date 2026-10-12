//! Explicit, per-tab consent for complete PWA updates. Game saves stay compatible.
#[cfg(any(target_arch = "wasm32", test))]
use crate::state::GameState;
#[cfg(any(target_arch = "wasm32", test))]
use serde::{Deserialize, Serialize};
#[cfg(any(target_arch = "wasm32", test))]
#[derive(Serialize, Deserialize)]
struct Resume {
    target: String,
    game: GameState,
}
#[cfg(any(target_arch = "wasm32", test))]
fn decode_resume(json: &str, bundle: &str) -> Option<GameState> {
    let resume: Resume = serde_json::from_str(json).ok()?;
    if bundle.is_empty() || resume.target != bundle {
        return None;
    }
    crate::state::decode_saved_state(&serde_json::to_string(&resume.game).ok()?).ok()
}
#[cfg(target_arch = "wasm32")]
pub fn take_resume() -> Option<GameState> {
    let window = web_sys::window()?;
    let bundle = window
        .document()?
        .query_selector("meta[name=sudoku-bundle]")
        .ok()??
        .get_attribute("content")?;
    let storage = window.session_storage().ok()??;
    let json = storage.get_item("sudoku_update_resume").ok()??;
    let state = decode_resume(&json, &bundle)?;
    storage.remove_item("sudoku_update_resume").ok()?;
    Some(state)
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use crate::{
        game_history::HistoryEntry,
        state::{AppState, GameHistory},
    };
    use futures_channel::oneshot;
    use leptos::prelude::*;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        time::Duration,
    };
    use wasm_bindgen::{prelude::*, JsCast};
    use wasm_bindgen_futures::{spawn_local, JsFuture};
    use web_sys::{
        Event, EventTarget, MessageChannel, MessageEvent, ServiceWorker, ServiceWorkerRegistration,
    };

    fn field(value: &JsValue, name: &str) -> String {
        js_sys::Reflect::get(value, &name.into())
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default()
    }
    fn packet(kind: &str, version: &str) -> JsValue {
        let value = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&value, &"type".into(), &kind.into());
        let _ = js_sys::Reflect::set(&value, &"version".into(), &version.into());
        value.into()
    }
    async fn version(worker: &ServiceWorker) -> Option<String> {
        let channel = MessageChannel::new().ok()?;
        let (send, receive) = oneshot::channel();
        let send = Rc::new(RefCell::new(Some(send)));
        let reply = send.clone();
        let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if field(&event.data(), "type") == "SUDOKU_UPDATE_STATUS" {
                if let Some(send) = reply.borrow_mut().take() {
                    let _ = send.send(Some(field(&event.data(), "version")));
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        channel
            .port1()
            .set_onmessage(Some(callback.as_ref().unchecked_ref()));
        let timeout = set_timeout_with_handle(
            move || {
                if let Some(send) = send.borrow_mut().take() {
                    let _ = send.send(None);
                }
            },
            Duration::from_secs(3),
        )
        .ok()?;
        let ports = js_sys::Array::new();
        ports.push(&channel.port2());
        let sent =
            worker.post_message_with_transferable(&packet("SUDOKU_UPDATE_STATUS", ""), &ports);
        let result = if sent.is_ok() {
            receive.await.ok().flatten()
        } else {
            None
        };
        timeout.clear();
        channel.port1().set_onmessage(None);
        channel.port1().close();
        result.filter(|v| !v.is_empty())
    }
    fn reserve_notice(height: f64) {
        if let Some(root) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            if let Some(root) = root.dyn_ref::<web_sys::HtmlElement>() {
                let _ = root
                    .style()
                    .set_property("--update-notice-height", &format!("{}px", height));
            }
        }
    }
    struct NoticeObserver {
        observer: web_sys::ResizeObserver,
        _callback: Closure<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>,
    }
    impl Drop for NoticeObserver {
        fn drop(&mut self) {
            self.observer.disconnect();
        }
    }
    struct Binding {
        target: EventTarget,
        name: &'static str,
        callback: Closure<dyn FnMut(Event)>,
    }
    impl Drop for Binding {
        fn drop(&mut self) {
            let _ = self.target.remove_event_listener_with_callback(
                self.name,
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }
    struct Runtime {
        state: AppState,
        history: GameHistory,
        target: RwSignal<Option<String>>,
        dismissed: RwSignal<bool>,
        preparing: RwSignal<bool>,
        ready: RwSignal<bool>,
        committing: RwSignal<bool>,
        failed: RwSignal<bool>,
        original: RefCell<Option<GameState>>,
        worker: RefCell<Option<ServiceWorker>>,
        registration: RefCell<Option<ServiceWorkerRegistration>>,
        bindings: RefCell<Vec<Binding>>,
        notice_observer: RefCell<Option<NoticeObserver>>,
        last_check: Cell<f64>,
        checking: Cell<bool>,
        alive: Cell<bool>,
        enabled: Cell<bool>,
        reload: Cell<bool>,
        epoch: Cell<u64>,
    }
    impl Runtime {
        fn bind(
            self: &Rc<Self>,
            target: EventTarget,
            name: &'static str,
            action: impl Fn(Event) + 'static,
        ) {
            let callback = Closure::wrap(Box::new(action) as Box<dyn FnMut(Event)>);
            if target
                .add_event_listener_with_callback(name, callback.as_ref().unchecked_ref())
                .is_ok()
            {
                self.bindings.borrow_mut().push(Binding {
                    target,
                    name,
                    callback,
                });
            }
        }
        fn cancel(&self) {
            if self.committing.get_untracked() {
                return;
            }
            self.epoch.set(self.epoch.get().wrapping_add(1));
            self.ready.set(false);
            self.preparing.set(false);
            if let Some(original) = self.original.borrow_mut().take() {
                self.state.0.set(original);
            }
            if let Some(storage) =
                web_sys::window().and_then(|w| w.session_storage().ok().flatten())
            {
                let _ = storage.remove_item("sudoku_update_resume");
            }
        }
        fn cleanup_cache(&self) {
            if !self.enabled.get() {
                return;
            }
            if let Some(window) = web_sys::window() {
                let running = window
                    .document()
                    .and_then(|d| d.query_selector("meta[name=sudoku-bundle]").ok().flatten())
                    .and_then(|m| m.get_attribute("content"));
                if let (Some(controller), Some(running)) =
                    (window.navigator().service_worker().controller(), running)
                {
                    let _ = controller.post_message(&packet("SUDOKU_UPDATE_CLEANUP", &running));
                }
            }
        }
        fn attempt(self: &Rc<Self>) {
            if !self.alive.get() || !self.ready.get_untracked() || self.committing.get_untracked() {
                return;
            }
            if let (Some(worker), Some(target)) =
                (self.worker.borrow().as_ref(), self.target.get_untracked())
            {
                let controller = self
                    .registration
                    .borrow()
                    .as_ref()
                    .and_then(|r| r.active())
                    .or_else(|| {
                        web_sys::window().and_then(|w| w.navigator().service_worker().controller())
                    });
                if controller.is_some_and(|c| js_sys::Object::is(c.as_ref(), worker.as_ref())) {
                    self.changed();
                } else {
                    let _ = worker.post_message(&packet("SUDOKU_UPDATE_ACTIVATE", &target));
                }
            }
        }
        fn inspect(self: &Rc<Self>) {
            let Some(registration) = self.registration.borrow().clone() else {
                return;
            };
            let waiting = registration.waiting().is_some();
            let Some(worker) = registration.waiting().or_else(|| registration.active()) else {
                return;
            };
            let this = self.clone();
            spawn_local(async move {
                let Some(target) = version(&worker).await else {
                    return;
                };
                if !this.alive.get() {
                    return;
                }
                // Ignore a worker superseded while its status request was pending.
                if !this
                    .registration
                    .borrow()
                    .as_ref()
                    .and_then(|r| r.waiting().or_else(|| r.active()))
                    .is_some_and(|w| js_sys::Object::is(w.as_ref(), worker.as_ref()))
                {
                    return;
                }
                if !waiting
                    && web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.query_selector("meta[name=sudoku-bundle]").ok().flatten())
                        .and_then(|m| m.get_attribute("content"))
                        .as_deref()
                        == Some(&target)
                {
                    return;
                }
                if this.target.get_untracked().as_ref() != Some(&target) {
                    if this.committing.get_untracked() {
                        return;
                    }
                    this.cancel();
                    this.dismissed.set(false);
                    this.failed.set(false);
                    this.target.set(Some(target));
                }
                *this.worker.borrow_mut() = Some(worker);
            });
        }
        fn check(self: &Rc<Self>, foreground: bool) {
            if !self.alive.get() {
                return;
            }
            if foreground {
                self.dismissed.set(false);
            }
            self.inspect();
            self.attempt();
            self.cleanup_cache();
            let now = js_sys::Date::now();
            if self.checking.get() || now - self.last_check.get() < 60_000.0 {
                return;
            }
            let Some(registration) = self.registration.borrow().clone() else {
                return;
            };
            self.last_check.set(now);
            self.checking.set(true);
            let this = self.clone();
            spawn_local(async move {
                if let Ok(promise) = registration.update() {
                    let _ = JsFuture::from(promise).await;
                }
                if this.alive.get() {
                    this.checking.set(false);
                    this.inspect();
                }
            });
        }
        fn accept(self: &Rc<Self>) {
            if self.preparing.get_untracked() || self.ready.get_untracked() {
                return;
            }
            let Some(target) = self.target.get_untracked() else {
                return;
            };
            self.failed.set(false);
            self.preparing.set(true);
            self.epoch.set(self.epoch.get().wrapping_add(1));
            let epoch = self.epoch.get();
            self.prepare_when_idle(target, epoch);
        }
        fn prepare_when_idle(self: &Rc<Self>, target: String, epoch: u64) {
            if !self.alive.get() || self.epoch.get() != epoch {
                return;
            }
            if self.history.busy.get_untracked() {
                let this = self.clone();
                set_timeout(
                    move || this.prepare_when_idle(target, epoch),
                    Duration::from_millis(100),
                );
                return;
            }
            let original = self.state.0.get_untracked();
            *self.original.borrow_mut() = Some(original.clone());
            self.state.0.update(|s| s.freeze_for_replacement());
            crate::audio::cancel();
            crate::celebration::stop();
            let this = self.clone();
            spawn_local(async move {
                let result = async {
                    GameHistory::save_entry(&HistoryEntry::capture(&original), false)
                        .await
                        .map_err(|_| JsValue::from_str("history save failed"))?;
                    // A newer waiting bundle may revoke this attempt while IndexedDB awaits.
                    if !this.alive.get() || this.epoch.get() != epoch {
                        return Err(JsValue::NULL);
                    }
                    let window = web_sys::window().ok_or(JsValue::NULL)?;
                    let local = window.local_storage()?.ok_or(JsValue::NULL)?;
                    let session = window.session_storage()?.ok_or(JsValue::NULL)?;
                    let game = serde_json::to_string(&original).map_err(|_| JsValue::NULL)?;
                    local.set_item("sudoku_state", &game)?;
                    if local.get_item("sudoku_state")?.as_deref() != Some(&game) {
                        return Err(JsValue::NULL);
                    }
                    let snapshot = serde_json::to_string(&Resume {
                        target,
                        game: original,
                    })
                    .map_err(|_| JsValue::NULL)?;
                    session.set_item("sudoku_update_resume", &snapshot)?;
                    if session.get_item("sudoku_update_resume")?.as_deref() != Some(&snapshot) {
                        return Err(JsValue::NULL);
                    }
                    Ok::<(), JsValue>(())
                }
                .await;
                if !this.alive.get() || this.epoch.get() != epoch {
                    return;
                }
                if result.is_err() {
                    this.cancel();
                    this.failed.set(true);
                } else {
                    this.preparing.set(false);
                    this.ready.set(true);
                    this.attempt();
                }
            });
        }
        fn receive(self: &Rc<Self>, event: MessageEvent) {
            let data = event.data();
            let kind = field(&data, "type");
            let target = field(&data, "version");
            if kind == "SUDOKU_UPDATE_QUERY" && field(&data, "phase") == "running" {
                let running = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.query_selector("meta[name=sudoku-bundle]").ok().flatten())
                    .and_then(|m| m.get_attribute("content"));
                let reply = packet("SUDOKU_UPDATE_ACK", &target);
                let _ = js_sys::Reflect::set(
                    &reply,
                    &"ready".into(),
                    &(running.as_deref() == Some(&target)).into(),
                );
                if let Some(port) = event.ports().get(0).dyn_ref::<web_sys::MessagePort>() {
                    let _ = port.post_message(&reply);
                }
                return;
            }
            let source = event.source();
            if !self.worker.borrow().as_ref().is_some_and(|w| {
                source
                    .as_ref()
                    .is_some_and(|s| js_sys::Object::is(w.as_ref(), s))
            }) {
                return;
            }
            if kind == "SUDOKU_UPDATE_ABORT" {
                if self.target.get_untracked().as_deref() == Some(&target) {
                    self.committing.set(false);
                }
                return;
            }
            if kind != "SUDOKU_UPDATE_QUERY" {
                return;
            }
            let ready = self.ready.get_untracked()
                && self.target.get_untracked().as_deref() == Some(&target);
            if ready && field(&data, "phase") == "commit" {
                self.committing.set(true);
                let this = self.clone();
                let epoch = self.epoch.get();
                set_timeout(
                    move || {
                        if this.alive.get() && this.epoch.get() == epoch && !this.reload.get() {
                            this.committing.set(false);
                        }
                    },
                    Duration::from_secs(10),
                );
            }
            let reply = packet("SUDOKU_UPDATE_ACK", &target);
            let _ = js_sys::Reflect::set(&reply, &"ready".into(), &ready.into());
            if let Some(port) = event.ports().get(0).dyn_ref::<web_sys::MessagePort>() {
                let _ = port.post_message(&reply);
            }
        }
        fn changed(self: &Rc<Self>) {
            if self.reload.get() {
                return;
            }
            if !self.ready.get_untracked() {
                self.inspect();
                return;
            }
            let Some(controller) = self
                .registration
                .borrow()
                .as_ref()
                .and_then(|r| r.active())
                .or_else(|| {
                    web_sys::window().and_then(|w| w.navigator().service_worker().controller())
                })
            else {
                return;
            };
            let this = self.clone();
            spawn_local(async move {
                let target = version(&controller).await;
                if this.alive.get()
                    && this.ready.get_untracked()
                    && target == this.target.get_untracked()
                    && !this.reload.replace(true)
                {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().reload();
                    }
                }
            });
        }
        fn start(self: &Rc<Self>) {
            let Some(window) = web_sys::window() else {
                return;
            };
            let native = js_sys::Reflect::get(window.as_ref(), &"__SUDOKU_NATIVE__".into())
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if native
                || !js_sys::Reflect::has(window.navigator().as_ref(), &"serviceWorker".into())
                    .unwrap_or(false)
            {
                return;
            }
            self.enabled.set(true);
            let container = window.navigator().service_worker();
            let this = self.clone();
            self.bind(container.clone().into(), "message", move |e| {
                if let Ok(e) = e.dyn_into::<MessageEvent>() {
                    this.receive(e);
                }
            });
            let this = self.clone();
            self.bind(container.clone().into(), "controllerchange", move |_| {
                this.changed()
            });
            if let Some(document) = window.document() {
                let this = self.clone();
                self.bind(document.into(), "visibilitychange", move |_| {
                    if web_sys::window()
                        .and_then(|w| w.document())
                        .is_some_and(|d| !d.hidden())
                    {
                        this.check(true);
                    }
                });
            }
            let this = self.clone();
            self.bind(window.into(), "online", move |_| this.check(false));
            let this = self.clone();
            spawn_local(async move {
                let promise = container.get_registration();
                let Ok(value) = JsFuture::from(promise).await else {
                    return;
                };
                let Ok(registration) = value.dyn_into::<ServiceWorkerRegistration>() else {
                    return;
                };
                if !this.alive.get() {
                    return;
                }
                *this.registration.borrow_mut() = Some(registration.clone());
                let watch = this.clone();
                this.bind(registration.into(), "updatefound", move |_| {
                    let Some(worker) = watch
                        .registration
                        .borrow()
                        .as_ref()
                        .and_then(|r| r.installing())
                    else {
                        return;
                    };
                    let notify = watch.clone();
                    watch.bind(worker.into(), "statechange", move |_| notify.inspect());
                });
                this.check(false);
            });
        }
    }
    #[derive(Clone, Copy)]
    pub(crate) struct UpdateController {
        handle: StoredValue<Rc<Runtime>, LocalStorage>,
        pub target: RwSignal<Option<String>>,
        pub dismissed: RwSignal<bool>,
        pub preparing: RwSignal<bool>,
        pub ready: RwSignal<bool>,
        pub committing: RwSignal<bool>,
        pub failed: RwSignal<bool>,
    }
    impl UpdateController {
        pub fn accept(self) {
            self.handle.get_value().accept();
        }
        pub fn cancel(self) {
            self.handle.get_value().cancel();
        }
        pub fn clear_notice(self) {
            self.handle.get_value().notice_observer.borrow_mut().take();
            reserve_notice(0.0);
        }
        pub fn watch_notice(self, element: &web_sys::HtmlElement) {
            reserve_notice(element.get_bounding_client_rect().height() + 16.0);
            let callback = Closure::wrap(Box::new(
                move |entries: js_sys::Array, _: web_sys::ResizeObserver| {
                    if let Some(entry) = entries.get(0).dyn_ref::<web_sys::ResizeObserverEntry>() {
                        reserve_notice(entry.target().get_bounding_client_rect().height() + 16.0);
                    }
                },
            )
                as Box<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>);
            if let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) {
                observer.observe(element);
                *self.handle.get_value().notice_observer.borrow_mut() = Some(NoticeObserver {
                    observer,
                    _callback: callback,
                });
            }
        }
    }
    pub(crate) fn controller(state: AppState, history: GameHistory) -> UpdateController {
        let runtime = Rc::new(Runtime {
            state,
            history,
            target: RwSignal::new(None),
            dismissed: RwSignal::new(false),
            preparing: RwSignal::new(false),
            ready: RwSignal::new(false),
            committing: RwSignal::new(false),
            failed: RwSignal::new(false),
            original: RefCell::new(None),
            worker: RefCell::new(None),
            registration: RefCell::new(None),
            bindings: RefCell::new(vec![]),
            notice_observer: RefCell::new(None),
            last_check: Cell::new(0.0),
            checking: Cell::new(false),
            alive: Cell::new(true),
            enabled: Cell::new(false),
            reload: Cell::new(false),
            epoch: Cell::new(0),
        });
        runtime.start();
        let tick = runtime.clone();
        let interval = set_interval_with_handle(
            move || {
                tick.inspect();
                tick.attempt();
                tick.cleanup_cache();
            },
            Duration::from_secs(30),
        )
        .ok();
        let cleanup = StoredValue::new_local(runtime.clone());
        on_cleanup(move || {
            let cleanup = cleanup.get_value();
            cleanup.alive.set(false);
            cleanup.bindings.borrow_mut().clear();
            cleanup.notice_observer.borrow_mut().take();
            reserve_notice(0.0);
            if let Some(interval) = interval {
                interval.clear();
            }
        });
        UpdateController {
            handle: cleanup,
            target: runtime.target,
            dismissed: runtime.dismissed,
            preparing: runtime.preparing,
            ready: runtime.ready,
            committing: runtime.committing,
            failed: runtime.failed,
        }
    }
}
#[cfg(target_arch = "wasm32")]
pub(crate) use browser::controller;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reload_snapshot_is_version_bound_and_preserves_progress() {
        let mut game = GameState::default();
        game.paused = true;
        game.notes[0] = 5;
        game.cell_colors[1] = 2;
        game.timer_seconds = 73;
        game.history.push(crate::state::Snapshot {
            board: game.board,
            notes: game.notes,
            cell_colors: game.cell_colors,
        });
        game.redo_stack = game.history.clone();
        let json = serde_json::to_string(&Resume {
            target: "bundle-b".into(),
            game: game.clone(),
        })
        .unwrap();
        assert!(decode_resume(&json, "bundle-a").is_none());
        assert!(decode_resume(&json, "").is_none());
        assert!(decode_resume("bad", "bundle-b").is_none());
        let resumed = decode_resume(&json, "bundle-b").unwrap();
        assert_eq!(resumed.board, game.board);
        assert_eq!(resumed.notes, game.notes);
        assert_eq!(resumed.cell_colors, game.cell_colors);
        assert_eq!(resumed.history.len(), 1);
        assert_eq!(resumed.redo_stack.len(), 1);
        assert!(resumed.paused);
        assert_eq!(resumed.timer_seconds, 73);
        assert_eq!(resumed.effects, game.effects);
    }
}
