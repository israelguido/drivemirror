# DriveMirror

DriveMirror espelha uma pasta local para uma pasta privada de um remoto do
[rclone](https://rclone.org/). Ele usa `rclone sync`: apagar um arquivo na
pasta local também o apaga no destino na próxima sincronização.

## Recursos do MVP

- sincronização manual pela janela ou pela bandeja;
- painel compacto ao clicar no ícone da bandeja (estado, último sync, origem e
  destino, **Sincronizar agora**, **Configurações…** e **Sair**); em Hyprland,
  Sway e KDE Wayland ele aparece logo abaixo da barra superior via
  `gtk4-layer-shell`, e nos demais ambientes como uma janela pequena;
- intervalo automático configurável, com padrão de 10 minutos;
- filtros de arquivos e pastas em uma interface simples;
- configuração TOML editável em `~/.config/drivemirror/config.toml`;
- tema GTK do sistema e cor de destaque do Omarchy;
- interface em português (Brasil), inglês e espanhol;
- página **Contas rclone** para criar, reconectar e remover remotos do Google Drive;
- inicialização automática no login;
- pacotes para Arch/Omarchy e Debian/Ubuntu/Pop!_OS.

DriveMirror não cria links, compartilhamentos ou permissões públicas e não lê
as credenciais do rclone. Configure o remoto na página **Contas rclone** ou com
`rclone config`.

## Uso

1. Escolha uma pasta local.
2. Informe somente o nome do remoto, como `gdrive`.
3. Informe uma subpasta, como `Backup/Documentos`. A raiz do Drive é bloqueada.
4. Ajuste o intervalo e os filtros, salve e clique em **Sincronizar agora**.

Os padrões iniciais ignoram `.DS_Store`, `Thumbs.db`, `desktop.ini`, backups
`*.bak`, temporários e caches comuns. Itens excluídos não são apagados do
destino, pois o app nunca usa `--delete-excluded`.

## Idiomas

Por padrão o idioma segue o locale do sistema (`LC_ALL`, `LC_MESSAGES`, `LANG`):
locales `pt*` usam português (Brasil), `es*` usam espanhol e os demais usam
inglês. O seletor **Idioma** na janela troca o idioma na hora e grava
`language = "system" | "pt-BR" | "en" | "es"` no TOML. Arquivos antigos sem esse
campo continuam funcionando como `system`.

## Contas rclone

A página **Contas rclone** lista os remotos (`rclone listremotes --long`) e cria
um remoto do Google Drive a partir de nome, Client ID e Client Secret:

1. Crie seu próprio Client ID seguindo
   <https://rclone.org/drive/#making-your-own-client-id>. O app não cria
   projetos nem credenciais no Google Cloud.
2. Preencha os campos e clique em **Criar remote**. O app conduz o protocolo
   `rclone config create --non-interactive` e mostra cada pergunta do rclone.
3. Quando o rclone abrir o navegador, autorize o acesso. O botão **Cancelar**
   encerra o processo; se a criação não terminar, o app oferece remover o
   remoto incompleto.

**Reconectar** renova o token pelo mesmo protocolo. **Remover** pede
confirmação e apaga só a configuração do rclone; nenhum arquivo do Google Drive
é apagado. Só remotos do tipo `drive` podem ser reconectados; os demais aparecem
na lista e podem ser removidos.

## Desenvolvimento

```sh
./bin/test
./bin/build
```

No Omarchy/Arch, instale com `./bin/install`. Em Debian, Ubuntu ou Pop!_OS,
gere o pacote com `./packaging/debian/build-deb` e instale o `.deb` resultante.
Gere o `.deb` na distribuição em que ele será usado, para vinculá-lo às versões
de GTK e glibc disponíveis naquele sistema. O `.deb` depende de
`libgtk4-layer-shell0`, disponível a partir do Debian 13 e do Ubuntu 24.10.

## Segurança

O arquivo TOML guarda apenas caminhos, nome do remoto, filtros e intervalo.
Credenciais continuam exclusivamente no arquivo de configuração do rclone,
que não faz parte deste repositório.

O Client Secret só existe na memória do app enquanto a criação do remoto está
em andamento (o rclone precisa dele em cada etapa) e o campo é limpo assim que o
processo começa. Ele nunca é gravado pelo app nem aparece em logs ou mensagens:
toda saída do rclone exibida passa por uma redação que o troca por `<redacted>`.
Como qualquer argumento de processo, ele fica visível para o próprio usuário em
`/proc` enquanto o rclone roda.

