# CrewSync 0.1.0-alpha.1

Primeira release pública para testes em Linux, sob GPL-3.0-only.

## Download e instalação

Escolha o pacote x86_64 (Intel/AMD) ou aarch64 (ARM64), extraia e execute:

```sh
./install.sh --client
```

No servidor, use `./install.sh --server`; sem opção, instala os dois. Não exige
Rust/Cargo/Python no uso diário nem ajustes no PATH. Manual simples no pacote e em
https://goncalvesl.github.io/crewsync/manual.html. Cada pacote tem checksum SHA-256.
A fonte correspondente, scripts de build e dependências estão em `crewsync-0.1.0-alpha.1-source.tar.gz`.

## Funcionalidades

- Servidor HTTPS nativo Rust com CA e endereço verificados.
- Convite de uso único, token por dispositivo, revogação e permissões read/write.
- Sincronização manual ou por eventos com reconciliação periódica, exclusões
  persistentes, diretórios vazios e conflitos preservados.
- Histórico/restauração, retenção configurável e coleta segura com servidor parado.
- Backup verificável e restauração em destino novo.

## Compatibilidade e validação

Linux kernel >=5.6 com /proc e filesystem local. Pacotes estáticos musl x86_64 e
ARM64, sem dependência de glibc. Configuração e sessões do piloto são preservadas;
faça backup com serviço parado antes de atualizar. Não rebaixar após histórico v2.

Testes locais: 72 Rust, 5 HTTPS, 3 TLS, gerador de certificados, instalador, geração
do manual e preparação/gates de release. A publicação é condicionada à matriz
nativa x86_64/ARM64, ENOSPC real isolado, smoke dos pacotes e build offline das fontes.

## Limitações da alpha

Rede privada, servidor serializado, sem quotas totais, retomada parcial,
compactação ou limpeza automática de cópias locais. Windows/GUI/Android são futuros.
O piloto básico foi confirmado pelo mantenedor; matriz física ampliada e medições
entre máquinas continuam pendentes. Esta é uma alpha para teste, não uma versão
estável nem substituto de backup.
