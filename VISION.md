# Visão do Projeto: Sudoku PWA com Leptos (Rust + WASM)

**Nome do projeto:** `free-sudoku-pwa` 
**Tipo:** Single Page Application (SPA) Progressive Web App (PWA)  
**Plataforma principal:** Tablets e smartphones Android (instalável como app nativo) + Web moderna  
**Deploy:** Vercel (estático, zero custo)  
**Status desejado:** MVP funcional em 4-5 semanas (dev experiente)

---

## 1. Declaração de Visão

Criar um jogo de Sudoku moderno, leve, bonito e **100% funcional offline**, construído **inteiramente em Rust** usando o framework **Leptos**, compilado para WebAssembly (WASM). 

O app deve ser uma Progressive Web App instalável no Android (Chrome/Edge) como se fosse um aplicativo normal — com ícone na tela inicial, abertura em janela standalone, splash e funcionamento offline completo após o primeiro carregamento.

O diferencial técnico: geração **client-side** de puzzles de Sudoku com 5 níveis de dificuldade reais (Fácil → Mestre), usando lógica Rust pura de alta performance. Zero backend, zero dependência de servidor, deploy estático simples no Vercel.

O objetivo é entregar uma experiência premium de puzzle mobile-first, com código limpo, performático e mantível.

---

## 2. Objetivos de Sucesso

| Objetivo | Métrica de Sucesso |
|----------|---------------------|
| Experiência App-like no Android | Instalável via prompt nativo, abre em standalone, ícone bonito, funciona offline |
| Geração de puzzles de qualidade | 5 dificuldades distintas e consistentes; todo puzzle tem **exatamente 1 solução** |
| Performance | Geração de puzzle < 150ms (mesmo Expert); UI 60fps; WASM < 400KB gzipped |
| UX Mobile | Touch targets ≥ 48px, sem lag, teclado virtual grande, modo notas fluido |
| Código | Lógica Sudoku 100% testável em Rust (cargo test); UI reativa com Leptos signals |
| Deploy | Um comando ou GitHub Action → produção no Vercel |

---

## 3. Funcionalidades do MVP

### Core Gameplay
- Grid 9×9 responsivo e touch-friendly
- 5 dificuldades: **Fácil | Médio | Difícil | Expert | Mestre**
- Botão "Novo Jogo" que revela seleção de dificuldade (Fácil | Médio | Difícil | Expert | Mestre) e gera puzzle novo
- Input via **number pad virtual** (botões 1-9 em linha horizontal + Apagar + alternar Nota)
- **Restrição de design:** Zero elementos `<input>` na UI do jogo — usar apenas `<button>` e `<div>` para evitar que o teclado virtual do OS interfira com o number pad customizado
- **Modo Notas** (pencil marks): múltiplos candidatos pequenos por célula
- Validação em tempo real com destaque visual de conflitos (mesmo número na linha/coluna/bloco)
- **Destaque visual da linha e coluna** da célula selecionada
- **Undo / Redo** com histórico de estados (histórico limpo ao usar dica)
- Timer da partida (com botão pausar)
- Botão **Dica** (revela célula com menos candidatos, limpa notas relacionadas, apaga histórico undo/redo, célula fica amarela; desabilitada no Mestre)
- Detecção de vitória + feedback visual (animação simples / confetti leve via CSS ou canvas)
- Atalhos de teclado completos (números, Backspace, U para undo, etc.)

### PWA & Android Install
- `manifest.json` completo e válido
- Service Worker que cacheia o app shell (HTML + WASM + CSS + JS + ícones)
- Botão ou prompt "Instalar aplicativo" usando a API `beforeinstallprompt`
- Ícones de alta qualidade em múltiplos tamanhos (192×192, 512×512) + **maskable**
- `display: "standalone"`, `orientation: "portrait"` (direção inicial preferida; manifesto não trava rotação — para lock real usar `screen.orientation.lock("portrait")` no primeiro toque do usuário)
- Funcionamento 100% offline após primeiro load (geração de puzzles é local)

### Outros MVP
- Tema claro/escuro automático (system preference) + toggle manual
- Persistência de jogo em andamento no `localStorage` (recupera ao recarregar a página)
- Design limpo, minimalista e moderno (Tailwind)
- Totalmente responsivo (mobile-first, excelente no Android, bom no desktop)

---

## 4. Funcionalidades Futuras (v2+)

