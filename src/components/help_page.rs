use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HelpPage() -> impl IntoView {
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
        (
            "🎯 Drop",
            "Modo drop",
            "Ative, clique num número para selecioná-lo, depois clique nas células para adicionar/remover notas ou colocar o número (dependendo se o modo notas está ligado).",
        ),
        ("⏸/▶", "Pausar", "Pausa ou retoma o cronômetro."),
        (
            "✅/❌",
            "Contador de erros",
            "Mostra ✅ 0 quando não há erros. Incrementa ❌ a cada número errado. Não retrocede com Desfazer.",
        ),
        (
            "🔄 Novo Jogo",
            "Novo jogo",
            "Inicia um novo jogo. A dificuldade vem das técnicas usadas para resolver: Fácil (candidatos únicos), Médio (candidatos bloqueados), Difícil (pares/trios), Expert (X-Wing/XY-Wing e cadeias curtas) e Mestre (cadeias alternadas). O cabeçalho mostra a técnica mais avançada. Jogos antigos mantêm o nível anterior.",
        ),
        ("↩ Desfazer", "Desfazer", "Desfaz a última ação. O histórico é apagado ao usar uma dica."),
        ("↪ Refazer", "Refazer", "Refaz a ação desfeita. O histórico é apagado ao usar uma dica."),
        (
            "📝 Auto Notas",
            "Auto notas",
            "Preenche automaticamente os lápis válidos em todas as células vazias.",
        ),
        (
            "💡 Dica",
            "Dica",
            "Revela a célula com menos candidatos, limpa as notas relacionadas e apaga o histórico de desfazer. Células reveladas ficam amarelas. Disponível em todos os níveis quando ativada nas Configurações.",
        ),
        (
            "🀄 Efeito Dominó",
            "Efeito Dominó",
            "Após um acerto normal ou no modo Drop, preenche células vazias com apenas um candidato pela linha, coluna e bloco 3×3. Configure o intervalo inicial, a aceleração, o intervalo mínimo e o limite de células vazias nas Configurações. Limite 0 significa sem limite; outros valores permitem o efeito quando restarem até essa quantidade. Ativado por padrão quando restarem até 10 células vazias.",
        ),
        (
            "🔊 Som",
            "Som",
            "Ao acertar uma célula, toca um som. Pode ser um bip, uma pequena explosão, ou desligado. Configurável na página de Configurações.",
        ),
        ("☀️/🌙", "Modo escuro", "Alterna entre modo claro e escuro."),
        (
            "📲 Instalar",
            "Instalar",
            "Instala o app no dispositivo como PWA, para uso offline.",
        ),
    ];

    view! {
        <div class="w-full max-w-[min(90vw,500px)] mx-auto pb-8">
            <A
                href="/"
                attr:class="flex items-center gap-0.5 text-accent active:bg-control select-none py-2.5 px-1 -ml-1 rounded-lg transition-opacity no-underline"
            >
                <span class="text-xl leading-none">"‹"</span>
                <span class="text-[17px] font-normal">"Voltar"</span>
            </A>

            <h1 class="text-3xl font-bold tracking-wider text-center mt-4 mb-10">"SUDOKU"</h1>

            <div class="flex flex-col">
                {items.iter().enumerate().map(|(i, (icon, name, desc))| {
                    let border = if i < items.len() - 1 {
                        "border-bottom: 0.5px solid var(--ui-grid-thin);"
                    } else {
                        ""
                    };
                    view! {
                        <div class="grid gap-2" style=format!("grid-template-columns: auto 1fr; align-items: start; padding: 1.25rem 0; {border}")>
                            <span class="text-lg text-center" style="width: 80px; line-height: 1.25;">{icon.to_string()}</span>
                            <div>
                                <strong class="text-sm font-semibold">{name.to_string()}</strong>
                                <p class="text-xs text-muted" style="margin-top: 2px; line-height: 1.5;">{desc.to_string()}</p>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="text-center mt-10">
                <p class="text-sm text-muted max-w-[280px] mx-auto leading-relaxed">
                    "Sudoku gratuito, sem anúncios e sem rastreadores."
                </p>
                <p class="text-xs text-muted mt-5">
                    "Encontrou um problema? "
                    <a
                        href="https://github.com/wuerges/free-sudoku-spa/issues"
                        target="_blank"
                        rel="noopener"
                        class="text-accent underline"
                    >
                        "Abra uma issue no GitHub"
                    </a>
                </p>
            </div>

        </div>
    }
}
