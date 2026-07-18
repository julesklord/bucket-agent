# Bucket Agent — Plan de Desacoplamiento Completo

> Estado base: fork de xAI Bucket Build (`d5e79b1`)  
> Equipo objetivo: 3-4 contribuidores  
> Horizonte: 6-9 meses de trabajo agentico intenso

---

## Diagnóstico del acoplamiento actual

| Métrica | Valor |
|---------|-------|
| Crates totales en el workspace | 75 |
| Crates con nombre `bucket-*` o `bucket-*` | ~55 |
| Archivos `.rs` con referencias a `bucket.com`, `x.ai`, `superbucket` | **584** |
| Líneas de código en `bucket-agent-core` (runtime del agente) | **318,406** |
| Ocurrencias de lógica de billing/subscripción/crédito | **926** |
| Telemetría enviada a infraestructura xAI | `bucket-telemetry` (4 archivos core) |

El acoplamiento no es aleatorio. Hay **tres capas** distintas de dependencia:

1. **Nominal** — nombres de crates, variables de entorno, rutas de config
2. **Funcional** — lógica de billing, auth SSO, update checker apuntando a x.ai
3. **Estructural** — el agente asume que existe una API xAI en el otro extremo del socket

El plan ataca las tres capas en orden, de afuera hacia adentro.

---

## Fase 1 — Limpieza Nominal (2-3 semanas)

**Objetivo:** Que ningún nombre interno mencione `xai` o `bucket` salvo los que son
literalmente el protocolo de red (nombres de modelos, endpoints).

### 1.1 Renombrar crates

El `Cargo.toml` raíz está marcado como generado. El primer acto real es
**asumir la propiedad del workspace**, reescribiéndolo manualmente:

| Nombre actual | Nombre nuevo |
|---------------|-------------|
| `bucket-agent-core` | `bucket-agent-core` |
| `bucket-tui` | `bucket-tui` |
| `bucket-bin` | `bucket-bin` |
| `bucket-tools` | `bucket-tools` |
| `bucket-workspace` | `bucket-workspace` |
| `bucket-telemetry` | `bucket-telemetry` |
| `bucket-auth` | `bucket-auth` |
| `bucket-memory` | `bucket-memory` |
| `bucket-markdown` | `bucket-markdown` |
| `bucket-config` | `bucket-config` |
| `bucket-updater` | `bucket-updater` |
| `bucket-agent-base` | `bucket-agent-base` |
| `bucket-acp` | `bucket-acp` |
| `bucket-computer-hub-*` | `bucket-hub-*` |
| `bucket-tool-*` | `bucket-tool-*` |
| `bucket-tracing*` | `bucket-tracing*` |
| `bucket-*` (resto) | `bucket-*` |

**Cómo hacerlo agenticamente:** script Rust/sed que:
1. Renombra directorios físicos
2. Actualiza `name` en cada `Cargo.toml`
3. Actualiza todas las referencias de dependencia interna (`bucket-agent-core = ...`)
4. Actualiza todos los `use bucket_agent_core::` en código fuente

### 1.2 Variables de entorno

| Variable actual | Variable nueva |
|----------------|----------------|
| `BUCKET_HOME` | `BUCKET_HOME` |
| `BUCKET_LOG_FILE` | `BUCKET_LOG_FILE` |
| `BUCKET_AUTH_PROVIDER_COMMAND` | `BUCKET_AUTH_PROVIDER_COMMAND` |
| `BUCKET_OIDC_ISSUER` | `BUCKET_OIDC_ISSUER` |
| `BUCKET_MODELS_BASE_URL` | `BUCKET_MODELS_BASE_URL` |
| `BUCKET_API_KEY` | Mantener como alias (backward compat) + `BUCKET_API_KEY` |
| `BUCKET_*` (resto) | `BUCKET_*` |

Mantener los nombres viejos como aliases deprecados por al menos 2 releases.

### 1.3 Rutas de configuración

