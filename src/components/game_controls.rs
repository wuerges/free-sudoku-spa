use crate::state::AppState;
use crate::sudoku_engine::Difficulty;
use leptos::prelude::*;

fn format_time(seconds: u32) -> String {
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    format!("{m:02}:{s:02}")
}

#[component]
pub fn GameControls(state: AppState) -> impl IntoView {
    let timer = move || format_time(state.0.get().timer_seconds);
    let paused = move || state.0.get().paused;
    let won = move || state.0.get().won;
    let difficulty = move || state.0.get().difficulty;
    let show_new_game = RwSignal::new(false);

    view! {
        <div class="w-full max-w-[min(90vw,500px)] mx-auto mt-2 space-y-1.5">
            // Timer + errors + pause
            <div class="flex items-center justify-center gap-2 text-xl font-mono">
                <span>{move || timer()}</span>
                <span class="text-sm">
                    {move || {
                        let e = state.0.get().error_count;
                        if e == 0 {
                            "✅ 0".to_string()
                        } else {
                            format!("❌ {e}")
                        }
                    }}
                </span>
                <button
                    class="text-sm px-2 py-0.5 rounded bg-control active:bg-control-hover"
                    on:click=move |_| state.toggle_pause()
                >
                    {move || if paused() { "▶" } else { "⏸" }}
                </button>
            </div>

            // Win message
            <Show when=move || won()>
                <div class="text-center text-success-text font-bold text-lg animate-pulse">
                    "🎉 Parabéns! Puzzle resolvido!"
                </div>
            </Show>

            // New game button + difficulty selection
            <div class="flex flex-col items-center gap-1">
                <button
                    class="px-3 py-1 rounded text-sm font-medium bg-primary text-on-primary active:bg-primary-hover transition-colors"
                    on:click=move |_| show_new_game.update(|v| *v = !*v)
                >
                    "🔄 Novo Jogo"
                </button>
                <Show when=move || show_new_game.get()>
                    <div class="flex flex-wrap items-center justify-center gap-1">
                        {[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard, Difficulty::Expert, Difficulty::Master].iter().map(|&d| {
                            let label = match d {
                                Difficulty::Easy => "Fácil",
                                Difficulty::Medium => "Médio",
                                Difficulty::Hard => "Difícil",
                                Difficulty::Expert => "Expert",
                                Difficulty::Master => "Mestre",
                            };
                            view! {
                                <button
                                    class=move || format!(
                                        "px-2 py-1 rounded text-xs font-medium transition-colors {}",
                                        if difficulty() == d {
                                            "bg-primary text-on-primary"
                                        } else {
                                            "bg-control text-text active:bg-control-hover"
                                        }
                                    )
                                    on:click=move |_| {
                                        state.new_game(d);
                                        show_new_game.set(false);
                                    }
                                >
                                    {label}
                                </button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </Show>
            </div>

            // Action buttons
            <div class="flex justify-center gap-1.5">
                <Show when=move || state.0.get().undo_enabled>
                    <button
                        class="px-3 py-1.5 rounded text-sm font-medium bg-control active:bg-control-hover disabled:bg-disabled disabled:text-muted"
                        on:click=move |_| state.undo()
                        disabled=move || state.0.get().history.is_empty()
                    >
                        "↩ Desfazer"
                    </button>
                </Show>
                <Show when=move || state.0.get().undo_enabled>
                    <button
                        class="px-3 py-1.5 rounded text-sm font-medium bg-control active:bg-control-hover disabled:bg-disabled disabled:text-muted"
                        on:click=move |_| state.redo()
                        disabled=move || state.0.get().redo_stack.is_empty()
                    >
                        "↪ Refazer"
                    </button>
                </Show>
                <Show when=move || state.0.get().auto_notes_enabled>
                    <button
                        class="px-3 py-1.5 rounded text-sm font-medium bg-control active:bg-control-hover"
                        on:click=move |_| state.auto_notes()
                    >
                        "📝 Auto Notas"
                    </button>
                </Show>
                <Show when=move || state.0.get().hint_enabled>
                    <button
                        class="px-3 py-1.5 rounded text-sm font-medium bg-hint text-hint-text active:bg-hint disabled:bg-disabled disabled:text-muted"
                        on:click=move |_| state.hint()
                        disabled=move || difficulty() == Difficulty::Master
                    >
                        "💡 Dica"
                    </button>
                </Show>
            </div>
        </div>
    }
}