- Estatísticas locais (tempos médios, streak, puzzles resolvidos por dificuldade)
- Dicas inteligentes baseadas em técnicas ("Naked Single nesta célula")
- Daily Puzzle (seed baseado na data)
- Exportar/importar puzzle (string padrão)
- Compartilhamento de resultado (texto + grid)

---

## 5. Stack Tecnológico

| Camada              | Tecnologia                              | Justificativa |
|---------------------|-----------------------------------------|-------------|
| Linguagem           | Rust (stable)                           | Segurança, performance, WASM nativo |
| UI Framework        | **Leptos** (CSR mode)                   | Reatividade granular excelente, signals, components, WASM-first |
| Build               | **Trunk** + Tailwind CLI standalone     | Trunk para WASM; Tailwind CLI standalone (v4 não usa mais `tailwind.config.js` — usa `@import "tailwindcss"` no CSS) |
| Estilo              | Tailwind CSS (v4+)                      | Rápido, responsivo, dark mode nativo. Migrar exemplos Leptos oficiais para v4 (ver leptos-rs/leptos#3688) |
| Lógica Sudoku       | Módulo Rust puro (`sudoku_engine`)      | Testável, sem_std friendly, zero JS |
| Sudoku Crate (opcional) | `number-place-rs` (crates.io) ou custom | `number-place-rs` v0.3 tem generator + solver + difficulty grading por técnicas humanas (único crate Rust com grading real). Custom engine como fallback |
| WASM bindings       | `wasm-bindgen` + `web-sys` + `console_error_panic_hook` | Acesso a DOM, localStorage, install prompt, Canvas; panic hook para debugging |
| PWA                 | Web App Manifest + Service Worker       | Instalável no Android, offline, com estratégia de atualização |
| Deploy              | **Vercel** (static) + GitHub Actions    | CDN edge, preview deploys, zero custo |
| Testes              | `cargo test` (engine + state) + manual / Playwright futuro | Engine e state management 100% testados |

**Nota sobre Tailwind v4 + Trunk:** Tailwind v4 usa `@import "tailwindcss"` em vez de `tailwind.config.js`. Usar o Tailwind CLI standalone (binary, sem Node.js) junto com Trunk — o suporte built-in de Tailwind no Trunk é para v3.

---

## 6. Arquitetura e Estrutura de Pastas (Recomendada)

```
free-sudoku-pwa/
├── Cargo.toml
├── Trunk.toml                 # config de build
├── index.html                 # template principal (veja modelo abaixo)
├── .gitignore                 # target/, dist/
├── vercel.json                # (opcional) headers, cache
├── public/                    # assets estáticos — copiados via <link data-trunk rel="copy-dir" href="public" />
│   ├── manifest.json
│   ├── sw.js                  # service worker
│   └── icons/
│       ├── icon-192.png
│       ├── icon-512.png
│       └── icon-maskable-512.png
├── style/
│   └── input.css              # @import "tailwindcss"; (processado pelo Tailwind CLI standalone)
├── src/
│   ├── main.rs                 # wasm entry point (console_error_panic_hook::set_once + mount_to_body)
│   ├── app.rs                 # componente raiz Leptos
│   ├── sudoku_engine.rs       # ★ CORE: Board, generate, solve, is_valid, count_solutions
│   ├── state.rs               # sinais globais (board, notes, history, timer, difficulty)
│   ├── components/
│   │   ├── sudoku_grid.rs
│   │   ├── cell.rs
│   │   ├── number_pad.rs
│   │   ├── game_controls.rs
│   │   └── header.rs
│   └── utils.rs               # local_storage, format_time, clipboard, etc.
└── README.md
```

**`index.html` mínimo:**
```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <link rel="manifest" href="/manifest.json" />
  <link data-trunk rel="copy-dir" href="public" />
  <link data-trunk rel="css" href="style/output.css" />
  <link data-trunk rel="rust" href="Cargo.toml" />
</head>
<body>
  <div id="loading">Carregando Sudoku...</div>
  <script>
    if ('serviceWorker' in navigator) {
      navigator.serviceWorker.register('/sw.js');
    }
  </script>
</body>
</html>
```
> Trunk substitui `<link data-trunk rel="rust">` pelo JS/WASM bundle. O `<div id="loading">` é removido pelo Leptos ao montar. O SW é registrado via script inline (antes do WASM carregar).

**`Cargo.toml` mínimo:**
```toml
[package]
name = "free-sudoku-pwa"
version = "0.1.0"
edition = "2024"

[dependencies]
leptos = { version = "0.7", features = ["csr"] }
console_error_panic_hook = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
  "Window", "Document", "Element", "HtmlElement",
  "Storage", "console",
  "KeyboardEvent",
  "BeforeInstallPromptEvent",
  "ServiceWorkerContainer", "ServiceWorkerRegistration",
] }
```
> Features do `web-sys` são o mínimo necessário. Adicionar `CanvasRenderingContext2d` se usar Canvas para confetti. Medir tamanho com `cargo bloat` após cada feature adicionada.

---

## 7. Geração de Puzzles e Dificuldades

### Enum
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
    Master,
}
```

### Abordagem Recomendada (Iterativa)

**Fase 1 (MVP - Rápida):**
- Gerar grid completo válido com backtracking otimizado.
- Remover células com **simetria rotacional 180°** (recomendado como padrão — puzzles ficam visualmente mais profissionais). Algoritmo:
  1. Embaralhar a ordem das 81 posições
  2. Para cada posição (e sua simétrica), remover o par de células
  3. Rodar o counter-solver (early exit ao encontrar 2+ soluções)
  4. Se ainda houver exatamente 1 solução, manter as células removidas; senão, restaurá-las
  5. Repetir até que remover mais células quebraria a unicidade
- Usar contagem de clues remanescentes como proxy inicial de dificuldade:
  - Easy: ~40-45 clues
  - Medium: ~32-38 clues
  - Hard: ~26-31 clues
  - Expert: ~20-25 clues
  - Master: ~17-19 clues (mínimo teórico para solução única)

**Fase 2 (Qualidade):**
- Implementar ou integrar analisador de técnicas humanas:
  - Naked/Hidden Singles
  - Naked/Hidden Pairs & Triples
  - Pointing/Claiming
  - X-Wing, Swordfish, etc.
- Medir o nível máximo de técnica necessário para resolver sem guessing.
- Usar isso para classificar com precisão.

**Alternativa acelerada:** O crate [`number-place-rs`](https://crates.io/crates/number-place-rs) (v0.3, 2026) já implementa generator + solver + difficulty grading por técnicas humanas (Beginner a Expert). Pode substituir o engine customizado na Fase 1 se o foco for velocidade de entrega.

> ⚠️ **Atenção:** A compatibilidade WASM do `number-place-rs` não está documentada. É um crate de computação pura (sem I/O) — *deve* compilar para `wasm32-unknown-unknown`, mas não há exemplos públicos. **Testar na Fase 0** antes de comprometer. Se falhar, o engine customizado é o fallback.

**Requisito não-negociável:** Todo puzzle gerado deve ter **exatamente uma solução**. O solver deve contar soluções com early-exit ao encontrar 2+.

---

## 8. Experiência PWA no Android

Requisitos mínimos para instalação nativa:

1. Servido via **HTTPS** (Vercel entrega automaticamente)
2. `manifest.json` válido com:
   - `name`, `short_name`, `start_url`
   - `display: "standalone"`
   - `icons` (192 e 512 PNG + maskable recomendado)
   - `theme_color`, `background_color`
3. Service Worker registrado e funcionando (cache do app shell)
4. engagement signals (Vercel + visitas repetidas ajudam)

No código:
- Detectar `beforeinstallprompt` event
- Expor botão "Instalar Sudoku" que chama `prompt()`
- Após instalação: app abre sem chrome UI, ícone aparece na gaveta de apps

**Service Worker** com estratégia de cache e atualização:
- Cachear todos os assets estáticos no evento `install` (HTML + WASM + CSS + JS + ícones)
- Estratégia **Cache First** para o WASM/JS (assets versionados pelo Trunk com hash no filename)
- **Ciclo de atualização obrigatório**:
  - `skipWaiting()` no `install` para ativar o novo SW imediatamente
  - `clients.claim()` para controlar todas as páginas abertas
  - Detectar novo SW com `registration.onupdatefound` e exibir "Nova versão disponível — recarregar?"
  - Sem isso, deploys de bugfix nunca alcançam usuários com a versão cacheada

**Limitação conhecida:** Foco inicial em Android (Chrome/Edge). iOS Safari tem suporte a PWA mais limitado (sem `beforeinstallprompt`, instalação manual via "Add to Home Screen", storage quota de 50MB para SW cache). Testar no iOS Safari no final do MVP para documentar limitações conhecidas.

---

## 9. Deploy no Vercel

Usar **GitHub Actions** com cache de Rust + Trunk:

1. Workflow no push para `main`: instala Rust stable + Trunk + Tailwind CLI
2. `tailwindcss -i style/input.css -o style/output.css --minify`
3. `trunk build --release`
4. Deploy do `dist/` via `vercel deploy --prod` com token secreto

**vercel.json** para headers de cache:
```json
{
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        { "key": "X-Content-Type-Options", "value": "nosniff" },
        { "key": "Cache-Control", "value": "public, max-age=31536000, immutable" }
      ]
    },
    {
      "source": "/index.html",
      "headers": [
        { "key": "Cache-Control", "value": "public, max-age=0, must-revalidate" }
      ]
    }
  ]
}
```
> **Nota:** Assets versionados com hash pelo Trunk → `immutable`. `index.html` sem hash → `must-revalidate`. COOP/COEP não são necessários (app single-threaded, sem `SharedArrayBuffer`).

---

## 10. Considerações Técnicas Críticas

- **Tamanho WASM**: Usar `wasm-opt -Oz --strip-debug --strip-producers` + features seletivas do `web-sys`. Meta < 350KB gzipped. Medir com `cargo bloat`.
- **Panic hook**: Chamar `console_error_panic_hook::set_once()` na inicialização WASM para stack traces legíveis no console do browser durante debugging.
- **Loading state**: Incluir um `<div id="loading">Carregando Sudoku...</div>` no `index.html` que o Leptos remove ao montar. Em 3G lento, o WASM pode levar 2-5s para baixar — o usuário precisa ver que algo está acontecendo.
- **Offline detection**: Nenhum indicador necessário — o app funciona 100% offline por design.
- **Estado reativo**: Usar `RwSignal`, `Memo`, `create_effect` do Leptos com granularidade alta (evitar re-render de todo grid a cada tecla).
- **Leptos 0.7 gotcha — match arms**: Ramos de `match` dentro de `view!` com estruturas de elementos diferentes causam erro de tipo. Usar `.into_any()` em cada ramo para unificar os tipos. Exemplo: `match state { State::A => view! { <div>A</div> }.into_any(), State::B => view! { <div><span>B</span></div> }.into_any() }`. Isso será encontrado ao renderizar diferentes estados do jogo (jogando, pausado, vitória).
- **Representação de dados**: Board como `[u8; 81]` (0 = empty, 1-9 = filled). Notes como `[u16; 81]` (bitflags para candidatos 1-9). Ambas flat arrays para performance e serialização simples.
- **Acessibilidade**: Grid com roles ARIA: `role="grid"` no container, `role="row"` em cada linha, `role="gridcell"` em cada célula com `aria-label` indicando posição e valor. Contraste WCAG AA mínimo. Suporte completo a teclado com navegação por setas.
- **i18n**: Strings de UI em Português (pt-BR). Extrair para módulo `i18n.rs` desde o início.
- **Persistência**: Serializar estado completo (board + notes + timer + difficulty) no localStorage com `serde` + `serde_json`.
- **Validação**: Nunca confiar só na UI — o `sudoku_engine` é a fonte da verdade.

---

## 11. Roadmap de Implementação

| Fase | Entregáveis |
|------|-------------|
| **0. Setup** | Leptos CSR + Trunk + Tailwind CLI standalone, scaffold de componentes, manifest + SW com update lifecycle, loading indicator |
| **1. Engine Core** | `sudoku_engine.rs` completo (generate com simetria, solve, unique check com early exit, valid move). Testes unitários. Integração com signals Leptos |
| **2. Gameplay MVP** | Grid interativo (sem `<input>`), number pad, modo notas, conflitos visuais, undo/redo, timer com pause, dica, vitória |
| **3. Polish + PWA** | Tema dark/light, responsividade tablet + celular, persistência localStorage, otimização WASM, teste real em Android |
| **4. Deploy** | GitHub Actions + Vercel com cache de cargo, preview deploy, Lighthouse PWA audit ≥ 90 |

**Total estimado:** ~4-5 semanas (dev experiente, ritmo focado).

---

## 12. Decisões de Design & Princípios

- **Mobile-first & Touch-first**: Células grandes, number pad gigante, zero dependência de hover.
- **Separação de concerns forte**: Lógica de Sudoku nunca mistura com DOM/Leptos.
- **Simplicidade intencional**: MVP bonito e jogável > features demais.
- **Performance obsessiva**: WASM + Leptos signals são extremamente rápidos — manter assim.
- **Offline by default**: O app deve funcionar perfeitamente sem internet depois do primeiro load.
- **Rust como vantagem**: Usar o tipo system para impossibilitar estados inválidos onde possível.

---

## 13. Riscos & Mitigações

| Risco | Mitigação |
|-------|-----------|
| Complexidade do rating de dificuldade | Começar com proxy simples (clues + simetria) → evoluir para técnicas humanas na Fase 2 |
| Service Worker cache staleness (usuários presos em versão antiga) | Implementar `skipWaiting()` + `clients.claim()` + detecção de novo SW com prompt de "Nova versão disponível" |
| Tamanho do WASM | Medir cedo com `wasm-opt --strip-producers`, usar features mínimas do web-sys, evitar deps pesadas |
| WASM debugging difícil (stack traces ilegíveis) | `console_error_panic_hook` + source maps via Trunk DWARF; documentar workflow de debugging no README |
| Tailwind v4 breaking changes durante desenvolvimento | Pinar versão específica do Tailwind CLI; testar build de produção desde a Fase 0 |
| Build no Vercel lento | Usar GitHub Actions com cache de cargo + trunk |
| Experiência de install PWA inconsistente | Testar em Android real (múltiplas versões: 13, 14, 15) + Chrome/Edge; Lighthouse PWA audit em CI |
| iOS Safari PWA limitado (sem beforeinstallprompt, storage 50MB) | Foco inicial em Android; testar no iOS Safari no final da Fase 3 e documentar limitações conhecidas |
| Leptos learning curve | Aproveitar docs oficiais + exemplos CSR do book.leptos.dev + comunidade Discord |
| Leptos 0.7 match arm type mismatch no `view!` | Usar `.into_any()` em cada ramo do match; documentar no README |
| `number-place-rs` incompatível com WASM | Testar compilação para `wasm32-unknown-unknown` na Fase 0; fallback: engine customizado |
| Teclado virtual do OS interferindo com number pad customizado | Zero `<input>` elements na UI do jogo; usar `<button>` e `<div>` exclusivamente |

---

## 14. Requisitos de Hardware do Browser

O app assume os seguintes requisitos como **não-negociáveis**:

| Requisito | Justificativa |
|-----------|---------------|
| **WebAssembly (WASM)** | Toda a lógica do jogo (engine Sudoku + UI Leptos) compila para WASM. Sem WASM, o app não funciona. |
| **localStorage** | Persistência do jogo em andamento, preferências de tema, e futuras estatísticas dependem de `localStorage`. |

Browsers modernos (Chrome 90+, Edge 90+, Firefox 90+, Safari 15+) suportam ambos. O market share combinado de browsers sem WASM é < 1% — não há fallback.

> **Nota:** `localStorage` tem limite típico de 5-10MB por origem. O estado serializado do jogo cabe em < 2KB. Não há risco de estourar a quota.

---

## 15. Convenções de Código

- **Idioma:** Inglês para identificadores, comentários, mensagens de commit e documentação técnica. Strings de UI em Português (pt-BR) inicialmente, extraídas para um módulo `i18n.rs` simples.
- **Formatação:** `rustfmt` com configuração padrão. CI deve rodar `cargo fmt --check`.
- **Linting:** `cargo clippy` no CI (`-D warnings`). Sem warnings permitidos.
- **Testes:** `cargo test` no CI. Cobertura não precisa ser 100%, mas o `sudoku_engine` deve ter testes para todos os algoritmos públicos (generate, solve, is_valid, count_solutions).
- **Commits:** Convencionais (`feat:`, `fix:`, `chore:`, `docs:`). Mensagens em inglês.

---

## Referências

- Leptos Book (CSR): https://book.leptos.dev/
- Trunk: https://trunkrs.dev/
- `number-place-rs` crate: https://crates.io/crates/number-place-rs
- PWA fundamentals: https://web.dev/progressive-web-apps/

---

**Primeiros passos após aprovação:**

1. Criar repositório GitHub `free-sudoku-pwa`
2. Setup: `cargo init` + Trunk + Tailwind CLI standalone + estrutura de pastas
3. Implementar `sudoku_engine.rs` (coração do projeto)
4. Subir grid jogável + number pad → iterar a partir daí

Este documento é uma spec viva — atualize conforme decisões de implementação forem tomadas.

---

*Documento criado em Junho 2026 • 3ª revisão: fact-checked, desalucinado, validado contra docs oficiais Leptos/Trunk — pronto para implementação*
