# DriveMirror

O DriveMirror é um app pequeno para Linux que mantém uma **cópia espelhada de
uma pasta do seu computador** em uma subpasta privada do **Google Drive**. Ele
usa o [rclone](https://rclone.org/) por baixo e fica na bandeja do sistema,
sincronizando quando você pede e a cada intervalo configurado.

![Aba Sincronização do DriveMirror com pasta local, remote, pasta no Drive, intervalo, idioma e exclusões](https://raw.githubusercontent.com/israelguido/drivemirror/main/docs/images/janela-sincronizacao.png)

*Capturas feitas com dados de demonstração.*

## Para quem é

- Quem usa Linux (Omarchy, Arch, Debian 13, Ubuntu 24.10+) e quer uma pasta
  sempre copiada para o Drive, sem cliente oficial do Google.
- Quem prefere uma interface simples a montar comandos e timers do rclone na
  mão.
- Quem entende que é um **espelho de mão única**: o Drive passa a refletir a
  pasta local, inclusive as exclusões.

Não é um backup com histórico de versões nem uma sincronização de duas vias.
Arquivos apagados no computador são apagados no Drive na sincronização
seguinte. Veja [[Perguntas frequentes|Perguntas-frequentes]].

## Por onde começar

1. [[Instalação|Instalacao]]: baixar e instalar o pacote.
2. [[Configurar o Google Drive|Configurar-Google-Drive]]: Client ID, Google
   Drive API e criação da conta no app.
3. [[Uso diário|Uso-diario]]: painel da bandeja, sincronização, intervalo e
   exclusões.
4. [[Solução de problemas|Solucao-de-problemas]]: mensagens de erro comuns e o
   que fazer.

## Links

- Repositório e README: <https://github.com/israelguido/drivemirror>
- Pacotes: <https://github.com/israelguido/drivemirror/tree/main/releases>
- Segurança e como reportar vulnerabilidades:
  <https://github.com/israelguido/drivemirror/blob/main/SECURITY.md>
- Licença: MIT
