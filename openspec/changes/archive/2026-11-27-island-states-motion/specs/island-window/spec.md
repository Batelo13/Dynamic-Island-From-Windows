## MODIFIED Requirements

### Requirement: Cápsula Idle
Sem atividade, o sistema SHALL manter a ilha oculta até ser revelada pelo topo (ver `island-states`). Quando revelada, SHALL exibir, no estado Idle, uma cápsula preta opaca com cantos totalmente arredondados (raio = metade da altura), sem bordas, título ou controles do Windows, com fundo transparente ao redor. A cápsula SHALL ter uma sombra suave e um contorno de 1 px cinza-escuro sutil, para seguir distinguível sobre fundos escuros.

#### Scenario: Aparência inicial
- **WHEN** o app é iniciado sem atividades
- **THEN** nada da ilha é visível na tela

#### Scenario: Cápsula revelada
- **WHEN** a ilha é revelada pelo topo sem atividade
- **THEN** apenas a cápsula Idle é visível, com tamanho padrão de 126 × 37 DIP (proporção ~3,4:1 das refs 1 e 2)
- **AND** a área ao redor da cápsula mostra o conteúdo das janelas de baixo, sem retângulo, borda ou sombra do Windows

#### Scenario: Fundo escuro
- **WHEN** a cápsula está sobre uma barra de título ou fundo preto/escuro
- **THEN** o contorno da cápsula continua perceptível

#### Scenario: Fundo claro
- **WHEN** a cápsula está sobre um fundo branco
- **THEN** a sombra é discreta (sem halo escuro marcado) e o contorno não se destaca do preto da cápsula

### Requirement: Click-through fora da cápsula
O sistema SHALL deixar passar para as janelas de baixo todos os eventos de mouse fora da forma atual da ilha (em qualquer estado), e SHALL receber os eventos de mouse dentro dela. Durante uma transição, a área clicável SHALL corresponder à forma de destino. Com a ilha Oculta, a única área que recebe eventos SHALL ser a faixa de revelação (2 px × largura do Idle, no centro superior da área útil); cliques nesses pixels não chegam às janelas de baixo (concessão aceita para detectar o cursor sem hook global nem polling).

#### Scenario: Clique ao lado da cápsula
- **WHEN** o usuário clica 2 px fora da borda da forma atual, sobre outra aplicação
- **THEN** a outra aplicação recebe o clique normalmente

#### Scenario: Clique na cápsula
- **WHEN** o usuário clica dentro da forma atual
- **THEN** o app registra o clique e nenhuma janela de baixo o recebe

#### Scenario: Área da sombra
- **WHEN** o usuário clica sobre a sombra da ilha, fora da forma
- **THEN** a outra aplicação recebe o clique

#### Scenario: Estado expandido
- **WHEN** a ilha está em Expanded e o usuário clica 2 px abaixo da borda inferior da forma
- **THEN** a outra aplicação recebe o clique

#### Scenario: Faixa de revelação
- **WHEN** a ilha está Oculta e o usuário clica 1 px ao lado da faixa de revelação, ou 1 px abaixo dela
- **THEN** a outra aplicação recebe o clique normalmente

#### Scenario: Oculta não bloqueia a tela
- **WHEN** a ilha está Oculta e o usuário clica onde a cápsula Idle apareceria
- **THEN** a outra aplicação recebe o clique normalmente
