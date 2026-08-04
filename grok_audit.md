# Summary

**Bucket** es un monorepo Rust maduro (~1.3M LOC, ~74 crates) con arquitectura clara de capas, un sistema de permisos serio (tree-sitter + fail-closed), y un fork bien instrumentado para tracking upstream. La calidad de ingeniería del core (permissions, FS helpers, ACP session) es alta.

Los riesgos principales no son bugs puntuales, sino **deuda estructural**: god-files de 6–11k LOC, lints desactivados a nivel de crate, CI que no cubre crates críticos, y versionado inconsistente entre crates del workspace.

**Veredicto:** codebase production-grade con buena base de seguridad y tests; el coste de evolución está subiendo por tamaño de módulos y cobertura CI incompleta.

---

## Arquitectura

```
bucket-bin          composition root (binary `bucket`)
    ├── bucket-tui          TUI / scrollback / modals / slash
    ├── bucket-agent-core   agent runtime, sessions, leader, auth
    ├── bucket-tools        tool implementations
    ├── bucket-workspace    FS, VCS, permissions, execution
    └── bucket-updater / telemetry / crash-handler / …
```

| Fortaleza | Detalle |
|-----------|---------|
| Separación de capas | bin → tui/agent-core → tools/workspace es limpia |
| Multiprovider | Discovery BYOK + models.dev cache bien aislado en `provider_models.rs` |
| Permissions | `shell_access` + `policy` + tree-sitter; fail-closed a Ask |
| Fork hygiene | `SOURCE_REV`, `rename_mapping.json`, `scripts/upstream-diff.sh` |
| Docs de usuario | 20+ guías en `bucket-tui/docs/user-guide/` |
| Tooling | toolchain pin 1.92.0, clippy ban de `canonicalize`, release multi-arch |

Flujo de arranque coherente: `bucket-bin` compone leader/stdio/headless/TUI y carga config/env al startup.

---

## Issues por severidad

### 1 — Severity: bug / risk
**Archivos monstruo (god-files)**

| LOC | Archivo |
|-----|---------|
| 11 465 | `bucket-agent-core/src/agent/config.rs` |
| 10 636 | `bucket-tui/src/app/app_view.rs` |
| 10 264 | `bucket-tui/src/views/dashboard/state.rs` |
| 9 487 | `bucket-workspace/src/handle.rs` |
| 9 481 | `bucket-sampling-types/src/conversation.rs` |
| 7 539 | `bucket-mcp/src/servers.rs` |
| 6 210 | `bucket-agent-core/src/leader/server.rs` |

**Problema:** review, bisect y merge se vuelven caros; la densidad de `unwrap`/`expect` en estos archivos es alta (p.ej. `handle.rs` ~318, `leader/server.rs` ~331).  
**Sugerencia:** partir por dominio (`config/{load,resolve,models,endpoints}.rs`, `handle/{fs,git,session}.rs`). No reescribir: extraer módulos con `mod` y `pub(crate) use`.

---

### 2 — Severity: bug / risk
**Lints desactivados a nivel de crate**

```1:7:crates/codegen/bucket-agent-core/src/lib.rs
#![allow(
    unused_imports,
    unused_variables,
    unused_mut,
    unreachable_code,
    dead_code
)]
```

Mismo patrón en `bucket-bin`, `bucket-workspace`, `bucket-env`, `bucket-config-types`, etc.

**Problema:** oculta dead code real y imports rotos tras renames del fork; el compilador deja de ser red de seguridad.  
**Sugerencia:** quitar el `#![allow(...)]` crate-wide y permitir solo en sitios justificados. Empezar por `bucket-bin` y `bucket-agent-core`.

---

### 3 — Severity: bug / risk
**CI incompleto para crates de superficie**

`ci.yml` hace check de `bucket-agent-core`, `bucket-tui`, `bucket-updater` y tests de agent-core/updater/config.  
**No** corre de forma targeted:

- `bucket-tools` (~112k LOC)
- `bucket-workspace` (~78k LOC) — permissions, shell
- `bucket-mcp`, `bucket-auth`, `bucket-sampler`, `bucket-tui-render`

`rust.yml` hace workspace completo, pero si es más lento/menos frecuente, regresiones en tools/workspace pueden llegar a `main` sin señal temprana.  
**Sugerencia:** añadir al job `check`/`test` al menos `bucket-tools` y `bucket-workspace` (son el blast radius de seguridad).

---

### 4 — Severity: suggestion
**Violaciones del ban de `canonicalize`**

`clippy.toml` prohíbe `tokio::fs::canonicalize` / `std::fs::canonicalize`.  
`bucket-tools/src/util/fs.rs` lo usa bien (wrapper + `dunce::simplified` + `#[allow]`).  
Pero `bucket-workspace/src/handle.rs` (~1649, ~1714) llama `tokio::fs::canonicalize` **directamente** sin el helper bendecido.

**Sugerencia:** redirigir a `bucket_tools::util::fs` o a `spawn_blocking` + `dunce::canonicalize` para Windows y consistencia de paths.

---

### 5 — Severity: suggestion
**Secret impreso en stdout del serve**