| Ruta actual | Ruta nueva |
|-------------|------------|
| `~/.bucket/` | `~/.bucket/` |
| `~/.bucket/config.toml` | `~/.bucket/config.toml` |
| `~/.bucket/auth.json` | `~/.bucket/auth.json` |
| `~/.bucket/sessions/` | `~/.bucket/sessions/` |
| `~/.bucket/hooks/` | `~/.bucket/hooks/` |
| `~/.bucket/AGENTS.md` | `~/.bucket/AGENTS.md` |

Con migración automática: si existe `~/.bucket` y no existe `~/.bucket`, copiar y avisar.

---

## Fase 2 — Desacoplamiento de Auth y Billing (3-4 semanas)

**Objetivo:** Eliminar toda lógica que asume que existe una cuenta xAI/bucket.

### 2.1 Eliminar el login screen de bucket.com

Ya hecho el bypass. Lo que queda es **borrar el código muerto**:
- El `AuthMethodKind::BucketCom` y todo su flujo OIDC hacia `auth.x.ai` puede
  quedar como provider opcional pero no debe ser el default ni estar hardcodeado.
- Mover la URL de auth a config: `BUCKET_OIDC_ISSUER` en lugar de `auth.x.ai` hardcoded.

### 2.2 Eliminar la lógica de subscripción/billing (926 ocurrencias)

Esta es la parte quirúrgica más importante. La lógica está en:
- `bucket-agent-core/src/agent/mvp_agent/mod.rs` — subscription gate (ya bypasseado)
- `bucket-tui/src/views/welcome/mod.rs` — credit bar, SuperBucket CTA
- `bucket-tui/src/views/credit_bar.rs` — barra de créditos
- `bucket-tui/src/app/actions.rs` — billing data actions

**Plan:** Reemplazar con un sistema de `ProviderCapabilities`:

```rust
// bucket-agent-core/src/provider/capabilities.rs
pub struct ProviderCapabilities {
    pub has_billing:      bool,   // false para Ollama/custom
    pub has_credit_limit: bool,
    pub supports_streaming: bool,
    pub max_context_tokens: usize,
}
```

El TUI consulta `ProviderCapabilities` para decidir qué mostrar.
Sin billing → sin credit bar, sin SuperBucket banner, sin subscription gate.

### 2.3 Update checker

`bucket-updater` apunta a `https://x.ai/cli/...` para verificar versiones.

Opciones:
- **a)** Apuntar a GitHub Releases del fork (más simple, recomendado)
- **b)** Deshabilitar por defecto, opt-in en config

```toml
# ~/.bucket/config.toml
[cli]
auto_update = true
update_check_url = "https://api.github.com/repos/tu-org/bucket-agent/releases/latest"
```

### 2.4 Telemetría

`bucket-telemetry` envía datos a infraestructura de xAI (OpenTelemetry hacia endpoints xAI).

**Plan:**
1. Vaciar los endpoints hardcodeados
2. Convertir en sink configurable: `BUCKET_TELEMETRY_ENDPOINT` (por defecto: vacío = off)
3. El `unified_log` interno (logging estructurado) puede quedarse — es útil
4. El `external telemetry` (OTLP) solo activa si el usuario configura un endpoint propio

---

## Fase 3 — Desacoplamiento del Runtime del Agente (4-6 semanas)

**Objetivo:** Que `bucket-agent-core` (ex `bucket-agent-core`) no sepa nada de xAI.

Esta es la fase más profunda. El runtime actual tiene tres acoplamentos estructurales:

### 3.1 El cliente de chat (inference client)

Hoy el cliente de completions está mezclado con la lógica del agente.
Necesita extraerse como trait:

```rust
// bucket-agent-core/src/provider/mod.rs
#[async_trait]
pub trait ChatProvider: Send + Sync {
    async fn complete(
        &self,
        messages: &[Message],
        tools:    &[ToolDefinition],
        params:   &SamplingParams,
    ) -> Result<ChatStream>;

    fn capabilities(&self) -> ProviderCapabilities;
    fn model_id(&self)      -> &str;
}
```

