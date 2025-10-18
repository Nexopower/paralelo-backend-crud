# Backend CRUD (Actix Web + MSSQL)

Este repositorio contiene el backend en Rust (Actix Web) usado para la tarea: "Desarrollar una app móvil con login y CRUD aplicando concurrencia (hilos, async/await, coroutines, etc.), siguiendo el patrón MVVM". El backend expone endpoints para registro/login y CRUD de usuarios, y un endpoint demo que realiza carga de datos concurrente.

## Requisitos
- Rust toolchain (stable)
- MSSQL accesible (puede ser local o en red)
- Variables en `.env` (ver abajo)

## Variables de entorno (.env)
- DATABASE_USERNAME - usuario DB
- DATABASE_PASSWORD - contraseña DB
- DATABASE_HOST - host DB (por ejemplo 127.0.0.1)
- DATABASE_PORT - puerto DB (1433 por defecto)
- DATABASE_NAME - nombre de la BD
- PORT - puerto donde corre la app (8080 por defecto)
- JWT_SECRET - secreto usado para derivar la "clave" de cifrado del token (default `secret123` si no se define)
- CONCURRENCY_LIMIT - número máximo de consultas DB concurrentes en `/load_concurrent` (default 20)
- DB_QUERY_TIMEOUT_SECS - timeout por consulta en segundos (default 5)
- FAIL_FAST - `true`/`false` si quieres que `/load_concurrent` falle al primer error (default `false`)

## Cómo ejecutar
1. Crear/editar `.env` con las variables.
2. Compilar y ejecutar:

```powershell
cargo build
cargo run
```

La aplicación arrancará y quedará escuchando en `0.0.0.0:{PORT}`.

## Endpoints
- POST /login
  - Body: `{ "username": "...", "password": "..." }`
  - Response: `200 { "token": "..." }` o `401`

- POST /users
  - Body: `{ "username": "...", "email": "...", "password": "..." }`
  - Response: `201` con el usuario creado

- GET /users
  - Response: `200` con la lista de usuarios

- GET /users/{id}
  - Response: `200` con usuario o `404`

- PUT /users/{id}
  - Body: `{ "username"?: "...", "email"?: "...", "password"?: "..." }`
  - Response: `200` con usuario actualizado

- DELETE /users/{id}
  - Response: `204` o `404`

- GET /load_concurrent
  - Demo de carga concurrente: obtiene la lista de usuarios y consulta cada usuario de forma concurrente.
  - Control de concurrencia: usa `CONCURRENCY_LIMIT` para limitar paralelismo.
  - Timeout por consulta: `DB_QUERY_TIMEOUT_SECS`.
  - Fail-fast: activar `FAIL_FAST=true` para que falle ante el primer error.

## Notas sobre concurrencia
- El servidor usa Tokio + SQLx: las consultas son asíncronas y no crean un hilo por petición.
- En `/load_concurrent` se ejecutan múltiples consultas en paralelo con `buffer_unordered` para limitar concurrencia.
- Si necesitas comportamiento "falla rápido" (equivalente exacto a `Promise.all` que rechaza al primer fallo), activa `FAIL_FAST=true`.
- Ajusta `CONCURRENCY_LIMIT` según la capacidad de tu servidor y pool de conexiones.

## Recomendaciones finales
- Añade middleware de autenticación para proteger rutas usando el token almacenado en la tabla `usertoken` (SPs `SP_VALIDATE_TOKEN` / `SP_GET_USER_TOKEN` existentes en la DB).
- Añadir script `db/init.sql` con DDL y SPs para reproducibilidad.
- Agregar tests de integración que cubran crear usuario -> login -> usar `/load_concurrent`.

---

Si quieres, puedo:
- generar el `db/init.sql` de ejemplo;
- añadir tests básicos (cargo test) que cubran el flujo;
- implementar el middleware de autenticación y proteger rutas.

Dime cuál quieres que agregue ahora.