use crate::sharing::SharedGame;
use leptos::prelude::*;

#[component]
pub fn BoardPreview(game: SharedGame, #[prop(default = false)] miniature: bool) -> impl IntoView {
    view! {
        <div class=if miniature {"share-preview history-miniature"} else {"share-preview"}
            role="img" aria-label="Prévia do Sudoku">
            {game.board.into_iter().enumerate().map(|(i,n)|view! {
                <span class="share-preview-cell" data-color=game.colors[i]
                    data-given=(game.givens[i]!=0).to_string()
                    data-error=(n!=0 && n!=game.solution[i]).to_string()
                    data-hint=(game.hinted[i]!=0).to_string()
                    style=format!("border-right-width:{}px;border-bottom-width:{}px",if i%9==2 || i%9==5 {3}else{1},if i/9==2 || i/9==5 {3}else{1})>
                    {if n==0 {String::new()}else{n.to_string()}}
                    <small>{if n==0 {(1..=9).filter(|v| game.notes[i] & (1<<(v-1))!=0).map(|v| v.to_string()).collect::<Vec<_>>().join(" ")}else{String::new()}}</small>
                </span>
            }).collect::<Vec<_>>()}
        </div>
    }
}