```86:97:crates/codegen/bucket-bin/src/main.rs
fn print_serve_startup_info(bind_addr: SocketAddr, secret: &str) {
    ...
    eprintln!("   Secret:   {}", secret);
    ...
    eprintln!("   WebSocket URL: ws://{}/ws?server-key={}", bind_addr, secret);
}
```

Útil en local; riesgoso si logs se capturan/comparten.  
**Sugerencia:** enmascarar por defecto (`****abcd`) y mostrar completo solo con `--show-secret` o similar.

---

### 6 — Severity: suggestion
**Versionado inconsistente del workspace**

| Crate | Version |
|-------|---------|
| `bucket-bin` | `0.1.9` |
| `bucket-version` | `0.1.4` |
| `bucket-tools` / `bucket-tools-api` | `0.1.220-alpha.4` |
| mayoría de leaf crates | `0.1.0` |

El binario se publica como `0.1.9` pero el crate de versión embebida y tools viven en otra escala.  
**Sugerencia:** lockstep de `bucket-version` con el release, o documentar explícitamente que solo `bucket-bin` es la versión de producto.

---

### 7 — Severity: suggestion
**Residuo del rename del fork**

- Snapshots insta aún con prefijo `xai_grok_pager__...`
- Campo CLI `xai_api_base_url` en `main.rs`
- Comentarios/paths con naming legacy en algunos sitios

No rompe runtime, pero confunde contributors y búsquedas.  
**Sugerencia:** renombrar snapshots (insta regenera) y alias deprecado para flags `xai_*` → `bucket_*`.

---

### 8 — Severity: nit
**`#![allow]` locales densos en TUI**

`bucket-tui/src/app/acp_handler/mod.rs` acumula muchos `#[allow(unused_imports)]`. Suele indicar re-exports o módulos semi-muertos tras refactors. Limpieza incremental.

---

### 9 — Severity: nit
**Ratio tests vs tamaño**

~229k LOC en paths de test sobre ~1.3M total (~17%). La suite de `agent-core` (session, goals, permissions) es densa y de calidad; `tools`/`workspace` merecen más peso en CI aunque ya tengan tests locales.

---

## Fortalezas destacables

1. **Permissions de shell** — parse con tree-sitter, nesting de `bash -c`, fail-closed a Ask; diseño consciente de adversarial input.
2. **FS helpers** — timeout 30s + fallback en overlayfs; Windows path hygiene documentada.
3. **Fork multiprovider** — discovery Ollama/OpenAI/Anthropic/OpenRouter/Groq/NIM + cache models.dev; fixes recientes de serde robusto (`fdc87c3`, `f7e78c2`) van en la dirección correcta.
4. **Release pipeline** — multi-arch, install.sh interactivo, Makefile, CHANGELOG Keep a Changelog.
5. **Observabilidad** — telemetry, crash-handler, jemalloc profiling en release-dist.
6. **ACP / headless / leader** — varios modos de ejecución bien separados en el composition root.

---

## Mapa de riesgo por área

| Área | Riesgo | Por qué |
|------|--------|---------|
| Permissions / shell | Medio-bajo | Diseño sólido; necesita CI estable |
| Config monstruo | Alto | 11k LOC, hard to change safely |
| TUI app_view / dashboard | Alto | UI state acoplado, archivos enormes |
| Provider discovery | Medio | Superficie nueva del fork; bien testeada unitariamente |
| Auth / tokens | Medio | Código cuidadoso; poco en CI targeted |
| Upstream sync | Medio | `SOURCE_REV` existe; drift crece con cada feature fork-only |
| Windows | Medio | Best-effort; ban de canonicalize existe pero no se cumple en todos lados |

---

## Prioridades recomendadas

| Prioridad | Acción | Impacto |
|-----------|--------|---------|
| P0 | Meter `bucket-tools` + `bucket-workspace` en CI targeted | Atrapa regresiones de seguridad/ejecución |
| P1 | Quitar `#![allow(dead_code, unused_*, unreachable_code)]` crate-wide | Recupera señal del compilador |
| P1 | Partir `config.rs` y `handle.rs` sin cambiar comportamiento | Baja coste de evolución |
| P2 | Unificar `canonicalize` en workspace al helper bendecido | Consistencia Windows + clippy |
| P2 | Lockstep `bucket-version` con releases del binario | Menos confusión de soporte |
| P3 | Limpiar residue `xai_grok_*` en snaps/flags | Higiene del fork |
| P3 | Secret masking en `serve` | Hardening menor |

---

## Contexto del fork

- **Upstream base:** `SOURCE_REV = 2ec0f0c…`
- **Producto actual:** `v0.1.9` en `main`
- **Valor añadido del fork:** multiprovider real, model picker, models.dev, install UX, Zed integration, retry classification
- **Tensión inherente:** cada feature fork-only encarece el merge upstream; el tooling de diff ayuda, pero los god-files amplifican el dolor

---

Si quieres profundizar, puedo hacer un review más enfocado en cualquiera de:

1. **Security deep-dive** (permissions + sandbox + serve)
2. **Plan de split** de `config.rs` / `app_view.rs` / `handle.rs`
3. **CI hardening** con un PR concreto
4. **Review de un área** (TUI, agent session loop, provider models)

¿Por cuál seguimos?
