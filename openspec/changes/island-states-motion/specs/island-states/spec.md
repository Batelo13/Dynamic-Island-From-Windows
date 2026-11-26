## Purpose

Define os estados visuais da Dynamic Island (Idle, Compact, Expanded), como ela transita entre eles com animação por mola e como responde ao mouse, mantendo custo zero quando parada.

## ADDED Requirements

### Requirement: Estados visuais
O sistema SHALL suportar três estados visuais, todos pretos, ancorados no mesmo centro horizontal e na mesma borda superior:
- **Idle**: 126 × 37 DIP, raio igual a metade da altura.
- **Compact**: pílula larga, mesma altura do Idle, com um slot à esquerda (ícone) e um à direita (informação).
- **Expanded**: retângulo arredondado com raio de ~32 DIP, grande o bastante para título, subtítulo e uma linha de controles.

Tamanhos, raios, cores e tipografia SHALL vir de um único conjunto de tokens.

#### Scenario: Ancoragem
- **WHEN** a ilha muda de qualquer estado para outro
- **THEN** o centro horizontal e a borda superior da forma permanecem fixos (tolerância de 1 px quando parada)

#### Scenario: Sem atividade
- **WHEN** não há atividade ativa
- **THEN** a ilha fica em Idle

#### Scenario: Atividade ativa
- **WHEN** uma atividade começa (no modo demo, nesta fase)
- **THEN** a ilha passa para Compact e mostra o conteúdo da atividade nos slots

### Requirement: Transição por mola
O sistema SHALL animar tamanho, raio e posição da forma entre estados com movimento de mola. A animação SHALL poder ser interrompida a qualquer momento, partindo da forma atual, sem saltos.

#### Scenario: Interrupção
- **WHEN** uma nova transição começa antes de a anterior terminar
- **THEN** a forma segue de onde está para o novo destino, sem voltar ao estado anterior nem pular

#### Scenario: Duração
- **WHEN** a ilha muda de estado com animações habilitadas
- **THEN** a forma assenta visualmente em até 600 ms

#### Scenario: Conteúdo durante a transição
- **WHEN** a forma está mudando de estado
- **THEN** o conteúdo do estado anterior some antes de ficar cortado, e o conteúdo do novo estado só aparece quando há espaço para ele

### Requirement: Interação com o mouse
O sistema SHALL responder ao mouse sem tomar foco:
- cursor sobre a ilha em Idle ou Compact → leve crescimento (feedback de hover);
- clique em Compact → passa para Expanded;
- cursor fora da ilha por ~1 s → Expanded recolhe para Compact e o hover se desfaz.

#### Scenario: Hover
- **WHEN** o cursor entra na ilha em Idle ou Compact
- **THEN** a forma cresce levemente (≤ 8% em cada dimensão) e volta ao tamanho normal quando o cursor sai

#### Scenario: Clique expande
- **WHEN** o usuário clica na ilha em Compact
- **THEN** a ilha passa para Expanded e a janela em primeiro plano continua a mesma

#### Scenario: Clique em Idle
- **WHEN** o usuário clica na ilha em Idle, sem atividade
- **THEN** o estado não muda

#### Scenario: Recolher ao sair
- **WHEN** a ilha está em Expanded e o cursor fica fora dela por 1 s (inclusive após clicar em outra aplicação)
- **THEN** a ilha recolhe para Compact

#### Scenario: Volta antes do tempo
- **WHEN** o cursor sai da ilha em Expanded e volta antes de 1 s
- **THEN** a ilha continua em Expanded

### Requirement: Texto nítido
O sistema SHALL renderizar o texto dos estados Compact e Expanded com antisserrilhamento, nítido na escala de DPI do monitor, e SHALL redesenhá-lo automaticamente se o dispositivo gráfico for perdido ou trocado.

#### Scenario: Mudança de escala
- **WHEN** a escala do monitor muda com a ilha em Compact ou Expanded
- **THEN** o texto é redesenhado na nova escala, sem desfoque

#### Scenario: Perda de dispositivo gráfico
- **WHEN** o driver de vídeo é reiniciado ou a GPU muda
- **THEN** a ilha volta a exibir o texto corretamente, sem reiniciar o app

### Requirement: Redução de animações
O sistema SHALL respeitar a configuração "Efeitos de animação" do Windows: quando desligada, as mudanças de estado e de hover SHALL ser aplicadas sem animação.

#### Scenario: Animações desligadas
- **WHEN** "Efeitos de animação" está desligado e a ilha muda de estado
- **THEN** a forma final aparece imediatamente, sem movimento intermediário

#### Scenario: Configuração alterada com o app aberto
- **WHEN** o usuário liga ou desliga "Efeitos de animação" com o app aberto
- **THEN** a próxima transição já segue a nova configuração

### Requirement: Custo da animação
O sistema SHALL executar trabalho apenas durante transições. Depois que a forma assenta, SHALL não restar laço de renderização, timer periódico nem polling.

#### Scenario: Após assentar
- **WHEN** a ilha assenta em qualquer estado e não há interação por 10 minutos
- **THEN** o consumo médio fica abaixo de 0,5% de CPU e 50 MB de memória privada, medido pelo roteiro de benchmark

#### Scenario: Durante transições
- **WHEN** o roteiro de benchmark força transições contínuas por 30 s
- **THEN** a CPU média do processo do app é registrada em `docs/benchmarks.md`, e a CPU volta ao nível de repouso em até 2 s depois que as transições param

### Requirement: Bloqueio e suspensão
O sistema SHALL cancelar as interações pendentes quando a sessão é bloqueada ou o computador suspende, e SHALL retomar já no estado de repouso correto.

#### Scenario: Bloqueio com a ilha expandida
- **WHEN** a sessão é bloqueada com a ilha em Expanded
- **THEN** ao desbloquear, a ilha aparece em Compact (ou em Idle, sem atividade), sem hover

### Requirement: Modo demo em builds de debug
Em builds de debug, o sistema SHALL oferecer na bandeja um menu "Demo" para iniciar e encerrar uma atividade de exemplo com texto placeholder. Esse menu SHALL não existir em builds de release.

#### Scenario: Demo em debug
- **WHEN** o desenvolvedor escolhe Demo → Iniciar atividade em um build de debug
- **THEN** a ilha passa para Compact com o texto de exemplo, e Demo → Encerrar a devolve para Idle

#### Scenario: Release
- **WHEN** o menu da bandeja é aberto em um build de release
- **THEN** não há item Demo
