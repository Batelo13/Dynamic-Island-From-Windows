## MODIFIED Requirements

### Requirement: Estados visuais
O sistema SHALL suportar quatro estados visuais, ancorados no mesmo centro horizontal; os três visíveis são pretos e compartilham a mesma borda superior:
- **Oculta**: nada desenhado na tela; a forma fica recolhida acima do topo da área útil.
- **Idle**: 126 × 37 DIP, raio igual a metade da altura.
- **Compact**: pílula larga, mesma altura do Idle, com um slot à esquerda (ícone) e um à direita (informação).
- **Expanded**: retângulo arredondado com raio de ~32 DIP, grande o bastante para título, subtítulo e uma linha de controles.

Tamanhos, raios, cores e tipografia SHALL vir de um único conjunto de tokens.

#### Scenario: Ancoragem
- **WHEN** a ilha muda de qualquer estado para outro
- **THEN** o centro horizontal e a borda superior da forma permanecem fixos (tolerância de 1 px quando parada)

#### Scenario: Sem atividade
- **WHEN** não há atividade ativa e a ilha não foi revelada pelo topo
- **THEN** a ilha fica Oculta

#### Scenario: Atividade ativa
- **WHEN** uma atividade começa (música tocando, ou o demo em builds de debug)
- **THEN** a ilha passa para Compact e mostra o conteúdo da atividade nos slots, e permanece visível enquanto a atividade durar

#### Scenario: Atividade termina
- **WHEN** a atividade termina e o cursor não está sobre a ilha
- **THEN** a ilha sobe e fica Oculta

### Requirement: Revelar pelo topo
Quando a ilha está Oculta (sem atividade, ou com música pausada há mais de 30 s), o sistema SHALL revelá-la quando o cursor permanecer por 150 ms numa faixa invisível de 2 px de altura × a largura do Idle, no centro superior da área útil, e SHALL ocultá-la de novo quando o cursor ficar fora dela por ~1 s. Sem atividade, a forma revelada é a cápsula Idle; com música pausada, é o Compact da música. A faixa existe só enquanto a ilha está Oculta.

#### Scenario: Revelar
- **WHEN** a ilha está Oculta sem atividade e o cursor fica 150 ms na faixa do centro superior
- **THEN** a cápsula Idle desce do topo com movimento de mola, sem tomar foco

#### Scenario: Revelar com música pausada
- **WHEN** a ilha está Oculta com música pausada há mais de 30 s e o cursor fica 150 ms na faixa
- **THEN** o Compact da música pausada desce do topo

#### Scenario: Passagem rápida
- **WHEN** o cursor passa pela faixa e sai antes de 150 ms
- **THEN** a ilha continua Oculta

#### Scenario: Ocultar ao sair
- **WHEN** a ilha foi revelada e o cursor fica fora dela por 1 s
- **THEN** a ilha sobe e fica Oculta

#### Scenario: Fora do centro
- **WHEN** o cursor toca o topo da tela fora da faixa (ex.: 1 px ao lado)
- **THEN** a ilha continua Oculta
