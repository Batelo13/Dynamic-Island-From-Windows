## Purpose

Define como a janela da Dynamic Island aparece, se posiciona e convive com o restante do Windows sem atrapalhar foco, cliques ou digitação.

## Requirements

### Requirement: Cápsula Idle
O sistema SHALL exibir, no estado Idle, uma cápsula preta opaca com cantos totalmente arredondados (raio = metade da altura), sem bordas, título ou controles do Windows, com fundo transparente ao redor.

#### Scenario: Aparência inicial
- **WHEN** o app é iniciado sem atividades
- **THEN** apenas a cápsula Idle é visível, com tamanho padrão de 126 × 37 DIP (proporção ~3,4:1 das refs 1 e 2)
- **AND** a área ao redor da cápsula mostra o conteúdo das janelas de baixo, sem retângulo, borda ou sombra do Windows

### Requirement: Posicionamento padrão e configurável
O sistema SHALL posicionar a cápsula centralizada horizontalmente no topo da área útil (excluindo barras do sistema) do monitor configurado, a uma margem superior configurável (padrão 8 DIP). Na ausência de monitor configurado ou se ele não existir, SHALL usar o monitor principal.

#### Scenario: Monitor principal
- **WHEN** o app inicia com as configurações padrão
- **THEN** o centro horizontal da cápsula coincide com o centro da área útil do monitor principal (tolerância de 1 px)
- **AND** a borda superior da cápsula fica a 8 DIP do topo da área útil

#### Scenario: Monitor configurado desconectado
- **WHEN** o monitor configurado não está mais presente
- **THEN** a cápsula passa para o monitor principal sem encerrar o app

#### Scenario: Barra de tarefas no topo
- **WHEN** a barra de tarefas está ancorada no topo da tela
- **THEN** a cápsula fica abaixo dela, sem sobreposição

### Requirement: DPI e mudanças de tela
O sistema SHALL manter tamanho físico (em DIP) e nitidez corretos em qualquer escala de DPI e SHALL se reposicionar quando resolução, escala, área útil ou conjunto de monitores mudar.

#### Scenario: Escala diferente por monitor
- **WHEN** a cápsula está em um monitor a 150% de escala
- **THEN** ela mede 189 × 56 px físicos e é renderizada sem desfoque

#### Scenario: Mudança de resolução
- **WHEN** a resolução ou a escala do monitor atual muda
- **THEN** a cápsula volta à posição definida em até 1 segundo, sem precisar reiniciar o app

### Requirement: Não roubar foco
O sistema SHALL nunca ativar sua janela nem tirar o foco do teclado de outra aplicação, inclusive ao iniciar, ao se reposicionar ou quando a cápsula é clicada.

#### Scenario: Digitação em andamento
- **WHEN** o usuário está digitando em outra janela e a ilha aparece ou é clicada
- **THEN** a janela em primeiro plano e o foco do teclado permanecem inalterados

#### Scenario: Barra de tarefas e Alt+Tab
- **WHEN** o app está em execução
- **THEN** a ilha não aparece na barra de tarefas nem na lista do Alt+Tab

### Requirement: Click-through fora da cápsula
O sistema SHALL deixar passar para as janelas de baixo todos os eventos de mouse fora da área visível da cápsula, e SHALL receber os eventos de mouse dentro dela.

#### Scenario: Clique ao lado da cápsula
- **WHEN** o usuário clica 2 px fora da borda da cápsula, sobre outra aplicação
- **THEN** a outra aplicação recebe o clique normalmente

#### Scenario: Clique na cápsula
- **WHEN** o usuário clica dentro da cápsula
- **THEN** o app registra o clique e nenhuma janela de baixo o recebe

### Requirement: Sempre visível
O sistema SHALL manter a ilha acima de janelas normais enquanto estiver visível.

#### Scenario: Janela maximizada
- **WHEN** outra aplicação é maximizada
- **THEN** a cápsula continua visível sobre ela

### Requirement: Suspensão, retomada e bloqueio
O sistema SHALL continuar funcional e corretamente posicionado após suspensão/hibernação e após bloqueio/desbloqueio da sessão.

#### Scenario: Retomada
- **WHEN** o computador retoma de suspensão, inclusive com monitores diferentes conectados
- **THEN** a cápsula aparece na posição correta em até 2 segundos, sem reinício manual

#### Scenario: Desbloqueio
- **WHEN** o usuário desbloqueia a sessão
- **THEN** a cápsula continua visível e no lugar certo
