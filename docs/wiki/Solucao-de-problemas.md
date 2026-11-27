🇧🇷 [[Português|Solucao-de-problemas]] · 🇺🇸 [[English|Troubleshooting]] · 🇪🇸 [[Español|Solucion-de-problemas]]

# Solução de problemas

### "O remote "gdrive" não existe no rclone. Crie-o na aba Contas rclone."

O nome em **Remoto rclone** não corresponde a nenhum remoto configurado.

- Crie a conta na aba **Contas rclone**
  ([[Configurar o Google Drive|Configurar-Google-Drive]]), ou
- corrija o nome: só o nome, sem `:` e sem a pasta.

A sincronização automática mostra esse aviso uma vez e só tenta de novo quando
a configuração ou a lista de remotos muda.

### "Google Drive API has not been used in project … before or it is disabled"

A Google Drive API não está ativada no projeto do seu Client ID.

1. Abra o [Google Cloud Console](https://console.cloud.google.com/) no projeto
   do Client ID.
2. **APIs e serviços → Biblioteca → Google Drive API → Ativar**.
3. Espere alguns minutos e clique em **Sincronizar agora**.

### Remoto "incompleto" ou "O remote … não concluiu a autorização"

A criação da conta parou antes de o Google devolver o token: o navegador foi
fechado, a operação foi cancelada ou a autorização falhou. O app não deixa
sincronizar com esse remoto.

- Na aba **Contas rclone**, clique em **Reconectar** e autorize no navegador; ou
- clique em **Remover** e crie a conta de novo.

### O navegador não abriu para autorizar

Enquanto o app mostra "Aguardando autorização no navegador…", clique em
**Abrir link de autorização** (passe o mouse sobre ele para ver o endereço). O
endereço começa com `http://127.0.0.1` e só funciona neste computador, enquanto
a autorização está pendente.

### A autorização para de funcionar depois de alguns dias

Se o app no Google Cloud estiver em modo de **teste**, o Google pode expirar a
autorização depois de alguns dias. Use **Reconectar** na aba Contas rclone ou
publique o app na tela de consentimento OAuth, conforme o
[guia do rclone](https://rclone.org/drive/#making-your-own-client-id).

### O painel não abre abaixo da barra

O painel ancorado usa o protocolo layer-shell, disponível no Hyprland, Sway e
KDE Plasma (Wayland). No GNOME, em X11 e em outros compositores ele abre como
janela pequena, o que é esperado.

### Não aparece ícone na bandeja

A barra precisa suportar StatusNotifierItem (Waybar, KDE Plasma etc.). No GNOME
instale uma extensão de AppIndicator. Sem bandeja, o app avisa na janela
completa e continua funcionando por ela; abra-o pelo menu de aplicativos.

### "Escolha uma subpasta remota; a raiz do Drive não é permitida"

Informe uma subpasta em **Pasta no Drive**, por exemplo `Backup/Obsidian`.

### "A configuração mudou externamente; revise-a antes de salvar"

O `config.toml` foi alterado fora do app enquanto havia alterações não salvas
na janela. Confira os valores exibidos e salve de novo.

### "rclone não foi encontrado; instale-o ou verifique o PATH"

Instale o rclone (`sudo pacman -S rclone` ou `sudo apt install rclone`).

### Ainda com problemas?

Abra uma issue em <https://github.com/israelguido/drivemirror/issues>
descrevendo a distribuição, a versão e a mensagem exibida. Não inclua Client
Secret, tokens nem o seu `rclone.conf`. Para falhas de segurança, siga o
[SECURITY.md](https://github.com/israelguido/drivemirror/blob/main/SECURITY.md).
