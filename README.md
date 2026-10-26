# alertmanager-ntfy-adapter

Высокопроизводительный, легковесный вебхук-адаптер на **Rust**, транслирующий уведомления из **Prometheus Alertmanager** в push-уведомления **ntfy**.

---

## Возможности

- **Минимальное потребление ресурсов**: собран в статический musl-бинарник и упакован в образ `FROM scratch` (потребление памяти < 10 МБ).
- **Форматирование уведомлений**:
  - Четкий заголовок со статусом (`[FIRING:1]`, `[RESOLVED:1]`), именем алерта и целевым объектом (`namespace/pod (container)` или `instance`).
  - Приоритеты ntfy: автоматическое сопоставление `critical` ➔ `5` (максимальный звук/пробивание DND), `warning` ➔ `4`, `info` ➔ `3`, `resolved` ➔ `2`.
  - Теги и эмодзи: `rotating_light` (🚨) при срабатывании, `white_check_mark` (✅) при закрытии инцидента.
  - Умная ссылка (Click Action): автоматическое извлечение PromQL-выражения из `generatorURL` и построение ссылки на **Grafana Explore** для моментального открытия графиков при тапе по пушу на телефоне.
- **Поддержка аутентификации**: Bearer-токен через переменную `NTFY_TOKEN` или файл `NTFY_TOKEN_FILE`.
- **Kubernetes-ready**: эндпоинт `/healthz` для liveness/readiness проб и корректный graceful shutdown по сигналам SIGINT/SIGTERM.

---

## Переменные окружения

| Переменная | По умолчанию | Описание |
| :--- | :--- | :--- |
| `NTFY_URL` | `http://ntfy.apps.svc.cluster.local/` | Базовый URL сервера ntfy |
| `NTFY_TOPIC` | `grafana` | Имя топика по умолчанию |
| `NTFY_TOKEN` | *(пусто)* | Bearer-токен авторизации в ntfy |
| `NTFY_TOKEN_FILE` | `/etc/ntfy-token/token` | Путь к файлу с токеном |
| `GRAFANA_URL` | `https://grafana.xinfra.ru` | Внешний URL Grafana |
| `GRAFANA_DATASOURCE_UID` | `prometheus` | UID источника данных Prometheus в Grafana |
| `GRAFANA_ORG_ID` | `1` | ID организации в Grafana |
| `GRAFANA_RANGE_FROM` | `now-1h` | Начало временного диапазона в ссылках Explore |
| `GRAFANA_RANGE_TO` | `now` | Конец временного диапазона в ссылках Explore |
| `LISTEN_ADDR` / `PORT` | `0.0.0.0:8080` | Адрес/порт HTTP-сервера |
| `RUST_LOG` | `info` | Уровень логирования (`info`, `debug`, `trace`) |

---

## Эндпоинты

- `GET /healthz` — проверка работоспособности (возвращает `200 OK`).
- `POST /alertmanager` — приём webhook от Alertmanager (возвращает `204 No Content` при успешной публикации в ntfy или `502 Bad Gateway` при сбое).

---

## Локальная разработка и сборка

### Запуск тестов
```bash
cargo test
```

### Локальный запуск
```bash
NTFY_URL="https://ntfy.xinfra.ru" \
NTFY_TOPIC="grafana" \
NTFY_TOKEN="tk_..." \
cargo run
```

### Сборка статического бинарника
```bash
cargo build --release --target x86_64-unknown-linux-musl
```

---

## Сборка контейнера и CI/CD

В проекте настроен многоэтапный `Dockerfile` на базе `scratch` и Forgejo Actions workflow (`.forgejo/workflows/image.yml`):
- При пуше тега вида `v*` (например, `v0.1.0`) запускается сборка образа.
- Образ автоматически публикуется в реестр `zot.xinfra.ru/alertmanager-ntfy-adapter:<tag>` и `latest`.

---

## Версионирование

Для управления версиями используется `bump-my-version`:
```bash
# Проверить следующую версию
uv run --with bump-my-version bump-my-version show-bump

# Поднять patch (0.1.0 -> 0.1.1) с созданием коммита и git-тега
uv run --with bump-my-version bump-my-version bump patch

# Поднять minor (0.1.0 -> 0.2.0)
uv run --with bump-my-version bump-my-version bump minor
```
