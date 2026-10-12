use crate::state::{AppState, GameHistory};
use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
#[component]
pub fn UpdateBanner(state: AppState, history: GameHistory) -> impl IntoView {
    use crate::i18n::{use_i18n, Message};
    use web_sys::Event;
    let i18n = use_i18n();
    let controller = crate::updates::controller(state, history);
    let target = controller.target;
    let dismissed = controller.dismissed;
    let preparing = controller.preparing;
    let ready = controller.ready;
    let committing = controller.committing;
    let failed = controller.failed;
    let notice = NodeRef::<leptos::html::Aside>::new();
    Effect::new(move || {
        let visible =
            target.get().is_some() && !dismissed.get() && !preparing.get() && !ready.get();
        i18n.locale.get();
        failed.get();
        controller.clear_notice();
        if visible {
            if let Some(element) = notice.get() {
                controller.watch_notice(&element);
            }
        }
    });
    let dialog = NodeRef::<leptos::html::Dialog>::new();
    Effect::new(move || {
        if let Some(element) = dialog.get() {
            if preparing.get() || ready.get() {
                if !element.open() {
                    let _ = element.show_modal();
                }
            } else {
                element.close();
            }
        }
    });
    view! {
        <Show when=move || target.get().is_some() && !dismissed.get() && !preparing.get() && !ready.get()>
            <aside node_ref=notice class="update-banner" role="status" aria-live="polite">
                <p>{move || i18n.text(Message::UpdateAvailable)}</p>
                <Show when=move || failed.get()><p role="alert">{move || i18n.text(Message::UpdateSaveFailed)}</p></Show>
                <div class="update-actions">
                    <button id="update-now" class="ui-action" on:click=move |_|controller.accept()>{move || i18n.text(if failed.get(){Message::TryAgain}else{Message::UpdateNow})}</button>
                    <button id="update-later" class="ui-action" on:click=move |_|{dismissed.set(true);failed.set(false);}>{move || i18n.text(Message::UpdateLater)}</button>
                </div>
            </aside>
        </Show>
        <dialog id="update-waiting" node_ref=dialog class="share-dialog update-dialog" aria-labelledby="update-heading" on:cancel=move |event:Event|event.prevent_default()>
            <h2 id="update-heading">{move || i18n.text(if preparing.get(){Message::UpdateSaving}else if committing.get(){Message::UpdateApplying}else{Message::UpdateWaiting})}</h2>
            <p>{move || i18n.text(Message::UpdateWaitingIntro)}</p>
            <button id="update-cancel" class="ui-action" disabled=move || preparing.get() || committing.get() on:click=move |_|controller.cancel()>{move || i18n.text(Message::UpdateCancel)}</button>
        </dialog>
    }
}
#[cfg(not(target_arch = "wasm32"))]
#[component]
pub fn UpdateBanner(state: AppState, history: GameHistory) -> impl IntoView {
    let _ = (state, history);
}
