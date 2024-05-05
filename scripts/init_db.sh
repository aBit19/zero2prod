#! /usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ] 
then
  PSQL_PRESENT=$false
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo "Error: sqlx is not installed." >&2
  echo "Run cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres" >&2
  exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_NAME:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

if [[ -z "${SKIP_DOCKER}" ]] 
then
  RUNNING_PG=$(docker ps --filter 'name=pg_newsletter' --format '{{.ID}}')
  if [[ -n $RUNNING_PG ]]; then
    echo >&2 "there is a postgres container already running, kill it with"
    echo >&2 "    docker kill ${RUNNING_PG}"
    exit 1
  fi
docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p ${DB_PORT}:5432 \
    --name pg_newsletter \
    -d postgres \
    postgres -N 1000
fi

if [[ $PSQL_PRESENT ]]; then
  until PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
  done
else
  >&2 echo "PSQL is not present, waiting for 3 seconds before trying to migrate the database"
  sleep 3
fi

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"
