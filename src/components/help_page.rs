use leptos::prelude::*;

#[component]
pub fn HelpPage(show_help: RwSignal<bool>) -> impl IntoView {
    let items: Vec<(&str, &str, &str)> = vec![
        ("☀️/🌙", "Modo escuro", "Alterna entre modo claro e escuro."),
        ("📲 Instalar", "Instalar app", "Instala o Sudoku no seu dispositivo como um aplicativo nativo (PWA)."),
        ("1 a 9", "Números", "No modo normal, coloca o número na célula selecionada. No modo notas, alterna o lápis do número na célula."),
        ("⌫ Apagar", "Apagar", "Remove o número ou as notas da célula selecionada."),
        ("📝 Nota", "Modo notas", "Ativa ou desativa o modo de notas (lápis). Quando ativo, os números marcam candidatos em vez de preencher a célula."),
        ("⏸/▶", "Pausar", "Pausa ou retoma o cronômetro do jogo."),
        ("🔄 Novo Jogo", "Novo jogo", "Inicia um novo jogo. É possível escolher a dificuldade: Fácil, Médio, Difícil ou Expert."),
        ("↩ Desfazer", "Desfazer", "Desfaz a última ação realizada."),
        ("↪ Refazer", "Refazer", "Refaz a ação que foi desfeita."),
        ("📝 Auto Notas", "Auto notas", "Preenche automaticamente todas as notas de lápis válidas em todas as células vazias, usando as regras do Sudoku."),
        ("💡 Dica", "Dica", "Revela a resposta correta de uma célula vazia."),
        ("Resolver", "Resolver", "Completa automaticamente todo o jogo com a solução correta."),
    ];

    view! {
        <div class="w-full max-w-[min(90vw,500px)] flex flex-col flex-1">
            // Back button
            <button
                class="self-start px-3 py-1 mb-2 rounded text-sm font-medium bg-gray-200 dark:bg-gray-700 active:opacity-70"
                on:click=move |_| show_help.set(false)
            >
                "← Voltar"
            </button>

            // Title
            <div class="text-center mb-4">
                <div class="text-4xl mb-2">{ "🧩" }</div>
                <h2 class="text-xl font-bold">"Sudoku"</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 max-w-xs mx-auto leading-relaxed">
                    "Implementação completa, sem anúncios, sem rastreadores. Código aberto no GitHub."
                </p>
                <a
                    href="https://github.com/wuerges/free-sudoku-spa/issues"
                    target="_blank"
                    rel="noopener"
                    class="text-xs text-blue-500 dark:text-blue-400 underline mt-1 inline-block"
                >
                    "Encontrou um problema? Abra uma issue."
                </a>
            </div>

            // Button list
            <div class="space-y-2">
                {items.into_iter().map(|(icon, name, desc)| view! {
                    <div class="flex items-start gap-3 p-2 rounded-lg bg-gray-50 dark:bg-gray-700/50">
                        <div class="text-xl w-10 text-center shrink-0">{icon.to_string()}</div>
                        <div class="min-w-0">
                            <div class="text-sm font-semibold">{name.to_string()}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">{desc.to_string()}</div>
                        </div>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
