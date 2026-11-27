## Purpose

Mostra na Dynamic Island a música (ou outra mídia) que está tocando no Windows, com capa, informações e controles de reprodução, a partir da sessão de mídia atual do sistema.

## ADDED Requirements

### Requirement: Sessão de mídia atual
O sistema SHALL acompanhar a sessão de mídia que o Windows considera atual (a mesma exibida pelos controles de mídia do sistema), de qualquer player que publique sessão (ex.: Spotify, navegadores). O sistema SHALL reagir a mudanças de sessão, de faixa e de estado de reprodução por notificação do sistema, sem consultar periodicamente.

#### Scenario: Música começa
- **WHEN** o usuário dá play numa música no Spotify
- **THEN** em até 1 segundo a ilha aparece em Compact com a capa da faixa, sem gesto nenhum

#### Scenario: Troca de faixa
- **WHEN** a faixa muda (próxima, anterior ou fim da faixa)
- **THEN** capa, título e artista exibidos passam a ser os da nova faixa em até 1 segundo

#### Scenario: Vários players
- **WHEN** dois players publicam sessões ao mesmo tempo
- **THEN** a ilha mostra apenas a sessão que o Windows indica como atual e acompanha quando o Windows muda de sessão

#### Scenario: Player fechado
- **WHEN** o player que estava na ilha é fechado
- **THEN** a ilha passa para a próxima sessão atual do Windows ou, se não houver nenhuma, fica Oculta

#### Scenario: Player fechado para a bandeja
- **WHEN** o usuário fecha a janela do player e ele continua rodando em segundo plano com a sessão vazia (sem título)
- **THEN** a ilha trata isso como "sem música": fica Oculta e, revelada pelo topo, mostra a cápsula Idle

### Requirement: Compact de música
Com música tocando, o Compact SHALL mostrar a capa da faixa em miniatura, com cantos arredondados, à esquerda, e barrinhas de equalizador animadas à direita. Com a música pausada, as barrinhas SHALL ficar paradas e baixas.

#### Scenario: Tocando
- **WHEN** a música está tocando
- **THEN** o Compact mostra a capa à esquerda e as barrinhas se movendo à direita

#### Scenario: Pausada
- **WHEN** a música é pausada
- **THEN** as barrinhas param em uma altura baixa e uniforme

#### Scenario: Sem capa
- **WHEN** a faixa não tem capa disponível
- **THEN** o lugar da capa mostra um ícone de nota musical sobre fundo cinza-escuro

### Requirement: Expanded de música
Com música como atividade, o Expanded SHALL mostrar capa, título, artista, uma barra de progresso com tempo decorrido à esquerda e restante à direita, e os botões anterior, tocar/pausar (o ícone reflete o estado atual) e próxima. A barra de progresso é apenas exibição.

#### Scenario: Conteúdo
- **WHEN** o usuário clica na ilha em Compact com música
- **THEN** o Expanded mostra capa, título, artista, barra de progresso, tempos e os três botões

#### Scenario: Progresso avança
- **WHEN** a música toca com o Expanded aberto
- **THEN** a barra avança continuamente e os tempos se atualizam a cada segundo

#### Scenario: Texto longo
- **WHEN** título ou artista não cabem na largura disponível
- **THEN** o texto é cortado com reticências, sem quebrar linha nem invadir os outros elementos

#### Scenario: Duração desconhecida
- **WHEN** o player não informa duração ou posição (ex.: transmissão ao vivo)
- **THEN** a barra de progresso e os tempos ficam ocultos, e o resto do Expanded permanece

### Requirement: Controles de reprodução
Os botões do Expanded SHALL enviar ao player os comandos anterior, tocar/pausar e próxima, sem tirar o foco nem trazer o player para frente. Botões que o player não suporta no momento SHALL aparecer esmaecidos e não fazer nada.

#### Scenario: Pausar
- **WHEN** o usuário clica em tocar/pausar com a música tocando
- **THEN** a música pausa, o ícone vira "tocar" em até 1 segundo e a janela em primeiro plano continua a mesma

#### Scenario: Próxima
- **WHEN** o usuário clica em próxima
- **THEN** o player pula para a próxima faixa e a ilha mostra a nova faixa

#### Scenario: Comando não suportado
- **WHEN** o player informa que não aceita "anterior"
- **THEN** o botão anterior aparece esmaecido e o clique nele não faz nada

#### Scenario: Passar o mouse sobre um botão
- **WHEN** o cursor passa sobre um botão habilitado do Expanded
- **THEN** um círculo claro e suave cresce atrás do ícone, o cursor vira a mão de clique, e ambos somem quando o cursor sai do botão; botões esmaecidos não reagem

#### Scenario: Clique fora dos botões
- **WHEN** o usuário clica no Expanded fora dos botões
- **THEN** nada acontece (a ilha não recolhe nem troca de estado)

### Requirement: Pausa e visibilidade
Com a música pausada, a ilha SHALL continuar visível por 30 s e então ficar Oculta. Se a reprodução voltar antes disso, SHALL continuar visível sem interrupção. Enquanto a ilha estiver Oculta com música pausada, revelar pelo topo SHALL mostrar o Compact da música pausada.

#### Scenario: Pausa longa
- **WHEN** a música fica pausada por 30 s e o cursor não está sobre a ilha
- **THEN** a ilha sobe e fica Oculta

#### Scenario: Retomada rápida
- **WHEN** a música é pausada e retomada antes de 30 s
- **THEN** a ilha permanece visível o tempo todo

#### Scenario: Revelar com música pausada
- **WHEN** a ilha está Oculta com música pausada e o usuário revela pelo topo
- **THEN** aparece o Compact da música pausada, e um clique abre o Expanded com os controles

#### Scenario: Retomar pela ilha
- **WHEN** o usuário clica em tocar no Expanded de uma música pausada
- **THEN** a música volta a tocar e a ilha permanece visível

### Requirement: Custo da música
Com música tocando e a ilha em Compact, o sistema SHALL ficar abaixo de 50 MB de conjunto de trabalho privado e não executar trabalho periódico no processo do app; as animações das barrinhas e da barra de progresso SHALL rodar no compositor do sistema. A atualização dos tempos a cada segundo SHALL ocorrer apenas enquanto o Expanded estiver visível.

#### Scenario: Ouvindo por 10 minutos
- **WHEN** o roteiro de benchmark mede o app por 10 minutos com música tocando e a ilha em Compact
- **THEN** o conjunto de trabalho privado fica abaixo de 50 MB e a CPU média do app abaixo de 0,5%

#### Scenario: Expanded fechado
- **WHEN** o Expanded recolhe
- **THEN** a atualização dos tempos para e nenhum timer periódico permanece ativo
