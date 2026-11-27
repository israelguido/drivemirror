# Instalação

O DriveMirror 0.3.0 tem pacotes para dois sistemas, publicados na pasta
[`releases/`](https://github.com/israelguido/drivemirror/tree/main/releases) do
repositório.

| Sistema | Arquivo |
|---|---|
| Omarchy / Arch Linux (x86_64) | `drivemirror-0.3.0-1-x86_64.pkg.tar.zst` |
| Debian 13 / Ubuntu 24.10 ou mais novo (amd64) | `drivemirror_0.3.0_amd64.deb` |

## 1. Baixar

```bash
# Omarchy / Arch
curl -LO https://github.com/israelguido/drivemirror/raw/main/releases/drivemirror-0.3.0-1-x86_64.pkg.tar.zst
# Debian / Ubuntu
curl -LO https://github.com/israelguido/drivemirror/raw/main/releases/drivemirror_0.3.0_amd64.deb
```

## 2. Conferir o arquivo

Baixe também o `SHA256SUMS` e confira:

```bash
curl -LO https://github.com/israelguido/drivemirror/raw/main/releases/SHA256SUMS
sha256sum -c SHA256SUMS --ignore-missing
```

Instale só se a linha do arquivo terminar em `OK`.

## 3. Instalar

```bash
# Omarchy / Arch
sudo pacman -U drivemirror-0.3.0-1-x86_64.pkg.tar.zst
# Debian 13 / Ubuntu 24.10+
sudo apt install ./drivemirror_0.3.0_amd64.deb
```

O gerenciador de pacotes instala junto as dependências: GTK 4,
`gtk4-layer-shell` (`libgtk4-layer-shell0` no Debian/Ubuntu), `rclone` e
`xdg-desktop-portal`. O `.deb` não instala em versões anteriores ao Debian 13 ou
ao Ubuntu 24.10, porque elas não têm `libgtk4-layer-shell0`.

## O que é instalado

- o programa `drivemirror` em `/usr/bin`;
- o atalho no menu de aplicativos;
- uma entrada de início automático em `/etc/xdg/autostart`, que abre o
  DriveMirror direto na bandeja ao entrar na sessão (`drivemirror --tray`).

## Primeira execução

Abra o **DriveMirror** pelo menu de aplicativos. A janela completa aparece para
a configuração inicial. Continue em
[[Configurar o Google Drive|Configurar-Google-Drive]].

## Desinstalar

```bash
# Omarchy / Arch
sudo pacman -R drivemirror
# Debian / Ubuntu
sudo apt remove drivemirror
```

A configuração do DriveMirror (`~/.config/drivemirror/`) e a do rclone
(`~/.config/rclone/rclone.conf`) continuam no seu usuário; apague-as à mão se
quiser. Nenhum arquivo do Drive é apagado.

## Compilar do código-fonte

Veja a seção
[Compilar do código-fonte](https://github.com/israelguido/drivemirror#compilar-do-c%C3%B3digo-fonte)
do README.