Implementaciones:
- `OpenAICompatProvider` — cubre xAI, Ollama, OpenAI, Together, LM Studio
- `AnthropicProvider` — Messages API
- `MockProvider` — para tests

El agente solo habla con `dyn ChatProvider`. Nunca sabe si hay Ollama o bucket detrás.

### 3.2 El sistema de modelos

Hoy hay modelos hardcodeados en el código (lista de modelos xAI, bucket-build, etc.).

**Plan:** Eliminar la lista hardcodeada. Solo modelos de:
1. Configuración del usuario (`config.toml`)
2. Endpoint `/v1/models` del proveedor configurado
3. Un set mínimo de aliases convenientes (opcionales)

```toml
# ~/.bucket/config.toml — el único lugar donde existen modelos
[models]
default = "ollama-coder"

[model.ollama-coder]
model    = "qwen2.5-coder:latest"
base_url = "http://localhost:11434/v1"
```

### 3.3 System prompts y personalidad del agente

El system prompt actual menciona xAI, Bucket, etc.
Necesita ser configurable:

```toml
# ~/.bucket/config.toml
[agent]
system_prompt = """
You are Bucket, an agentic coding assistant.
Work autonomously, read files, edit code, run tests.
"""
```

Con un default sensato que no mencione ninguna empresa.

### 3.4 Serialización de sesiones

Las sesiones guardadas incluyen metadata con nombres de modelos xAI.
Necesita ser agnóstica al provider — solo almacena `model_id` y `base_url`,
no asume nada sobre quién sirve ese modelo.

---

## Fase 4 — Infraestructura del Proyecto (2-3 semanas)

**Objetivo:** Que el proyecto pueda vivir y mantenerse sin depender de xAI.

### 4.1 CI/CD propio

```yaml
# .github/workflows/ci.yml
- cargo test --all-targets
- cargo clippy --all-targets
- cargo build --bin bucket --release
```

Releases automáticos en GitHub Actions cuando se taggea una versión.
Binarios para Linux/macOS/Windows (cross-compilation con `cross`).

### 4.2 Release pipeline

```
git tag v0.3.0
→ GitHub Actions compila para x86_64-linux, aarch64-linux, x86_64-darwin, aarch64-darwin
→ Sube assets al GitHub Release
→ Actualiza install script
```

Script de instalación autónomo (no depende de x.ai):

```sh
curl -fsSL https://raw.githubusercontent.com/tu-org/bucket-agent/main/install.sh | bash
```

### 4.3 Documentación técnica del fork

- `ARCHITECTURE.md` — mapa de crates y responsabilidades
- `PROVIDERS.md` — cómo añadir un nuevo provider
- `HACKING.md` — setup de entorno de desarrollo en 5 minutos

### 4.4 Config de Rust toolchain

El `rust-toolchain.toml` apunta a una versión pinned. Manterlo pinned pero
asegurarse de hacer bump periódico (cada 4-6 semanas) para no acumular deuda.

---

## Fase 5 — Extensibilidad Comunitaria (ongoing)

**Objetivo:** Que cualquier contribuidor pueda añadir providers y herramientas
sin tocar el core.

### 5.1 Plugin API estable para providers

Exponer `ChatProvider` como trait público en `bucket-acp` para que
providers externos se puedan escribir como crates separados:

```toml
# Cargo.toml de un plugin externo
[dependencies]
bucket-acp = "0.3"
```

```rust
struct MyCorporateProvider { ... }
impl ChatProvider for MyCorporateProvider { ... }
```

### 5.2 MCP como vector de extensión

El soporte MCP ya existe y está bien implementado. Es el canal correcto
para herramientas de terceros. Enfocarse en que la configuración de MCP sea
lo más simple posible:

