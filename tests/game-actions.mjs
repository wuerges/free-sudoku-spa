// Shared browser interaction for the collapsible game actions panel.
export async function openGameActions(page) {
  const trigger = page.getByRole('button', {name:'Ações', exact:true});
  if (await trigger.getAttribute('aria-expanded') !== 'true') await trigger.click();
}

export async function clickGameAction(page, name) {
  if (name === 'Desfazer' || name === 'Refazer') {
    await page.locator('.entry-actions').getByRole('button', {name, exact:true}).click();
    return;
  }
  await openGameActions(page);
  await page.getByRole('button', {name, exact:true}).click();
  // Keep the difficulty choices open; other actions return to the board.
  if (name !== 'Novo jogo' && await page.getByRole('button', {name:'Fechar ações', exact:true}).isVisible()) {
    await page.getByRole('button', {name:'Fechar ações', exact:true}).click();
  }
}
