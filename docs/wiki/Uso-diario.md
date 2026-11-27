# Uso diário

## Configurar o que sincronizar

Na aba **Sincronização** da janela completa:

| Campo | O que informar |
|---|---|
| **Pasta local** | A pasta do computador a espelhar. Use **Escolher…**. Precisa existir. |
| **Remoto rclone** | Só o nome do remoto, por exemplo `gdrive`, sem `:`. |
| **Pasta no Drive** | A subpasta de destino, por exemplo `Backup/Obsidian`. A raiz do Drive não é aceita, nem `.` ou `..`. |
| **Intervalo (min)** | De 1 a 1440 minutos (24 h). Padrão: 10. |
| **Idioma** | Sistema, Português (Brasil), English ou Español. Muda na hora. |

Clique em **Salvar configurações**, ou direto em **Sincronizar agora**, que
salva antes de rodar.

> **Atenção:** é um espelho. O que você apagar na pasta local é apagado na
> subpasta do Drive na próxima sincronização, e arquivos que existam só no
> destino são removidos. Use uma subpasta dedicada.

## Sincronizar

- **Manual:** **Sincronizar agora** na janela, no painel ou no menu do ícone.
- **Automática:** a cada intervalo, enquanto o app estiver aberto (na bandeja
  ou com a janela).
- Só uma sincronização roda por vez. O resultado aparece na linha de status da
  janela e no estado do painel.
- Se a sincronização automática não puder rodar (configuração inválida, remoto
  inexistente ou incompleto), o aviso aparece uma vez. Ela só tenta de novo
  quando a configuração ou a lista de remotos muda. A sincronização manual
  sempre tenta.

## Exclusões

Dois campos, um padrão por linha, no formato de filtro do rclone:

- **Arquivos ignorados**: padrão inicial `.DS_Store`, `Thumbs.db`,
  `desktop.ini`, `*~`, `*.swp`, `*.tmp`, `*.bak`;
- **Pastas ignoradas**: padrão inicial `.Trash-*/**` e `.cache/**`. Para
  ignorar uma pasta, use `nome/**`.

Itens excluídos não são enviados, e o que já estiver no Drive com esses nomes
**não é apagado** (o app nunca usa `--delete-excluded`).

## Painel da bandeja

Um clique no ícone abre um painel com o estado (**Em dia**,
**Sincronizando…**, **Falha** ou **Não configurado**), o horário do último sync,
a origem e o destino, e os botões **Sincronizar agora**, **Configurações…** e
**Sair**.

- No Hyprland, Sway e KDE Plasma (Wayland) ele aparece no canto superior
  direito, abaixo da barra, e fecha com **Esc**, com outro clique no ícone ou
  pouco depois de o ponteiro sair dele.
- Em outros ambientes ele abre como uma janela pequena.
- O menu do ícone (clique direito) tem **Configurações…**, **Sincronizar agora**
  e **Sair**.
- Fechar a janela completa só a esconde; o app continua na bandeja.
- **Sair** espera uma sincronização em andamento terminar.

O horário do último sync fica só na memória: depois de reiniciar o app, ele
mostra "nunca" até a próxima sincronização.

## Arquivo de configuração

Tudo fica em `~/.config/drivemirror/config.toml` (botão
**Abrir arquivo de configuração**). Se você editar o arquivo com o app aberto,
ele recarrega os valores; se houver alterações não salvas na janela, o app avisa
em vez de sobrescrever.

```toml
local_folder = "/home/voce/Notas/Obsidian"
remote = "gdrive"
remote_folder = "Backup/Obsidian"
interval_minutes = 10
exclude = [".DS_Store", "*.tmp"]
exclude_folders = [".cache/**"]
language = "system"
```

O arquivo não guarda credenciais.
