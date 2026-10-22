#!/bin/sh
# CA e certificado de servidor para teste privado em uma rede IPv4.
set -eu
if [ "$#" -ne 2 ]; then
    echo "Uso: $0 NOVO_DIRETORIO IP_DO_SERVIDOR" >&2
    exit 2
fi
cert_dir=$1
server_ip=$2
/usr/bin/python3 -I -c 'import ipaddress, sys; ipaddress.IPv4Address(sys.argv[1])' "$server_ip"
umask 077
mkdir -m 700 "$cert_dir"
openssl req -x509 -newkey rsa:3072 -sha256 -nodes -days 365 \
    -subj '/CN=CrewSync Pilot CA' \
    -addext 'basicConstraints=critical,CA:TRUE' \
    -addext 'keyUsage=critical,keyCertSign,cRLSign' \
    -keyout "$cert_dir/ca.key" -out "$cert_dir/ca.crt"
openssl req -new -newkey rsa:3072 -sha256 -nodes \
    -subj '/CN=CrewSync Pilot Server' \
    -keyout "$cert_dir/server.key" -out "$cert_dir/server.csr"
cat > "$cert_dir/extensions.cnf" <<EXTENSIONS
basicConstraints=critical,CA:FALSE
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=DNS:localhost,IP:127.0.0.1,IP:$server_ip
EXTENSIONS
openssl x509 -req -in "$cert_dir/server.csr" -CA "$cert_dir/ca.crt" \
    -CAkey "$cert_dir/ca.key" -CAcreateserial -days 90 -sha256 \
    -extfile "$cert_dir/extensions.cnf" -out "$cert_dir/server.crt"
openssl verify -CAfile "$cert_dir/ca.crt" "$cert_dir/server.crt"
echo "Copie apenas ca.crt para os clientes. As chaves .key permanecem privadas."
