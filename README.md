# DriveMirror

DriveMirror espelha uma pasta local para uma pasta privada de um remoto do
[rclone](https://rclone.org/). Ele usa `rclone sync`: apagar um arquivo na
pasta local também o apaga no destino na próxima sincronização.

## Recursos do MVP

- sincronização manual pela janela ou pela bandeja;
- intervalo automático configurável, com padrão de 10 minutos;
- filtros de arquivos e pastas em uma interface simples;
- configuração TOML editável em `~/.config/drivemirror/config.toml`;
- tema GTK do sistema e cor de destaque do Omarchy;
- inicialização automática no login;
- pacotes para Arch/Omarchy e Debian/Ubuntu/Pop!_OS.

DriveMirror não cria links, compartilhamentos ou permissões públicas e não lê
as credenciais do rclone. Configure primeiro um remoto com `rclone config`.

## Uso

1. Escolha uma pasta local.
2. Informe somente o nome do remoto, como `gdrive`.
3. Informe uma subpasta, como `Backup/Documentos`. A raiz do Drive é bloqueada.
4. Ajuste o intervalo e os filtros, salve e clique em **Sincronizar agora**.

Os padrões iniciais ignoram `.DS_Store`, `Thumbs.db`, `desktop.ini`, backups
`*.bak`, temporários e caches comuns. Itens excluídos não são apagados do
destino, pois o app nunca usa `--delete-excluded`.

## Desenvolvimento

```sh
./bin/test
./bin/build
```

No Omarchy/Arch, instale com `./bin/install`. Em Debian, Ubuntu ou Pop!_OS,
gere o pacote com `./packaging/debian/build-deb` e instale o `.deb` resultante.
Gere o `.deb` na distribuição em que ele será usado, para vinculá-lo às versões
de GTK e glibc disponíveis naquele sistema.

## Segurança

O arquivo TOML guarda apenas caminhos, nome do remoto, filtros e intervalo.
Credenciais continuam exclusivamente no arquivo de configuração do rclone,
que não faz parte deste repositório.

