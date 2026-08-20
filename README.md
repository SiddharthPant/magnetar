# Magnetar
This project is a production ready template with the following stack:
- Axum
- Askama
- Datastar
- NATS
- Postgres

## Local Setup and Commands
General migration commands are
```
sqlx migrate add <migration-name>
sqlx migrate run
sqlx database reset -y
```
