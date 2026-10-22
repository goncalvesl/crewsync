use super::config::Snapshot;
use crate::{auth::Device, config::Permission};

pub fn escape(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        output.push_str(match character {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&#39;",
            _ => {
                output.push(character);
                continue;
            }
        });
    }
    output
}
fn hidden(name: &str, value: &str) -> String {
    format!(
        "<input type=\"hidden\" name=\"{}\" value=\"{}\">",
        escape(name),
        escape(value)
    )
}
fn fields(csrf: &str, revision: &str) -> String {
    hidden("csrf", csrf) + &hidden("revision", revision)
}
fn page(title: &str, body: &str, csrf: Option<&str>) -> String {
    let navigation = csrf.map(|csrf| format!("<nav aria-label=\"Principal\"><a href=\"#usuarios\">Usuários</a><a href=\"#pastas\">Pastas</a><a href=\"#dispositivos\">Dispositivos</a></nav><form class=\"logout\" method=\"post\" action=\"/admin/logout\">{}<button class=\"secondary\">Sair</button></form>", hidden("csrf", csrf))).unwrap_or_default();
    format!(
        "<!doctype html><html lang=\"pt-BR\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{} · CrewSync</title><link rel=\"stylesheet\" href=\"/admin/style.css\"></head><body><header><div class=\"wrap\"><div class=\"brand\">CrewSync<span>Administração do servidor</span></div>{navigation}</div></header><main class=\"wrap\" id=\"conteudo\">{body}</main><footer class=\"wrap\">Pastas compartilhadas. Servidor próprio.</footer></body></html>",
        escape(title)
    )
}
pub fn message(title: &str, text: &str) -> String {
    page(
        title,
        &format!(
            "<div class=\"login\"><div class=\"card\"><h1>{}</h1><p>{}</p><a href=\"/\">Voltar ao painel</a></div></div>",
            escape(title),
            escape(text)
        ),
        None,
    )
}
pub fn login(csrf: &str) -> String {
    page(
        "Entrar",
        &format!(
            "<div class=\"login\"><div class=\"card\"><p class=\"eyebrow\">Acesso administrativo</p><h1>Seu servidor, organizado.</h1><p class=\"muted\">Gerencie as pessoas, pastas e computadores da equipe.</p><form id=\"login-form\" method=\"post\" action=\"/admin/login\">{}<label>Chave administrativa<input name=\"key\" type=\"password\" autocomplete=\"current-password\" required maxlength=\"64\" minlength=\"64\" autofocus></label><button>Entrar no painel</button></form><p><small>Use a chave mostrada na instalação do servidor. Os convites de dispositivos são usados somente nos clientes.</small></p></div></div>",
            hidden("csrf", csrf)
        ),
        None,
    )
}
pub fn invite(user: &str, code: &str, ttl: u64) -> String {
    page(
        "Convite gerado",
        &format!(
            "<div class=\"login\"><div class=\"card\"><p class=\"eyebrow\">Dispositivo novo</p><h1>Convite para {}</h1><p>Copie e entregue ao usuário por um meio confiável. Vale por {} minutos e uma única ativação.</p><label>Convite<input class=\"secret\" id=\"invite-code\" readonly value=\"{}\"></label><p class=\"muted\">Gere outro convite para cada computador, mesmo sendo da mesma pessoa.</p><a href=\"/\">Voltar aos dispositivos</a></div></div>",
            escape(user),
            ttl / 60,
            escape(code)
        ),
        None,
    )
}
pub fn dashboard(snapshot: &Snapshot, devices: &[Device], csrf: &str, filter: &str) -> String {
    let config = &snapshot.config;
    let common = fields(csrf, &snapshot.revision);
    let users: Vec<_> = config.users().collect();
    let shares: Vec<_> = config.shares().collect();
    let user_options = users
        .iter()
        .map(|(name, _)| {
            format!(
                "<option value=\"{}\">{}</option>",
                escape(name),
                escape(name)
            )
        })
        .collect::<String>();
    let active_options = users
        .iter()
        .filter(|(_, enabled)| *enabled)
        .map(|(name, _)| {
            format!(
                "<option value=\"{}\">{}</option>",
                escape(name),
                escape(name)
            )
        })
        .collect::<String>();
    let share_options = shares
        .iter()
        .map(|(name, _)| {
            format!(
                "<option value=\"{}\">{}</option>",
                escape(name),
                escape(name)
            )
        })
        .collect::<String>();
    let mut body = format!(
        "<div class=\"intro\"><p class=\"eyebrow\">Visão geral</p><h1>Um lugar para cuidar da equipe.</h1><p class=\"muted\">Cadastre pessoas, escolha o acesso às pastas e acompanhe os dispositivos.</p></div><div class=\"stats\"><div class=\"stat\"><strong>{}</strong><span>usuários ativos</span></div><div class=\"stat\"><strong>{}</strong><span>pastas compartilhadas</span></div><div class=\"stat\"><strong>{}</strong><span>dispositivos ativos</span></div></div>",
        users.iter().filter(|(_, enabled)| *enabled).count(),
        shares.len(),
        devices
            .iter()
            .filter(|device| !device.revoked && config.user_enabled(&device.user))
            .count()
    );
    body.push_str(&format!("<section id=\"usuarios\" class=\"card\"><div class=\"section-head\"><div><h2>Usuários</h2><p class=\"muted\">Cada pessoa pode usar vários computadores com o mesmo acesso.</p></div></div><form id=\"user-form\" class=\"panel\" method=\"post\" action=\"/admin/user\">{common}{}<label>Nome do usuário<input name=\"user\" required maxlength=\"64\" pattern=\"[A-Za-z0-9_\\-]+\" placeholder=\"Ex.: ana\" autocomplete=\"off\"></label><button>Adicionar usuário</button></form><div class=\"table-wrap\"><table><thead><tr><th>Usuário</th><th>Estado</th></tr></thead><tbody>", hidden("enabled", "true")));
    for (name, enabled) in &users {
        body.push_str(&format!("<tr data-user=\"{}\"><td><strong>{}</strong></td><td><form class=\"inline\" method=\"post\" action=\"/admin/user\">{common}{}<label><span class=\"muted\">Acesso</span><select name=\"enabled\" aria-label=\"Estado de {}\"><option value=\"true\" {}>Ativo</option><option value=\"false\" {}>Bloqueado</option></select></label><button class=\"secondary\">Salvar usuário</button></form></td></tr>", escape(name), escape(name), hidden("user", name), escape(name), if *enabled { "selected" } else { "" }, if *enabled { "" } else { "selected" }));
    }
    if users.is_empty() {
        body.push_str("<tr><td colspan=\"2\" class=\"empty\">Nenhum usuário cadastrado. Adicione a primeira pessoa acima.</td></tr>");
    }
    body.push_str("</tbody></table></div><p><small>Bloquear um usuário impede o acesso de todos os seus dispositivos. Os arquivos são preservados.</small></p></section>");
    body.push_str(&format!("<section id=\"pastas\" class=\"card\"><div class=\"section-head\"><div><h2>Pastas e permissões</h2><p class=\"muted\">Os clientes criam cada pasta autorizada dentro da raiz local escolhida.</p></div></div><div class=\"grid\"><form id=\"share-form\" class=\"panel\" method=\"post\" action=\"/admin/share\">{common}<h3>Nova pasta compartilhada</h3><label>Nome da pasta<input name=\"share\" required maxlength=\"255\" placeholder=\"Ex.: Projetos\"></label><button>Adicionar pasta</button></form><form id=\"permission-form\" class=\"panel\" method=\"post\" action=\"/admin/permission\">{common}<h3>Definir acesso</h3><label>Pasta<select name=\"share\" required><option value=\"\">Escolha a pasta</option>{share_options}</select></label><label>Usuário<select name=\"user\" required><option value=\"\">Escolha o usuário</option>{user_options}</select></label><label>Permissão<select name=\"access\" required><option value=\"\">Escolha a permissão</option><option value=\"none\">Sem acesso</option><option value=\"read\">Somente leitura</option><option value=\"write\">Leitura e escrita</option></select></label><button {}>Salvar permissão</button></form></div><div class=\"table-wrap\"><table><thead><tr><th>Pasta</th><th>Quem pode acessar</th></tr></thead><tbody>", if users.is_empty() || shares.is_empty() { "disabled" } else { "" }));
    for (name, permissions) in &shares {
        let descriptions = permissions
            .iter()
            .take(6)
            .map(|(user, permission)| {
                format!(
                    "{}: {}",
                    escape(user),
                    if *permission == Permission::Write {
                        "leitura e escrita"
                    } else {
                        "leitura"
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(" · ");
        let more = if permissions.len() > 6 {
            format!(" · e mais {}", permissions.len() - 6)
        } else {
            String::new()
        };
        body.push_str(&format!("<tr data-share=\"{}\"><td><strong>{}</strong></td><td class=\"permissions\">{}{more}</td></tr>", escape(name), escape(name), if descriptions.is_empty() { "Sem acesso concedido" } else { &descriptions }));
    }
    if shares.is_empty() {
        body.push_str("<tr><td colspan=\"2\" class=\"empty\">Nenhuma pasta cadastrada.</td></tr>");
    }
    body.push_str("</tbody></table></div><p><small>Leitura e escrita permite criar, alterar e excluir. Subpastas herdam o acesso. Coloque os arquivos pelos clientes; não altere o armazenamento interno do servidor.</small></p></section>");
    body.push_str(&format!("<section id=\"dispositivos\" class=\"card\"><div class=\"section-head\"><div><h2>Dispositivos</h2><p class=\"muted\">Um convite por computador. Revogue somente o dispositivo desejado.</p></div></div><div class=\"grid\"><form id=\"invite-form\" class=\"panel\" method=\"post\" action=\"/admin/invite\">{common}<h3>Ativar outro computador</h3><label>Usuário<select name=\"user\" required>{active_options}</select></label><label>Validade do convite<select name=\"ttl\"><option value=\"600\">10 minutos</option><option value=\"3600\" selected>1 hora</option><option value=\"86400\">24 horas</option></select></label><button {}>Gerar convite</button></form><form class=\"panel\" method=\"get\" action=\"/\"><h3>Conferir por usuário</h3><label>Mostrar dispositivos de<select name=\"user\"><option value=\"\">Todos os usuários</option>", if active_options.is_empty() { "disabled" } else { "" }));
    for (name, _) in &users {
        body.push_str(&format!(
            "<option value=\"{}\" {}>{}</option>",
            escape(name),
            if *name == filter { "selected" } else { "" },
            escape(name)
        ));
    }
    body.push_str("</select></label><button class=\"secondary\">Filtrar dispositivos</button></form></div><div class=\"table-wrap\"><table><thead><tr><th>Computador</th><th>Usuário</th><th>Estado</th><th>Ação</th></tr></thead><tbody>");
    let filtered: Vec<_> = devices
        .iter()
        .filter(|device| filter.is_empty() || device.user == filter)
        .collect();
    for device in filtered.iter().take(100) {
        let active = !device.revoked && config.user_enabled(&device.user);
        body.push_str(&format!("<tr data-device=\"{}\"><td><strong>{}</strong></td><td>{}</td><td><span class=\"badge {}\">{}</span></td><td>", escape(&device.id), escape(&device.name), escape(&device.user), if active { "" } else { "off" }, if device.revoked { "Revogado" } else if active { "Ativo" } else { "Usuário bloqueado" }));
        if !device.revoked {
            body.push_str(&format!("<form method=\"post\" action=\"/admin/revoke\">{common}{}<button class=\"danger\">Revogar dispositivo</button></form>", hidden("device", &device.id)));
        }
        body.push_str("</td></tr>");
    }
    if filtered.is_empty() {
        body.push_str("<tr><td colspan=\"4\" class=\"empty\">Nenhum dispositivo para este usuário. Gere um convite e ative o cliente.</td></tr>");
    }
    body.push_str("</tbody></table></div>");
    if filtered.len() > 100 {
        body.push_str("<p class=\"muted\">Exibindo os primeiros 100 dispositivos. Use o filtro por usuário.</p>");
    }
    body.push_str("<p><small>Alterações de acesso valem nas próximas operações. Revogar ou bloquear não apaga as cópias locais. A sessão administrativa dura até 8 horas e termina ao reiniciar o servidor.</small></p></section>");
    page("Painel", &body, Some(csrf))
}
