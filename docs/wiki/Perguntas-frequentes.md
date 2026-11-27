# Perguntas frequentes

### É um backup?

Não no sentido de guardar versões antigas. O DriveMirror faz o Drive ficar
igual à pasta local: se um arquivo é apagado ou estragado no computador, a
próxima sincronização leva essa mudança ao Drive. Para histórico, combine com
outra ferramenta de backup ou com o histórico de versões do próprio Google
Drive.

### As alterações feitas no Drive voltam para o computador?

Não. A cópia é de mão única (computador → Drive). Mudanças feitas direto na
subpasta do Drive são desfeitas na sincronização seguinte.

### Posso sincronizar a raiz do Drive?

Não. O app exige uma subpasta para evitar que um espelho apague o resto do seu
Drive.

### Posso sincronizar mais de uma pasta?

Não nesta versão: uma pasta local para uma subpasta do Drive.

### Por que preciso criar meu próprio Client ID?

O Client ID compartilhado do rclone está sendo desativado pelo Google, e um
Client ID próprio evita limites de uso compartilhados. O app não cria projetos
nem credenciais no Google Cloud por você.

### O que o DriveMirror guarda das minhas credenciais?

Nada. Quem guarda o Client Secret e o token é o rclone, em
`~/.config/rclone/rclone.conf`. Detalhes em
[SECURITY.md](https://github.com/israelguido/drivemirror/blob/main/SECURITY.md).

### Que acesso ao Drive o app pede?

O remoto é criado com o escopo `drive` do rclone, que dá acesso completo ao
Drive da conta autorizada. Isso é necessário para sincronizar em qualquer
subpasta.

### Apagar um remoto apaga meus arquivos?

Não. **Remover** apaga só a configuração do remoto no rclone. Os arquivos no
Drive continuam lá.

### Os arquivos ignorados são apagados do Drive?

Não. Itens que batem com as exclusões não são enviados e também não são
apagados do destino.

### O app sincroniza com o computador desligado ou com o app fechado?

Não. A sincronização automática acontece enquanto o DriveMirror está aberto (ele
inicia na bandeja quando você entra na sessão).

### Funciona no GNOME?

Funciona, com duas diferenças: o painel abre como janela pequena (o GNOME não
oferece o protocolo layer-shell), e o ícone na bandeja exige uma extensão de
AppIndicator. Sem ícone, use o app pelo menu de aplicativos.

### Como troco o idioma?

No seletor **Idioma** da aba Sincronização. "Sistema" segue o idioma da sua
sessão.