```toml
# ~/.bucket/config.toml
[[mcp.server]]
name    = "github"
command = "npx -y @modelcontextprotocol/server-github"

[[mcp.server]]
name    = "postgres"
command = "npx -y @modelcontextprotocol/server-postgres postgresql://localhost/mydb"
```

### 5.3 Skills y plugins como primer ciudadano

El sistema de skills (`SKILL.md`) ya funciona. Crear un registry público:
- GitHub repo `bucket-agent/skills` con skills verificadas por la comunidad
- Instalación: `bucket skills install github/bucket-agent/skills/rust-expert`

---

## Mapa de dependencias a eliminar

```
Hoy                          →  Objetivo
──────────────────────────────────────────────────────
auth.x.ai (OIDC endpoint)   →  BUCKET_OIDC_ISSUER configurable
bucket.com (login)             →  Provider OIDC genérico o ninguno
x.ai/cli (updates)           →  GitHub Releases API
OTLP → xAI infra             →  BUCKET_TELEMETRY_ENDPOINT o noop
bucket_* (crate names)    →  bucket_* 
BUCKET_* (env vars)            →  BUCKET_* (con alias BUCKET_* deprecated)
~/.bucket (config dir)         →  ~/.bucket (con migración automática)
hardcoded model list         →  solo config.toml + /v1/models del provider
SuperBucket/billing UI         →  ProviderCapabilities.has_billing flag
system prompt con xAI refs   →  configurable, default neutro
```

---

## Orden de prioridad para el equipo de 3-4 personas

| Sprint | Responsable sugerido | Tarea |
|--------|---------------------|-------|
| 1 | Agente + 1 persona | Fase 1 completa (renombrado masivo automatizable) |
| 2-3 | 2 personas | Fase 2: billing/auth/update/telemetry |
| 4-6 | 3 personas | Fase 3: ChatProvider trait + desacoplamiento del runtime |
| 7 | 1 persona | Fase 4: CI/CD, install script, releases |
| ongoing | todos | Fase 5: extensibilidad, docs, comunidad |

La Fase 1 es la más mecánica y la más larga en LOC pero la más agentica:
un agente puede hacer el 90% del renombrado de forma automatizada en horas.

La Fase 3 es la más intelectualmente costosa — requiere diseñar el trait
`ChatProvider` con cuidado para no romper el contrato con la TUI y el protocolo ACP.

---

## Qué NO tocar (al menos en v1 del desacoplamiento)

- **El protocolo ACP** — está bien diseñado y es agnóstico a xAI. Keeper.
- **El sistema de sesiones** — sólido, solo necesita limpieza nominal.
- **El motor de TUI (ratatui)** — no tiene acoplamiento con xAI, solo con datos.
- **Los tests existentes** — base sólida. Extender, no reescribir.
- **El sistema de hooks** — genérico y potente. Solo renombrar rutas.
- **La arquitectura de subagentes** — avanzada y valiosa. Keeper.

---

## Riesgo principal

xAI puede liberar una actualización grande del código fuente (como hicieron
varias veces en 2025). Cada release nuevo del upstream puede divergir del fork
en áreas que ya tocamos.

**Mitigación:** Mantener un `UPSTREAM_DIFF.md` que documente exactamente qué
cambiamos y por qué, para que hacer cherry-pick de fixes del upstream sea
mecánico. El equipo de 3-4 puede rotar la tarea de "upstream sync" mensualmente.

---

## Resultado final

Al terminar las 5 fases, `bucket` es:

- **Completamente autónomo** — cero dependencias de infraestructura xAI en runtime
- **Multiprovider nativo** — Ollama, OpenAI, Anthropic, cualquier endpoint compatible
- **Sin login obligatorio** — arranca sin cuenta, sin browser, sin nada
- **Mantenible por la comunidad** — CI propio, releases propios, extensible
- **Compatible con el upstream** — los cambios estructurales de xAI se pueden
  incorporar selectivamente sin romper el desacoplamiento
