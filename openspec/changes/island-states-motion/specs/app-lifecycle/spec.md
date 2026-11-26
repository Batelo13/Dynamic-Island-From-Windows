## MODIFIED Requirements

### Requirement: Consumo em repouso
Sem atividades (ilha Oculta ou Idle), o sistema SHALL não executar laço de renderização nem polling periódico, e SHALL ficar abaixo de 50 MB de conjunto de trabalho privado (a memória que o Gerenciador de Tarefas mostra) e de 0,5% de CPU média em uma janela de 10 minutos, medidos pelo roteiro de benchmark do projeto. O commit privado SHALL ser registrado no benchmark como informação, sem meta: o Windows reserva commit não utilizado quando há dispositivo gráfico no processo.

#### Scenario: Medição em repouso
- **WHEN** o roteiro de benchmark mede o app parado por 10 minutos
- **THEN** o conjunto de trabalho privado fica abaixo de 50 MB e a CPU média abaixo de 0,5%

#### Scenario: Sem texto exibido
- **WHEN** o app roda sem nunca ter exibido texto (sem atividade)
- **THEN** nenhum dispositivo gráfico é criado no processo e o commit privado fica no mesmo patamar da Fase 1 (poucos MB)
