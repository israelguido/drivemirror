# Segurança

## Como reportar uma vulnerabilidade

Não abra uma issue pública. Use o canal privado do GitHub:

**<https://github.com/israelguido/drivemirror/security/advisories/new>**

Inclua a versão do DriveMirror, a distribuição, os passos para reproduzir e o
impacto esperado. **Não envie Client Secrets, tokens nem o seu `rclone.conf`**;
se o problema envolver credenciais, use valores falsos e, se algum valor real
tiver vazado, revogue-o no Google Cloud Console.

Este é um projeto mantido por uma pessoa, sem prazo de resposta garantido. As
correções saem em novas versões publicadas em [`releases/`](releases/).

## Escopo

Dentro do escopo:

- o app DriveMirror (código em `src/`) e os pacotes publicados em `releases/`;
- vazamento de credenciais (Client Secret, token) em arquivos, mensagens,
  saída de processos ou no `config.toml`;
- argumentos passados ao rclone que possam ser interpretados como opções ou
  apontar para um destino diferente do configurado;
- sincronização que apague ou envie dados fora da pasta local e da subpasta do
  Drive escolhidas.

Fora do escopo: vulnerabilidades do próprio rclone, do Google Drive, do GTK ou
da distribuição (reporte a esses projetos), e o comportamento documentado de
espelhamento, em que apagar um arquivo localmente o apaga no Drive.

## O que o app faz com credenciais

- O Client ID e o Client Secret digitados na aba **Contas rclone** são passados
  ao `rclone config create`/`config update` como argumentos de processo, sem
  shell. O rclone grava as credenciais e o token no próprio arquivo,
  `~/.config/rclone/rclone.conf`.
- O DriveMirror não grava o secret nem o token em disco e não gera logs. O
  campo do secret é limpo quando o rclone começa, e o secret fica na memória
  do app só enquanto a criação da conta está em andamento.
- Toda saída do rclone exibida no app passa por uma redação que troca o secret
  por `<redacted>`.
- Para marcar remotos incompletos, o app lê `rclone config dump` e verifica só
  se existe um token não vazio. O conteúdo do dump é descartado na hora e
  nunca é exibido.
- Limite conhecido: enquanto o rclone roda durante a criação da conta, os
  argumentos (incluindo o secret) ficam visíveis em `/proc` para processos do
  mesmo usuário.
