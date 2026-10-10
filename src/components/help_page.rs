use crate::components::icon::{Icon, IconName};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HelpPage() -> impl IntoView {
    let items: Vec<(Option<IconName>, &str, &str)> = vec![
        (
            None,
            "Números",
            "No modo normal, coloca o número na célula. No modo notas, alterna o lápis.",
        ),
        (
            Some(IconName::Erase),
            "Apagar",
            "Remove o número ou as notas da célula selecionada.",
        ),
        (
            Some(IconName::Pencil),
            "Modo notas",
            "Ativa o modo de lápis para marcar candidatos em vez de preencher. O botão Notas fica destacado enquanto o modo está ativo.",
        ),
        (
            Some(IconName::Drop),
            "Modo drop",
            "Ative, clique num número para selecioná-lo, depois clique nas células para adicionar/remover notas ou colocar o número (dependendo se o modo notas está ligado).",
        ),
        (Some(IconName::Pause), "Pausar", "Pausa ou retoma o cronômetro."),
        (
            Some(IconName::Clock),
            "Contador de erros",
            "Mostra o total de erros ao lado do cronômetro. Aumenta a cada número errado e não retrocede com Desfazer.",
        ),
        (
            Some(IconName::Reset),
            "Novo jogo",
            "Inicia um novo jogo. A dificuldade vem das técnicas usadas para resolver: Fácil (candidatos únicos), Médio (candidatos bloqueados), Difícil (pares/trios), Expert (X-Wing/XY-Wing e cadeias curtas) e Mestre (cadeias alternadas). O cabeçalho mostra a técnica mais avançada. Jogos antigos mantêm o nível anterior.",
        ),
        (Some(IconName::Undo), "Desfazer", "Desfaz a última ação. O histórico é apagado ao usar uma dica."),
        (Some(IconName::Redo), "Refazer", "Refaz a ação desfeita. O histórico é apagado ao usar uma dica."),
        (
            Some(IconName::AutoNotes),
            "Auto notas",
            "Preenche automaticamente os lápis válidos em todas as células vazias.",
        ),
        (
            Some(IconName::Hint),
            "Dica",
            "Revela a célula com menos candidatos, limpa as notas relacionadas e apaga o histórico de desfazer. Células reveladas ficam amarelas. Disponível em todos os níveis quando ativada nas Configurações.",
        ),
        (
            Some(IconName::Domino),
            "Efeito Dominó",
            "Após um acerto normal ou no modo Drop, preenche células vazias com apenas um candidato pela linha, coluna e bloco 3×3. Configure o intervalo inicial, a aceleração, o intervalo mínimo e o limite de células vazias nas Configurações. Limite 0 significa sem limite; outros valores permitem o efeito quando restarem até essa quantidade. Ativado por padrão quando restarem até 10 células vazias.",
        ),
        (
            Some(IconName::Sound),
            "Som",
            "Ao acertar uma célula, toca um som. Escolha Desligado, Bip ou Explosão na seção Som das Configurações.",
        ),
        (Some(IconName::Moon), "Modo escuro", "Alterna entre modo claro e escuro."),
        (
            Some(IconName::Install),
            "Instalar",
            "Instala o app no dispositivo como PWA, para uso offline.",
        ),
    ];

    view! {
        <main class="settings-page">
            <A href="/" attr:class="ui-back"><Icon name=IconName::Back /><span>"Voltar"</span></A>
            <h1 class="page-title">"Como jogar"</h1>
            <p class="page-description">"Conheça os controles e as assistências do Sudoku."</p>
            <div class="settings-section">
                {items.into_iter().map(|(icon, name, desc)| {
                    view! {
                        <div class="help-item">
                            <span aria-hidden="true">{match icon {
                                Some(name) => view! { <Icon name=name /> }.into_any(),
                                None => view! { <span class="font-bold">"1–9"</span> }.into_any(),
                            }}</span>
                            <div>
                                <h2 class="preference-label">{name}</h2>
                                <p class="section-description">{desc}</p>
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

        </main>
    }
}
