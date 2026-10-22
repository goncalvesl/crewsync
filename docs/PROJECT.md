# Decisões atuais do projeto

Este documento registra o escopo mais recente acordado. O `AGENTS.md` original permanece preservado; suas referências a Windows inicial e compactação não representam o escopo atual.

## Objetivo principal

Ser **simples de instalar, manter e usar, além de rápido**, preservando a integridade
dos arquivos. Minimizar passos, dependências e escolhas na instalação. Preferir
padrões úteis e reduzir a configuração manual. A documentação do usuário deve ser
curta, direta e suficiente para começar; detalhes de arquitetura e funcionamento
ficam em uma seção final opcional. Medir desempenho antes de prometer ou otimizar.

Licença escolhida pelo mantenedor: **GNU GPL versão 3 somente (GPL-3.0-only)**,
texto integral em [LICENSE](../LICENSE). Veja a decisão 0007.

## Requisitos definidos

Nome público: **CrewSync**. Repositório e diretório local: `crewsync`.

- Implementação em Rust, simples de operar, rápida e com integridade dos arquivos.
- Três modos: HTTP explícito para testes isolados, HTTPS por proxy do operador e HTTPS direto (CA privada opcional ou certificado existente). HTTPS valida cadeia e nome, com CAs do sistema ou CA privada; rede local/VPN e servidor remoto suportados (decisões 0009 e 0011).
- Servidor central executado somente em Linux; piloto inicial em dois computadores, com aproximadamente cinco usuários no uso previsto.
- Clientes Linux no piloto; Windows depois e Android no futuro, com acesso online sem exigir operação offline.
- Apenas pastas de primeiro nível compartilhadas; subpastas herdam permissões. O cliente pergunta somente a raiz local e cria dentro dela todas as pastas autorizadas.
- Permissões por usuário em arquivo no servidor: leitura ou leitura e escrita.
- Sem banco de dados externo; estado persistido em arquivos internos.
- Armazenamento `raw`, sem compactação. Não implementar armazenamento compactado.
- Projeto aberto no GitHub; site de apresentação e manual no GitHub Pages, sem domínio próprio; downloads no GitHub Releases.
- Toda funcionalidade deve ter testes automatizados e documentação correspondente.

## Arquitetura proposta para implementação

Conteúdo identificado por hash e manifestos por compartilhamento, inspirados no Git, sem branches ou commits manuais. Tokens individuais por dispositivo vinculados ao usuário, ativados por códigos temporários de uso único; servidor guarda hashes dos tokens. HTTP explícito para testes ou HTTPS validado, autorização por operação, revogação individual e negação de acesso por padrão.

Cliente inicial por CLI e serviço em segundo plano. Observação de arquivos com reconciliação periódica, streaming, limites de recursos, temporários e publicação atômica. Revisões ordenadas pelo servidor, preservação de conflitos e registros de exclusão. Links simbólicos e arquivos especiais não serão sincronizados inicialmente; caminhos não podem escapar do compartilhamento.

Histórico com versões completas e deduplicação por hash. Retenção configurável, aplicada em manutenção offline: até 30 dias e no máximo 10 versões anteriores por arquivo; lixeira de 30 dias. A versão atual permanece. Restauração cria nova revisão. Limpeza só remove objetos sem referências e precisa respeitar transferências em andamento. Histórico não substitui backup.

Painel administrativo implementado em Rust, com HTML renderizado no servidor, no mesmo endereço da sincronização. Gerencia usuários, pastas, permissões, convites e dispositivos. Chave administrativa separada dos tokens dos clientes, configuração TOML preservada e gravação atômica com controle de revisão (decisão 0012). Publicado na alpha.7; esta atualização acrescenta formatos de instalação para servidor Linux.

## Decisões ainda abertas

Aceite físico ampliado para lançamento estável. Alpha pública autorizada para testes na decisão 0008. Licença GPL-3.0-only definida. Distribuição musl x86_64/ARM64, retenção e runtime Rust foram definidos na [decisão 0006](decisions/0006-operation-native-distribution.md). Protocolo v1 e ponte HTTPS do piloto definidos em [0004-pilot.md](decisions/0004-pilot.md). SHA-256 e persistência local JSON estão definidos em [0002-storage.md](decisions/0002-storage.md). Configuração TOML e ACLs locais estão definidos em [0003-config-authorization.md](decisions/0003-config-authorization.md); a autenticação por dispositivo e a ponte HTTPS foram adicionadas para o piloto (decisão 0004). Documentar escolhas antes de estabilizar contratos. A fundação Cargo, edição Rust e alvo inicial de CI estão registrados em [0001-foundation.md](decisions/0001-foundation.md). Compactação na transmissão é apenas uma otimização futura a avaliar; não é requisito da primeira versão.

## Limites de hospedagem

GitHub Pages hospeda apenas o site estático, em `https://USUARIO.github.io/REPOSITORIO/`. Documentos privados e o serviço de sincronização ficam na infraestrutura do operador. Nunca publicar dados reais, credenciais ou configurações privadas.

## Distribuição do servidor Linux

Distribuição do servidor Linux: pacotes Arch `.pkg.tar.zst`, Debian `.deb` e
Fedora `.rpm`, além de tar.gz upstream. Pacotes nativos usam `/usr/bin`,
`/etc/crewsync` e `/var/lib/crewsync`, serviço systemd e usuário dedicado sem
login. O tar.gz instala em `/opt/crewsync`, `/etc/opt/crewsync` e `/var/opt/crewsync`.
O assistente `crewsync-server-setup` configura o servidor sem iniciar o daemon como
root; detalhes e limites de validação em DISTRIBUTION.md e decisão 0013.
