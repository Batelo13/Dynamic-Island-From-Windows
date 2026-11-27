## Purpose

Define como o app inicia, roda continuamente com consumo mínimo, persiste preferências, registra falhas e encerra, sem exigir privilégios administrativos.

## Requirements

### Requirement: Instância única
O sistema SHALL permitir apenas uma instância por sessão de usuário. Uma segunda execução SHALL encerrar-se silenciosamente sem afetar a primeira.

#### Scenario: Segunda execução
- **WHEN** o usuário executa o app enquanto ele já está rodando
- **THEN** o novo processo termina em até 1 segundo e continua existindo apenas uma ilha

### Requirement: Bandeja do sistema
O sistema SHALL exibir um ícone na bandeja com menu contendo, no mínimo, "Mostrar/Ocultar ilha" e "Sair", com textos no idioma do sistema (pt-BR ou inglês, com inglês como fallback).

#### Scenario: Ocultar e mostrar
- **WHEN** o usuário escolhe "Ocultar ilha" e depois "Mostrar ilha"
- **THEN** a cápsula desaparece e reaparece na mesma posição

#### Scenario: Sair
- **WHEN** o usuário escolhe "Sair"
- **THEN** a ilha e o ícone da bandeja desaparecem e o processo termina em até 1 segundo

### Requirement: Sem privilégios administrativos
O sistema SHALL funcionar integralmente com um usuário padrão, sem solicitar elevação (UAC).

#### Scenario: Usuário padrão
- **WHEN** o app é executado por um usuário sem privilégios de administrador
- **THEN** nenhum prompt de UAC aparece e todas as funções deste change funcionam

### Requirement: Configurações persistidas
O sistema SHALL ler e gravar as configurações (no mínimo: monitor, margem superior, visibilidade) em um arquivo local por usuário. Arquivo ausente, ilegível ou inválido SHALL resultar em valores padrão, sem falhar a inicialização e sem sobrescrever o arquivo inválido sem antes preservá-lo.

#### Scenario: Primeira execução
- **WHEN** não existe arquivo de configuração
- **THEN** o app inicia com os valores padrão

#### Scenario: Arquivo corrompido
- **WHEN** o arquivo de configuração contém JSON inválido
- **THEN** o app inicia com os valores padrão e preserva o arquivo original com o sufixo `.bak`

#### Scenario: Valores fora da faixa
- **WHEN** a margem superior configurada é negativa ou maior que 200 DIP
- **THEN** o valor é limitado à faixa válida

### Requirement: Log técnico limitado
O sistema SHALL registrar erros técnicos em arquivo local de tamanho máximo limitado (1 MB, com uma rotação) e SHALL NOT registrar conteúdo de mensagens, notificações, títulos de mídia ou dados pessoais.

#### Scenario: Limite de tamanho
- **WHEN** o log atinge 1 MB
- **THEN** ele é rotacionado e o total em disco não passa de 2 MB

### Requirement: Consumo em repouso
Sem atividades (ilha Oculta ou Idle), o sistema SHALL não executar laço de renderização nem polling periódico, e SHALL ficar abaixo de 50 MB de conjunto de trabalho privado (a memória que o Gerenciador de Tarefas mostra) e de 0,5% de CPU média em uma janela de 10 minutos, medidos pelo roteiro de benchmark do projeto. O commit privado SHALL ser registrado no benchmark como informação, sem meta: o Windows reserva commit não utilizado quando há dispositivo gráfico no processo.

#### Scenario: Medição em repouso
- **WHEN** o roteiro de benchmark mede o app parado por 10 minutos
- **THEN** o conjunto de trabalho privado fica abaixo de 50 MB e a CPU média abaixo de 0,5%

#### Scenario: Sem texto exibido
- **WHEN** o app roda sem nunca ter exibido texto (sem atividade)
- **THEN** nenhum dispositivo gráfico é criado no processo e o commit privado fica no mesmo patamar da Fase 1 (poucos MB)


### Requirement: Inicialização rápida
O sistema SHALL exibir a cápsula em menos de 2 segundos a partir do início do processo.

#### Scenario: Partida a frio
- **WHEN** o app é iniciado após um reinício do computador
- **THEN** a cápsula aparece em menos de 2 segundos (medido pelo próprio log de inicialização)
