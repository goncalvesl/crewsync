#!/bin/sh
# CA e certificado HTTPS para domínio DNS ou IPv4 de acesso dos clientes.
set -eu
if [ "$#" -ne 2 ]; then
    echo "Uso: $0 NOVO_DIRETORIO DOMINIO_OU_IPV4 | --check-address DOMINIO_OU_IPV4" >&2
    exit 2
fi
cert_dir=$1
server_host=$2
# A mesma validação atende o assistente antes de qualquer alteração no disco.
# Não aceitar URLs, portas, curingas, múltiplas linhas nem sintaxe OpenSSL.
if ! printf '%s\n' "$server_host" | LC_ALL=C awk '
    NR != 1 || length($0) > 253 { exit 1 }
    /^[0-9.]+$/ {
        n=split($0, octets, ".")
        if(n != 4) exit 1
        for(i=1;i<=4;i++)
            if(octets[i] !~ /^[0-9]+$/ || length(octets[i])>3 || octets[i]+0>255 || (length(octets[i])>1 && substr(octets[i],1,1)=="0")) exit 1
        next
    }
    {
        if($0 !~ /^[a-zA-Z0-9.-]+$/ || ($0 != "localhost" && index($0,".")==0)) exit 1
        n=split($0, labels, ".")
        for(i=1;i<=n;i++)
            if(length(labels[i])<1 || length(labels[i])>63 || labels[i] !~ /^[a-zA-Z0-9]/ || labels[i] !~ /[a-zA-Z0-9]$/) exit 1
    }
    END { if(NR != 1) exit 1 }
'; then
    echo 'Endereço inválido. Informe um domínio (sync.empresa.com) ou IPv4, sem https://, porta ou caminho.' >&2
    exit 2
fi
[ "$cert_dir" != --check-address ] || exit 0
case "$server_host" in *[!0-9.]*) server_san=DNS:$server_host ;; *) server_san=IP:$server_host ;; esac
umask 077
mkdir -m 700 "$cert_dir"
openssl req -x509 -newkey rsa:3072 -sha256 -nodes -days 365 \
    -subj '/CN=CrewSync CA' \
    -addext 'basicConstraints=critical,CA:TRUE' \
    -addext 'keyUsage=critical,keyCertSign,cRLSign' \
    -keyout "$cert_dir/ca.key" -out "$cert_dir/ca.crt"
openssl req -new -newkey rsa:3072 -sha256 -nodes \
    -subj '/CN=CrewSync Server' \
    -keyout "$cert_dir/server.key" -out "$cert_dir/server.csr"
cat > "$cert_dir/extensions.cnf" <<EXTENSIONS
basicConstraints=critical,CA:FALSE
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=DNS:localhost,IP:127.0.0.1,$server_san
EXTENSIONS
openssl x509 -req -in "$cert_dir/server.csr" -CA "$cert_dir/ca.crt" \
    -CAkey "$cert_dir/ca.key" -CAcreateserial -days 90 -sha256 \
    -extfile "$cert_dir/extensions.cnf" -out "$cert_dir/server.crt"
openssl verify -CAfile "$cert_dir/ca.crt" "$cert_dir/server.crt"
echo "Copie apenas ca.crt para os clientes. As chaves .key permanecem privadas."
