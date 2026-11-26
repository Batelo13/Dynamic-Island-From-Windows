## Purpose

Define os estados visuais da Dynamic Island (Oculta, Idle, Compact, Expanded), como ela aparece pelo topo da tela, transita entre estados com animação por mola e responde ao mouse, mantendo custo zero quando parada.

## ADDED Requirements

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
- **WHEN** uma atividade começa (no modo demo, nesta fase)
- **THEN** a ilha passa para Compact e mostra o conteúdo da atividade nos slots, e permanece visível enquanto a atividade durar

#### Scenario: Atividade termina
- **WHEN** a atividade termina e o cursor não está sobre a ilha
- **THEN** a ilha sobe e fica Oculta

### Requirement: Revelar pelo topo
Sem atividade, o sistema SHALL revelar a cápsula Idle quando o cursor permanecer por 150 ms numa faixa invisível de 2 px de altura × a largura do Idle, no centro superior da área útil, e SHALL ocultá-la de novo quando o cursor ficar fora dela por ~1 s. A faixa existe só enquanto a ilha está Oculta.

#### Scenario: Revelar
- **WHEN** a ilha está Oculta e o cursor fica 150 ms na faixa do centro superior
- **THEN** a cápsula Idle desce do topo com movimento de mola, sem tomar foco

#### Scenario: Passagem rápida
- **WHEN** o cursor passa pela faixa e sai antes de 150 ms
- **THEN** a ilha continua Oculta

#### Scenario: Ocultar ao sair
- **WHEN** a ilha foi revelada sem atividade e o cursor fica fora dela por 1 s
- **THEN** a ilha sobe e fica Oculta

#### Scenario: Fora do centro
- **WHEN** o cursor toca o topo da tela fora da faixa (ex.: 1 px ao lado)
- **THEN** a ilha continua Oculta

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
- clique em Idle ou Compact → passa para Expanded (sem atividade, o Expanded mostra hora e data como conteúdo provisório);
- cursor fora da ilha por ~1 s → recolhe para Compact (com atividade) ou fica Oculta (sem atividade), e o hover se desfaz.

#### Scenario: Hover
- **WHEN** o cursor entra na ilha em Idle ou Compact
- **THEN** a forma cresce levemente (≤ 8% em cada dimensão) e volta ao tamanho normal quando o cursor sai

#### Scenario: Clique expande
- **WHEN** o usuário clica na ilha em Compact
- **THEN** a ilha passa para Expanded e a janela em primeiro plano continua a mesma

#### Scenario: Clique em Idle
- **WHEN** o usuário clica na cápsula Idle revelada, sem atividade
- **THEN** a ilha passa para Expanded mostrando hora e data, e a janela em primeiro plano continua a mesma

#### Scenario: Recolher ao sair
- **WHEN** a ilha está em Expanded e o cursor fica fora dela por 1 s (inclusive após clicar em outra aplicação)
- **THEN** a ilha recolhe para Compact se houver atividade, ou sobe e fica Oculta se não houver

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
- **WHEN** "Efeitos de animação" está desligado e a ilha muda de estado, aparece ou é ocultada
- **THEN** a forma final aparece (ou some) imediatamente, sem movimento intermediário

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
- **THEN** ao desbloquear, a ilha aparece em Compact (ou fica Oculta, sem atividade), sem hover

### Requirement: Modo demo em builds de debug
Em builds de debug, o sistema SHALL oferecer na bandeja um menu "Demo" para iniciar e encerrar uma atividade de exemplo com texto placeholder. Esse menu SHALL não existir em builds de release.

#### Scenario: Demo em debug
- **WHEN** o desenvolvedor escolhe Demo → Iniciar atividade em um build de debug
- **THEN** a ilha passa para Compact com o texto de exemplo, e Demo → Encerrar a devolve para Idle

#### Scenario: Release
- **WHEN** o menu da bandeja é aberto em um build de release
- **THEN** não há item Demo
