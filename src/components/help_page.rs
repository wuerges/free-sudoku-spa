use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn HelpPage() -> impl IntoView {
    let navigate = use_navigate();
    let items: Vec<(&str, &str, &str)> = vec![
        (
            "1 a 9",
            "Números",
            "No modo normal, coloca o número na célula. No modo notas, alterna o lápis.",
        ),
        (
            "⌫ Apagar",
            "Apagar",
            "Remove o número ou as notas da célula selecionada.",
        ),
        (
            "📝 Nota",
            "Modo notas",
            "Ativa o modo de lápis para marcar candidatos em vez de preencher.",
        ),
        ("⏸/▶", "Pausar", "Pausa ou retoma o cronômetro."),
        (
            "🔄 Novo Jogo",
            "Novo jogo",
            "Inicia um novo jogo. Escolha entre Fácil, Médio, Difícil, Expert ou Mestre.",
        ),
        ("↩ Desfazer", "Desfazer", "Desfaz a última ação."),
        ("↪ Refazer", "Refazer", "Refaz a ação desfeita."),
        (
            "📝 Auto Notas",
            "Auto notas",
            "Preenche automaticamente os lápis válidos em todas as células vazias.",
        ),
        (
            "💡 Dica",
            "Dica",
            "Revela a resposta correta de uma célula.",
        ),
        ("Resolver", "Resolver", "Completa o jogo com a solução."),
        ("☀️/🌙", "Modo escuro", "Alterna entre modo claro e escuro."),
        (
            "📲 Instalar",
            "Instalar",
            "Instala o app no dispositivo como PWA, para uso offline.",
        ),
    ];

    view! {
        <div class="w-full max-w-[min(90vw,500px)] mx-auto pb-8">
            <button
                class="flex items-center gap-0.5 text-blue-500 dark:text-blue-400 active:opacity-60 select-none py-2.5 px-1 -ml-1 rounded-lg transition-opacity"
                on:click=move |_| navigate("/", Default::default())
            >
                <span class="text-xl leading-none">"‹"</span>
                <span class="text-[17px] font-normal">"Voltar"</span>
            </button>

            <h1 class="text-3xl font-bold tracking-wider text-center mt-4 mb-10">"SUDOKU"</h1>

            <div class="flex flex-col">
                {items.iter().enumerate().map(|(i, (icon, name, desc))| {
                    let border = if i < items.len() - 1 {
                        "border-bottom: 0.5px solid #9ca3af;"
                    } else {
                        ""
                    };
                    view! {
                        <div class="grid gap-2" style=format!("grid-template-columns: auto 1fr; align-items: start; padding: 1.25rem 0; {border}")>
                            <span class="text-lg text-center" style="width: 80px; line-height: 1.25;">{icon.to_string()}</span>
                            <div>
                                <strong class="text-sm font-semibold">{name.to_string()}</strong>
                                <p class="text-xs text-gray-500 dark:text-gray-400" style="margin-top: 2px; line-height: 1.5;">{desc.to_string()}</p>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="text-center mt-10">
                <p class="text-sm text-slate-400 dark:text-slate-500 max-w-[280px] mx-auto leading-relaxed">
                    "Sudoku gratuito, sem anúncios e sem rastreadores."
                </p>
                <p class="text-xs text-slate-500 dark:text-slate-600 mt-5">
                    "Encontrou um problema? "
                    <a
                        href="https://github.com/wuerges/free-sudoku-spa/issues"
                        target="_blank"
                        rel="noopener"
                        class="text-blue-400 dark:text-blue-400 underline"
                    >
                        "Abra uma issue no GitHub"
                    </a>
                </p>
            </div>

        </div>
    }
}
